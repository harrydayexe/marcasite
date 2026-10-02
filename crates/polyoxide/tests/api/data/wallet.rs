//! Wallet endpoints: `/v2/positions`, `/v2/positions/combos`, `/v2/value`,
//! `/v2/approvals`, `/v2/user-pnl`, `/v2/user-stats`, `/v2/user-volume`.

use chrono::DateTime;
use futures_util::TryStreamExt as _;
use polyoxide::{
    Decimal, Error,
    data::{
        ComboPositionSortBy, ComboPositionStatus, PnlFidelity, PnlInterval, PositionSortBy,
        PositionStatus, SortDirection, TokenStandard,
    },
    types::TokenId,
};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use super::{
    fixtures::{self, COMBO_CONDITION, CONDITION, CONDITION_2, WALLET, WALLET_2},
    received_queries,
};
use crate::common;

#[tokio::test]
async fn list_positions_sends_filters_and_decodes_page() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/positions"))
        .and(query_param("user", WALLET))
        .and(query_param("status", "REDEEMABLE"))
        .and(query_param("limit", "1"))
        .and(query_param_is_missing("offset"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-trace-id", "trace-positions")
                .set_body_json(fixtures::page(
                    vec![fixtures::position(fixtures::TOKEN)],
                    Some("eyJkYXRhIjp7InR5cGUiOiJwb3NpdGlvbnMi"),
                )),
        )
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .data()
        .list_positions()
        .user(WALLET)
        .status(PositionStatus::Redeemable)
        .sort_by(PositionSortBy::CurrentValue)
        .sort_direction(SortDirection::Desc)
        .title("Slovan")
        .start(DateTime::from_timestamp(1, 0).unwrap())
        .limit(1)
        .send()
        .await
        .unwrap();
    assert_eq!(page.items().len(), 1);
    let position = &page.items()[0];
    assert_eq!(position.status, PositionStatus::Redeemable);
    assert_eq!(position.entry_cost_usdc.to_string(), "45159.4653");
    assert_eq!(
        page.next_cursor(),
        Some("eyJkYXRhIjp7InR5cGUiOiJwb3NpdGlvbnMi")
    );
    assert!(page.has_more());
    assert_eq!(page.trace_id(), Some("trace-positions"));
    assert_eq!(
        received_queries(&server).await,
        vec![format!(
            "user={WALLET}&limit=1&status=REDEEMABLE&title=Slovan&sort_by=CURRENT_VALUE&start=1&sort_direction=DESC"
        )]
    );
}

#[tokio::test]
async fn list_positions_stream_resends_anchor_and_filters() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/positions"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![fixtures::position("1"), fixtures::position("2")],
            Some("page-2"),
        )))
        .expect(1)
        .mount(&server)
        .await;
    // `has_more` is exact: an empty page with a cursor does not end the walk.
    Mock::given(method("GET"))
        .and(path("/v2/positions"))
        .and(query_param("cursor", "page-2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::page(Vec::new(), Some("page-3"))),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/positions"))
        .and(query_param("cursor", "page-3"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::position("3")], None)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let tokens: Vec<TokenId> = common::polymarket(&server)
        .data()
        .list_positions()
        .conditions([CONDITION])
        .title("Slovan")
        .limit(2)
        .into_stream()
        .map_ok(|p| p.token_id)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(
        tokens,
        vec![TokenId::from("1"), TokenId::from("2"), TokenId::from("3")]
    );
    let queries = received_queries(&server).await;
    assert_eq!(
        queries,
        vec![
            format!("condition={CONDITION}&limit=2&title=Slovan"),
            format!("condition={CONDITION}&limit=2&cursor=page-2&title=Slovan"),
            format!("condition={CONDITION}&limit=2&cursor=page-3&title=Slovan"),
        ]
    );
}

