//! Feed endpoints: `/v2/trades`, `/v2/activity`, `/v2/activity/combos`.

use chrono::DateTime;
use futures_util::TryStreamExt as _;
use polyoxide::{
    Decimal,
    data::{
        ActivitySide, ActivitySortBy, ActivityType, ComboActivityType, ComboLegStatus, FilterType,
        SortDirection,
    },
    types::Side,
};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use super::{
    fixtures::{self, COMBO_CONDITION, CONDITION, WALLET},
    received_queries,
};
use crate::common;

#[tokio::test]
async fn list_trades_sends_filters_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/trades"))
        .and(query_param("condition", CONDITION))
        .and(query_param("side", "BUY"))
        .and(query_param("taker_only", "false"))
        .and(query_param("filter_type", "CASH"))
        .and(query_param("filter_amount", "10.5"))
        .and(query_param("event_id", "1,2"))
        .and(query_param_is_missing("user"))
        .and(query_param_is_missing("offset"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::trade("0x1")], None)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .data()
        .list_trades()
        .conditions([CONDITION])
        .event_ids(["1", "2"])
        .side(Side::Buy)
        .taker_only(false)
        .filter_type(FilterType::Cash)
        .filter_amount(Decimal::new(105, 1))
        .send()
        .await
        .unwrap();
    let trade = &page.items()[0];
    assert_eq!(trade.side, Side::Buy);
    assert_eq!(trade.price.to_string(), "0.5203");
    assert_eq!(trade.size, Decimal::from(100));
    assert_eq!(trade.timestamp.timestamp(), 1_787_133_600);
    assert_eq!(page.next_cursor(), None);
    assert!(!page.has_more());
}

#[tokio::test]
async fn list_trades_stream_resends_identical_filters() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/trades"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![fixtures::trade("0x1"), fixtures::trade("0x2")],
            Some("seek-2"),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/trades"))
        .and(query_param("cursor", "seek-2"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::trade("0x3")], None)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let hashes: Vec<String> = common::polymarket(&server)
        .data()
        .list_trades()
        .user(WALLET)
        .start(DateTime::from_timestamp(1, 0).unwrap())
        .limit(2)
        .into_stream()
        .map_ok(|t| t.transaction_hash)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(hashes, vec!["0x1", "0x2", "0x3"]);
    assert_eq!(
        received_queries(&server).await,
        vec![
            format!("user={WALLET}&limit=2&start=1"),
            format!("user={WALLET}&limit=2&cursor=seek-2&start=1"),
        ]
    );
}

#[tokio::test]
async fn list_activity_sends_types_and_decodes_sides() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/activity"))
        .and(query_param("user", WALLET))
        .and(query_param("type", "TRADE,TIP"))
        .and(query_param("sort_by", "TIMESTAMP"))
        .and(query_param("sort_direction", "ASC"))
        .and(query_param("exclude_deposits_withdrawals", "false"))
        .and(query_param("end", "1787133600"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![
                fixtures::combo_trade_activity("SELL"),
                fixtures::activity("TIP", "IN"),
                fixtures::activity("CONVERSION", ""),
                fixtures::activity("SOMETHING_NEW", ""),
            ],
            None,
        )))
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .data()
        .list_activity(WALLET)
        .types([ActivityType::Trade, ActivityType::Tip])
        .sort_by(ActivitySortBy::Timestamp)
        .sort_direction(SortDirection::Asc)
        .exclude_deposits_withdrawals(false)
        .end(DateTime::from_timestamp(1_787_133_600, 0).unwrap())
        .send()
        .await
        .unwrap();
    let rows = page.items();
    assert_eq!(rows[0].activity_type, ActivityType::Trade);
    assert_eq!(rows[0].side, Some(ActivitySide::Sell));
    assert_eq!(rows[0].is_combo, Some(true));
    assert_eq!(rows[1].is_combo, None);
    assert_eq!(rows[1].activity_type, ActivityType::Tip);
    assert_eq!(rows[1].side, Some(ActivitySide::In));
    assert_eq!(rows[2].side, None);
    assert_eq!(rows[2].is_combo, None);
    assert_eq!(
        rows[3].activity_type,
        ActivityType::Unknown("SOMETHING_NEW".to_owned())
    );
}

#[tokio::test]
async fn list_combo_activity_decodes_legs() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/activity/combos"))
        .and(query_param("user", WALLET))
        .and(query_param("condition", COMBO_CONDITION))
        .and(query_param("limit", "10"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::combo_activity()], None)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .data()
        .list_combo_activity(WALLET)
        .conditions([COMBO_CONDITION])
        .limit(10)
        .send()
        .await
        .unwrap();
    let row = &page.items()[0];
    assert_eq!(row.activity_type, ComboActivityType::Split);
    assert_eq!(row.amount_usdc, Some(Decimal::from(25)));
    assert_eq!(row.payout_usdc, None);
    let leg = &row.legs[0];
    assert_eq!(leg.leg_status, ComboLegStatus::ResolvedWin);
    assert!(leg.leg_resolved_at.is_some());
    assert_eq!(leg.market.line, None);
    assert_eq!(leg.market.question.as_deref(), Some("Will it?"));
}

#[tokio::test]
async fn list_activity_stream_resends_filters_and_sort_direction() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/activity"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![
                fixtures::activity("SPLIT", ""),
                fixtures::activity("MERGE", ""),
            ],
            Some("act-2"),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/activity"))
        .and(query_param("cursor", "act-2"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::activity("REDEEM", "")], None)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let kinds: Vec<ActivityType> = common::polymarket(&server)
        .data()
        .list_activity(WALLET)
        .types([
            ActivityType::Split,
            ActivityType::Merge,
            ActivityType::Redeem,
        ])
        .conditions([CONDITION, CONDITION])
        .sort_direction(SortDirection::Asc)
        .full_history()
        .limit(2)
        .into_stream()
        .map_ok(|a| a.activity_type)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(
        kinds,
        vec![
            ActivityType::Split,
            ActivityType::Merge,
            ActivityType::Redeem
        ]
    );
    let common = format!("type=SPLIT%2CMERGE%2CREDEEM&condition={CONDITION}");
    assert_eq!(
        received_queries(&server).await,
        vec![
            format!("user={WALLET}&limit=2&{common}&start=1&sort_direction=ASC"),
            format!("user={WALLET}&limit=2&cursor=act-2&{common}&start=1&sort_direction=ASC"),
        ]
    );
}

#[tokio::test]
async fn list_combo_activity_stream_resends_user_and_conditions() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/activity/combos"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![fixtures::combo_activity()],
            Some("combo-2"),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/activity/combos"))
        .and(query_param("cursor", "combo-2"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::combo_activity()], None)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let rows: Vec<_> = common::polymarket(&server)
        .data()
        .list_combo_activity(WALLET)
        .conditions([COMBO_CONDITION])
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        received_queries(&server).await,
        vec![
            format!("user={WALLET}&condition={COMBO_CONDITION}"),
            format!("user={WALLET}&cursor=combo-2&condition={COMBO_CONDITION}"),
        ]
    );
}
