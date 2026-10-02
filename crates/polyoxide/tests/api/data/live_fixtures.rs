//! Decode tests for rows captured from the live Data API v2 (2026-10-02), trimmed.
//!
//! The documented examples never show these shapes: exponent-notation decimals, the
//! undocumented `first_entry_at`, non-trade activity rows with `""` ids, the live null
//! pattern of the PnL points, and combo rows with 62-digit condition ids. The capture
//! routes are listed in `live_fixtures.json` (`_sources`); the pinning live tests are in
//! `tests/live/data.rs`, and the deviations in `SPEC_DEVIATIONS.md`.

use polyoxide::{
    Decimal,
    data::{
        Activity, ActivityType, ComboActivity, ComboLegStatus, ComboPosition, ComboPositionStatus,
        Position, PositionStatus, UserPnlPoint, UserPnlSeries,
    },
};
use serde_json::Value;

fn fixtures() -> Value {
    serde_json::from_str(include_str!("live_fixtures.json")).expect("fixture file is JSON")
}

fn fixture<T: serde::de::DeserializeOwned>(key: &str) -> T {
    let mut all = fixtures();
    let value = all[key].take();
    serde_json::from_value(value).unwrap_or_else(|e| panic!("{key}: {e}"))
}

/// A CLOSED position: `entry_fees_usdc` and `total_cost_usdc` are served in exponent
/// notation (`9e-6`), and the row carries the undocumented `first_entry_at`.
#[test]
fn closed_position_with_exponent_decimals_and_first_entry_at() {
    let raw = fixtures()["_raw_position_closed_exponent"]
        .as_str()
        .expect("raw text")
        .to_owned();
    assert!(
        raw.contains(r#""entry_fees_usdc":9e-6"#),
        "exponent kept in the raw text"
    );
    let position: Position = serde_json::from_str(&raw).unwrap();
    assert_eq!(position.status, PositionStatus::Closed);
    assert_eq!(position.entry_fees_usdc, Decimal::new(9, 6));
    assert_eq!(position.total_cost_usdc, Decimal::new(9, 6));
    assert_eq!(
        position.first_entry_at.map(|t| t.timestamp()),
        Some(1_783_275_277)
    );
    assert_eq!(position.last_event_at.timestamp(), 1_783_389_079);

    // The same row through serde_json::Value (which rewrites the exponent) decodes alike.
    let again: Position = fixture("position_closed_exponent");
    assert_eq!(again, position);
}

/// A position with no native state: `last_event_at` and `first_entry_at` are both `0`.
#[test]
fn position_without_native_state_has_no_first_entry() {
    let position: Position = fixture("position_no_native_state");
    assert_eq!(position.first_entry_at, None);
    assert_eq!(position.last_event_at.timestamp(), 0);
    assert_eq!(position.status, PositionStatus::Redeemable);
    // `None` serializes back as `0`, like the wire.
    let value = serde_json::to_value(&position).unwrap();
    assert_eq!(value["first_entry_at"], 0);
    assert_eq!(serde_json::from_value::<Position>(value).unwrap(), position);
}

/// Rebate, yield, referral and deposit rows are non-trade rows: `condition_id`,
/// `token_id` and `side` are `""`. A merge row has a condition but no token.
#[test]
fn non_trade_activity_rows_decode_with_empty_ids() {
    let rows: std::collections::BTreeMap<String, Activity> = fixture("activity_rows");
    for (wire, kind) in [
        ("TAKER_REBATE", ActivityType::TakerRebate),
        ("MAKER_REBATE", ActivityType::MakerRebate),
        ("YIELD", ActivityType::Yield),
        ("REFERRAL_REWARD", ActivityType::ReferralReward),
        ("DEPOSIT", ActivityType::Deposit),
    ] {
        let row = &rows[wire];
        assert_eq!(row.activity_type, kind, "{wire}");
        assert!(!row.activity_type.is_unknown(), "{wire}");
        assert_eq!(row.condition_id, None, "{wire}");
        assert_eq!(row.token_id, None, "{wire}");
        assert_eq!(row.side, None, "{wire}");
        assert_eq!(row.size, row.usdc_size, "{wire}");
        // Serialized back as on the wire.
        let value = serde_json::to_value(row).unwrap();
        assert_eq!(value["condition_id"], "");
        assert_eq!(value["token_id"], "");
        assert_eq!(value["side"], "");
    }
    assert_eq!(rows["TAKER_REBATE"].usdc_size, Decimal::new(5_581_579, 4));

    let merge = &rows["MERGE"];
    assert_eq!(merge.activity_type, ActivityType::Merge);
    assert!(merge.condition_id.is_some());
    assert_eq!(merge.token_id, None);

    // A conversion likewise has a condition and no token.
    let conversion = &rows["CONVERSION"];
    assert_eq!(conversion.activity_type, ActivityType::Conversion);
    assert!(conversion.condition_id.is_some());
    assert_eq!(conversion.token_id, None);
}

/// A user-pnl point: only `deposits`, `withdrawals` and `cashflow_net` are `null`;
/// `unrealized_pnl` and `position_pnl` are present.
#[test]
fn pnl_point_has_the_live_null_pattern() {
    let point: UserPnlPoint = fixture("pnl_point");
    assert!(point.unrealized_pnl.is_some());
    assert!(point.position_pnl.is_some());
    assert!(point.trade_pnl.is_some());
    assert!(point.wallet_income.is_some());
    assert_eq!(point.deposits, None);
    assert_eq!(point.withdrawals, None);
    assert_eq!(point.cashflow_net, None);
    assert_eq!(point.realized_pnl, point.realized_market_pnl);

    let mut series = fixtures()["pnl_series_envelope"].take();
    series["points"] = Value::Array(vec![fixtures()["pnl_point"].take()]);
    let series: UserPnlSeries = serde_json::from_value(series).unwrap();
    assert_eq!(series.points, vec![point]);
}

/// A combo position: the combo and leg condition ids are `0x` plus 62 hex digits, and the
/// leg `end_date` is a bare date.
#[test]
fn combo_position_with_62_digit_ids() {
    let position: ComboPosition = fixture("combo_position");
    assert_eq!(position.status, ComboPositionStatus::Open);
    assert_eq!(position.combo_condition_id.as_str().len(), 2 + 62);
    assert!(position.combo_condition_id.as_str().starts_with("0x03"));
    let leg = &position.legs[0];
    assert_eq!(leg.leg_condition_id.as_str().len(), 2 + 62);
    assert_eq!(leg.leg_status, ComboLegStatus::Open);
    let end = leg.market.end_date.expect("a date");
    assert_eq!(end.to_rfc3339(), "2026-10-09T00:00:00+00:00");
    assert!(position.first_entry_at.is_some());
}

/// A combo activity row: the leg condition ids are regular bytes32 here (unlike on
/// `/v2/positions/combos`), and the leg `end_date` is RFC 3339.
#[test]
fn combo_activity_with_bytes32_leg_ids() {
    let row: ComboActivity = fixture("combo_activity");
    assert_eq!(row.combo_condition_id.as_str().len(), 2 + 62);
    let leg = &row.legs[0];
    assert_eq!(leg.leg_condition_id.as_str().len(), 2 + 64);
    let end = leg.market.end_date.expect("a timestamp");
    assert_eq!(end.to_rfc3339(), "2026-10-09T03:00:00+00:00");
    assert!(row.payout_usdc.is_some());
}

/// The second page of a cursor walk (`GET /v2/prices-history`, 2026-10-02, trimmed): the
/// running `offset` is greater than zero (it numbers rows across pages and is cosmetic),
/// and the points are bucket-aligned with `resolution_seconds` echoing the bucket.
#[test]
fn second_page_has_a_running_offset() {
    let json = r#"{
        "data": [
            {"timestamp": 1790848500, "price": 0.715, "resolution_seconds": 60},
            {"timestamp": 1790848560, "price": 0.715, "resolution_seconds": 60},
            {"timestamp": 1790848680, "price": 0.715, "resolution_seconds": 60}
        ],
        "pagination": {"limit": 3, "offset": 3, "has_more": true, "next_cursor": "eyJkYXRhIjp7InR5cGUiOiJwcmljZXNfaGlzdG9yeSJ9fQ"}
    }"#;
    let page: polyoxide::data::Page<polyoxide::data::PricePoint> =
        serde_json::from_str(json).unwrap();
    assert_eq!(page.pagination.offset, 3);
    assert_eq!(page.items().len(), 3);
    assert_eq!(
        page.next_cursor(),
        Some("eyJkYXRhIjp7InR5cGUiOiJwcmljZXNfaGlzdG9yeSJ9fQ")
    );
    assert_eq!(page.items()[0].resolution_seconds, 60);
}