#[tokio::test]
async fn list_positions_validates_before_sending() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let data = common::polymarket(&server).data().clone();

    let err = data.list_positions().send().await.unwrap_err();
    let Error::Validation(v) = &err else {
        panic!("expected Error::Validation, got {err:?}")
    };
    assert_eq!(v.parameter(), "user");

    let too_many: Vec<String> = (0..21).map(|i| format!("0x{i:064x}")).collect();
    let err = data
        .list_positions()
        .user(WALLET)
        .conditions(&too_many)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "condition"));

    let err = data
        .list_positions()
        .user(WALLET)
        .limit(1001)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "limit"));

    // `event_id` and `REDEEMABLE_LOST` are user-anchored only.
    let err = data
        .list_positions()
        .conditions([CONDITION])
        .event_ids(["12345"])
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "event_id"));
    let err = data
        .list_positions()
        .conditions([CONDITION])
        .status(PositionStatus::RedeemableLost)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "status"));

    // Malformed ids and an empty wallet never reach the server.
    let err = data
        .list_positions()
        .user(WALLET)
        .conditions(["0xzz"])
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "condition"));
    let err = data
        .list_positions()
        .user("")
        .conditions([CONDITION])
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "user"));

    // A validation failure in a stream is yielded as its first item.
    let result: Result<Vec<_>, _> = data.list_positions().into_stream().try_collect().await;
    assert!(matches!(result, Err(Error::Validation(_))));
}

#[tokio::test]
async fn list_positions_sends_each_condition_once() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/positions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(Vec::new(), None)))
        .expect(2)
        .mount(&server)
        .await;
    let data = common::polymarket(&server).data().clone();

    // Without `user` exactly one id is accepted; the same id twice is one id.
    data.list_positions()
        .conditions([CONDITION, CONDITION])
        .send()
        .await
        .unwrap();
    data.list_positions()
        .user(WALLET)
        .conditions([CONDITION, CONDITION_2, CONDITION])
        .send()
        .await
        .unwrap();
    assert_eq!(
        received_queries(&server).await,
        vec![
            format!("condition={CONDITION}"),
            format!("user={WALLET}&condition={CONDITION}%2C{CONDITION_2}"),
        ]
    );
}

#[tokio::test]
async fn stream_stops_when_has_more_is_false_despite_a_cursor() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/positions"))
        .and(query_param_is_missing("cursor"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::page_with(
                vec![fixtures::position("1")],
                Some("stale-cursor"),
                false,
            )),
        )
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/positions"))
        .and(query_param("cursor", "stale-cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(Vec::new(), None)))
        .expect(0)
        .mount(&server)
        .await;

    let data = common::polymarket(&server).data().clone();
    let page = data.list_positions().user(WALLET).send().await.unwrap();
    assert!(!page.has_more());
    assert_eq!(page.next_cursor(), None);
    assert_eq!(page.pagination.next_cursor.as_deref(), Some("stale-cursor"));

    let positions: Vec<_> = data
        .list_positions()
        .user(WALLET)
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(positions.len(), 1);
}

#[tokio::test]
async fn stream_stops_on_a_cursor_cycle() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/trades"))
        .and(query_param_is_missing("cursor"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::trade("0x1")], Some("a"))),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/trades"))
        .and(query_param("cursor", "a"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::trade("0x2")], Some("b"))),
        )
        .expect(1)
        .mount(&server)
        .await;
    // A misbehaving server sends the walk back to `a`: the stream ends instead of looping.
    Mock::given(method("GET"))
        .and(path("/v2/trades"))
        .and(query_param("cursor", "b"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::trade("0x3")], Some("a"))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let hashes: Vec<String> = common::polymarket(&server)
        .data()
        .list_trades()
        .into_stream()
        .map_ok(|t| t.transaction_hash)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(hashes, vec!["0x1", "0x2", "0x3"]);
}

#[tokio::test]
async fn list_combo_positions_resends_user_with_cursor() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/positions/combos"))
        .and(query_param("user", WALLET))
        .and(query_param("status", "RESOLVED_WIN,RESOLVED_LOSS"))
        .and(query_param("sort_by", "UPDATED"))
        .and(query_param("sort_direction", "ASC"))
        .and(query_param("updated_after", "1787133600"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![fixtures::combo_position("a")],
            Some("next"),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/positions/combos"))
        .and(query_param("user", WALLET))
        .and(query_param("cursor", "next"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::combo_position("b")], None)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let positions: Vec<_> = common::polymarket(&server)
        .data()
        .list_combo_positions(WALLET)
        .statuses([
            ComboPositionStatus::ResolvedWin,
            ComboPositionStatus::ResolvedLoss,
        ])
        .sort_by(ComboPositionSortBy::Updated)
        .sort_direction(SortDirection::Asc)
        .updated_after(DateTime::from_timestamp(1_787_133_600, 0).unwrap())
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(positions.len(), 2);
    assert_eq!(positions[1].combo_position_id, TokenId::from("b"));
    assert_eq!(positions[0].status, ComboPositionStatus::Redeemable);
    assert_eq!(
        positions[0].legs[0]
            .market
            .outcomes
            .as_deref()
            .map(<[String]>::len),
        Some(2)
    );
    assert_eq!(positions[0].combo_condition_id, COMBO_CONDITION);
    assert!(positions[0].first_entry_at.is_some());
    assert_eq!(
        positions[0].first_entry_at_micros,
        positions[0].first_entry_at
    );
}

#[tokio::test]
async fn list_combo_positions_validates_before_sending() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let data = common::polymarket(&server).data().clone();

    let err = data
        .list_combo_positions(WALLET)
        .updated_after(DateTime::from_timestamp(-60, 0).unwrap())
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "updated_after"));
    let err = data
        .list_combo_positions(WALLET)
        .updated_after(DateTime::from_timestamp(1_787_133_600, 0).unwrap())
        .updated_before(DateTime::from_timestamp(1_787_133_599, 0).unwrap())
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "updated_before"));
    let err = data
        .list_combo_positions(WALLET)
        .statuses([
            ComboPositionStatus::Redeemable,
            ComboPositionStatus::Partial,
        ])
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "status"));
    let err = data
        .list_combo_positions(WALLET)
        .conditions(["0x03aa"])
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "condition"));
}

#[tokio::test]
async fn get_portfolio_value_sends_conditions_csv() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/value"))
        .and(query_param("user", WALLET))
        .and(query_param(
            "condition",
            format!("{CONDITION},{CONDITION_2}"),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::envelope(
            json!({"proxy_wallet": WALLET, "value": 1234.5678}),
        )))
        .expect(1)
        .mount(&server)
        .await;

    let value = common::polymarket(&server)
        .data()
        .get_portfolio_value(WALLET)
        .conditions([CONDITION, CONDITION_2, CONDITION])
        .send()
        .await
        .unwrap();
    assert_eq!(value.proxy_wallet, WALLET);
    assert_eq!(value.value, Decimal::new(12_345_678, 4));
}

#[tokio::test]
async fn get_approvals_decodes_snapshot() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/approvals"))
        .and(query_param("user", WALLET))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!({
                "address": WALLET,
                "chain_id": 137,
                "checked_at": "2026-10-01T12:00:00Z",
                "contracts": [{
                    "id": "usdc-ctf-exchange",
                    "feature": "trading",
                    "token": WALLET_2,
                    "spender": "0x0000000000000000000000000000000000000002",
                    "standard": "ERC20",
                    "approved": true,
                    "amount": "1000000"
                }]
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let approvals = common::polymarket(&server)
        .data()
        .get_approvals(WALLET)
        .await
        .unwrap();
    assert_eq!(approvals.chain_id, 137);
    let contract = &approvals.contracts[0];
    assert_eq!(contract.standard, TokenStandard::Erc20);
    assert_eq!(contract.amount.as_deref(), Some("1000000"));
    assert!(!contract.is_unlimited());
}

#[tokio::test]
async fn wallet_routes_documenting_an_evm_address_validate_it() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let data = common::polymarket(&server).data().clone();

    for user in ["", "0x1234", "983eedfbd75803602e4a6e6ea9aab6dc6b9c6748"] {
        let err = data.get_approvals(user).await.unwrap_err();
        assert!(
            matches!(&err, Error::Validation(v) if v.parameter() == "user"),
            "{user:?}: {err}"
        );
        let err = data.get_user_stats(user).await.unwrap_err();
        assert!(
            matches!(&err, Error::Validation(v) if v.parameter() == "user"),
            "{user:?}: {err}"
        );
    }
    // Elsewhere only an empty wallet is rejected.
    for err in [
        data.get_portfolio_value(" ").send().await.unwrap_err(),
        data.get_user_pnl("").send().await.unwrap_err(),
        data.get_user_volume("").send().await.unwrap_err(),
    ] {
        assert!(matches!(&err, Error::Validation(v) if v.parameter() == "user"));
    }
    let err = data
        .get_user_volume(WALLET)
        .start(DateTime::from_timestamp(-86_400, 0).unwrap())
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "start"));
}

#[tokio::test]
async fn get_user_pnl_sends_interval_and_fidelity() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/user-pnl"))
        .and(query_param("user", WALLET))
        .and(query_param("interval", "1w"))
        .and(query_param("fidelity", "12h"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!({
                "proxy_wallet": WALLET,
                "interval": "1w",
                "fidelity": "12h",
                "source_fidelity": "1d",
                "points": [fixtures::pnl_point()]
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let series = common::polymarket(&server)
        .data()
        .get_user_pnl(WALLET)
        .interval(PnlInterval::OneWeek)
        .fidelity(PnlFidelity::TwelveHours)
        .send()
        .await
        .unwrap();
    assert_eq!(series.interval, PnlInterval::OneWeek);
    assert_eq!(series.source_fidelity, PnlFidelity::OneDay);
    let point = &series.points[0];
    assert_eq!(point.realized_pnl, Decimal::from(10));
    // `null` and missing amounts are "unavailable", never zero.
    assert_eq!(point.unrealized_pnl, None);
    assert_eq!(point.withdrawals, None);
    assert_eq!(point.deposits, Some(Decimal::from(100)));
}

#[tokio::test]
async fn get_user_stats_maps_null_data_to_none() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/user-stats"))
        .and(query_param("user", WALLET_2))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": null})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/user-stats"))
        .and(query_param("user", WALLET))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!({
                "proxy_wallet": WALLET,
                "trades": 0,
                "biggest_win": 0,
                "views": 0,
                "join_date": 1700000000,
                "all_time_pnl": null
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let data = common::polymarket(&server).data().clone();
    assert_eq!(data.get_user_stats(WALLET_2).await.unwrap(), None);
    let stats = data.get_user_stats(WALLET).await.unwrap().unwrap();
    assert_eq!(stats.trades, 0);
    assert_eq!(stats.join_date.map(|d| d.timestamp()), Some(1_700_000_000));
    assert_eq!(stats.all_time_pnl, None);
}

#[tokio::test]
async fn get_user_volume_sends_epoch_seconds() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/user-volume"))
        .and(query_param("user", WALLET))
        .and(query_param("start", "1787097600"))
        .and(query_param("end", "1787183999"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!({
                "volume": 200,
                "volume_usdc": 104.06,
                "trade_count": 2
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let volume = common::polymarket(&server)
        .data()
        .get_user_volume(WALLET)
        .start(DateTime::from_timestamp(1_787_097_600, 0).unwrap())
        .end(DateTime::from_timestamp(1_787_183_999, 0).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(volume.trade_count, 2);
    assert_eq!(volume.volume_usdc.to_string(), "104.06");
}
