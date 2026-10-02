//! Response fixtures.
//!
//! The Data API v2 spec (`docs/polymarket/specs/data-v2-openapi.json`) has no response examples; the
//! only documented example is the trimmed positions page in
//! `docs/polymarket/api-reference/data-api/overview.md`. Rows here reuse its values and fill every
//! other required field of the corresponding `components/schemas/*` schema with a value of
//! the documented type.

use serde_json::{Value, json};

pub const WALLET: &str = "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748";
/// A second wallet (`0x` + 40 hex digits).
pub const WALLET_2: &str = "0x0000000000000000000000000000000000000001";
pub const CONDITION: &str = "0xd9b06e2fd9ddb7ab61c9e3d5d8e074c555802478bbf75145804ff709a4246f79";
/// A second condition id (`0x` + 64 hex digits, as the overview documents).
pub const CONDITION_2: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";
/// A combo condition id as served live: `0x` plus 62 hex digits (captured from
/// `GET /v2/positions/combos`, 2026-10-02).
pub const COMBO_CONDITION: &str =
    "0x033c72a79df1dfd46683b15b5c0ce78ef50000000000000000000000000000";
pub const TOKEN: &str =
    "31974447302330162086995746309500877260929998201718217388109724292047967921664";

/// `{ data, pagination }` with `has_more` derived from the cursor.
pub fn page(items: Vec<Value>, next_cursor: Option<&str>) -> Value {
    page_with(items, next_cursor, next_cursor.is_some())
}

/// `{ data, pagination }` with an explicit `has_more`.
pub fn page_with(items: Vec<Value>, next_cursor: Option<&str>, has_more: bool) -> Value {
    let limit = items.len();
    json!({
        "data": items,
        "pagination": {
            "limit": limit,
            "offset": 0,
            "has_more": has_more,
            "next_cursor": next_cursor,
        }
    })
}

/// `{ data }`.
pub fn envelope(data: Value) -> Value {
    json!({ "data": data })
}

/// `components/schemas/Position`, values from the overview example.
pub fn position(token_id: &str) -> Value {
    json!({
        "proxy_wallet": WALLET,
        "condition_id": CONDITION,
        "token_id": token_id,
        "outcome": "Yes",
        "title": "Will ŠK Slovan Bratislava win on 2026-08-19?",
        "status": "REDEEMABLE",
        "current_size": 86780.64,
        "avg_price": 0.5203,
        "entry_cost_usdc": 45159.4653,
        "current_value": 0.0,
        "realized_pnl": -1082.5533,
        "unrealized_pnl": -45159.4653,
        "entry_fees_usdc": 0.0,
        "total_cost_usdc": 45159.4653,
        "current_price": 0.0,
        "total_size": 86780.64,
        "total_pnl": -46242.0186,
        "percent_pnl": -100.0,
        "percent_realized_pnl": -100.0,
        "redeemable": true,
        "mergeable": false,
        "negative_risk": false,
        "archived": false,
        "slug": "slovan-bratislava",
        "icon": "",
        "event_id": "12345",
        "event_slug": "slovan-bratislava-2026-08-19",
        "outcome_index": 0,
        "opposite_outcome": "No",
        "opposite_token_id": "1",
        "end_date": "2026-08-19",
        "last_event_at": 1787097600,
        "first_entry_at": 1787011200,
        "name": "",
        "profile_image": "",
        "verified": false
    })
}

/// `components/schemas/ComboLeg`.
pub fn combo_leg() -> Value {
    json!({
        "leg_index": 0,
        "leg_position_id": "456",
        "leg_condition_id": CONDITION,
        "leg_outcome_index": 0,
        "leg_outcome_label": "Yes",
        "leg_status": "RESOLVED_WIN",
        "leg_current_price": 1,
        "leg_resolved_at": "2026-08-19T10:00:00Z",
        "market": {
            "market_id": "789",
            "slug": "m",
            "title": "M",
            "outcome": "Yes",
            "image_url": "",
            "icon_url": "",
            "category": "sports",
            "subcategory": "soccer",
            "tags": [],
            "end_date": "2026-08-19T10:00:00Z",
            "event": {"event_id": "42", "event_slug": "e", "event_title": "E", "event_image": ""},
            "question": "Will it?",
            "group_item_title": "",
            "outcomes": ["Yes", "No"],
            "sports_market_type": "",
            "line": null
        }
    })
}

/// `components/schemas/ComboPosition`.
pub fn combo_position(id: &str) -> Value {
    json!({
        "combo_condition_id": COMBO_CONDITION,
        "outcome_index": 0,
        "outcome_label": "Yes",
        "combo_position_id": id,
        "proxy_wallet": WALLET,
        "current_size": 10,
        "entry_avg_price_usdc": 0.25,
        "entry_cost_usdc": 2.5,
        "gross_entry_cost_usdc": 2.52,
        "entry_fees_usdc": 0.02,
        "realized_payout_usdc": 0,
        "status": "REDEEMABLE",
        "redeemable": true,
        "first_entry_at": "2026-08-19T10:00:00Z",
        "first_entry_at_micros": 1787133600000000_i64,
        "legs_total": 1,
        "legs_resolved": 1,
        "legs_pending": 0,
        "legs": [combo_leg()],
        "resolved_at": "2026-08-19T10:00:00Z",
        "updated_at": "2026-08-19T10:00:00Z",
        "updated_at_micros": 1787133600000000_i64
    })
}

/// `components/schemas/UserPnlPoint`, in the live null pattern: `unrealized_pnl` and
/// `position_pnl` present, `deposits`, `withdrawals` and `cashflow_net` `null` (see the
/// capture in `live.rs`).
pub fn pnl_point() -> Value {
    json!({
        "timestamp": 1787133600,
        "source_block": 75000000,
        "realized_market_pnl": 10.5,
        "realized_lp_pnl": 0,
        "realized_combo_pnl": -0.5,
        "realized_pnl": 10,
        "volume": 100,
        "volume_usdc": 52.03,
        "trade_count": 3,
        "unrealized_pnl": -88.5,
        "position_pnl": -2271,
        "fees": -0.25,
        "deposits": null,
        "withdrawals": null,
        "cashflow_net": null
    })
}

/// `components/schemas/Trade`.
pub fn trade(hash: &str) -> Value {
    json!({
        "proxy_wallet": WALLET,
        "side": "BUY",
        "token_id": TOKEN,
        "condition_id": CONDITION,
        "size": 100,
        "price": 0.5203,
        "timestamp": 1787133600,
        "title": "Will ŠK Slovan Bratislava win on 2026-08-19?",
        "slug": "s",
        "icon": "",
        "event_slug": "e",
        "outcome": "Yes",
        "outcome_index": 0,
        "name": "",
        "pseudonym": "Calm-Owl",
        "bio": "",
        "profile_image": "",
        "profile_image_optimized": "",
        "transaction_hash": hash
    })
}

/// `components/schemas/Activity` of a non-combo row: `is_combo` is omitted ("omitted from
/// non-combo rows").
pub fn activity(kind: &str, side: &str) -> Value {
    json!({
        "proxy_wallet": WALLET,
        "timestamp": 1787133600,
        "condition_id": CONDITION,
        "type": kind,
        "size": 100,
        "usdc_size": 52.03,
        "transaction_hash": "0xfeed",
        "price": 0.5203,
        "token_id": TOKEN,
        "side": side,
        "outcome_index": 0,
        "title": "",
        "slug": "",
        "icon": "",
        "event_slug": "",
        "outcome": "Yes",
        "name": "",
        "pseudonym": "",
        "bio": "",
        "profile_image": "",
        "profile_image_optimized": ""
    })
}

/// `components/schemas/Activity` of a combo trade row: the only rows that carry
/// `is_combo`.
pub fn combo_trade_activity(side: &str) -> Value {
    let mut row = activity("TRADE", side);
    row["is_combo"] = json!(true);
    row
}

/// `components/schemas/ComboActivity`.
pub fn combo_activity() -> Value {
    json!({
        "id": "0xfeed-3",
        "type": "SPLIT",
        "proxy_wallet": WALLET,
        "combo_condition_id": COMBO_CONDITION,
        "combo_position_id": "123",
        "block_number": 75000000,
        "timestamp": 1787133600,
        "transaction_hash": "0xfeed",
        "legs": [combo_leg()],
        "amount_usdc": 25,
        "payout_usdc": null
    })
}

/// `components/schemas/MetaHolder` with one `Holder`.
pub fn holder_group(token_id: &str, wallet: &str) -> Value {
    json!({
        "token_id": token_id,
        "holders": [{
            "proxy_wallet": wallet,
            "bio": "",
            "token_id": token_id,
            "pseudonym": "Calm-Owl",
            "amount": 1500.25,
            "display_username_public": true,
            "outcome_index": 0,
            "name": "",
            "profile_image": "",
            "profile_image_optimized": "",
            "verified": false
        }]
    })
}

/// `components/schemas/BuilderStanding`.
pub fn builder_standing(rank: u64, code: &str) -> Value {
    json!({
        "rank": rank,
        "builder_name": code,
        "builder_code": code,
        "profile_image": "",
        "verified": true,
        "volume": 123456.5,
        "active_users": 42
    })
}

/// `components/schemas/PricePoint`.
pub fn price_point(timestamp: i64, price: f64, resolution_seconds: i64) -> Value {
    json!({"timestamp": timestamp, "price": price, "resolution_seconds": resolution_seconds})
}

/// `components/schemas/BiggestWinner`. A `combo` row carries the combo condition, no Gamma
/// event (`event_id` `0`, empty `event_slug`) and a `' / '`-joined title of its legs; a
/// `market` row a regular condition and its event.
pub fn biggest_winner(win_rank: u32, kind: &str) -> Value {
    let combo = kind == "combo";
    json!({
        "win_rank": win_rank,
        "kind": kind,
        "user_id": WALLET,
        "pnl": 900,
        "initial_value": 100,
        "final_value": 1000,
        "resolved_at": 1787133600,
        "condition_id": if combo { COMBO_CONDITION } else { CONDITION },
        "position_id": "123",
        "event_id": if combo { 0 } else { 42 },
        "event_slug": if combo { "" } else { "slovan-bratislava-2026-08-19" },
        "event_title": if combo { "Will A win? / Will B win?" } else { "Slovan Bratislava" },
        "user_name": "",
        "profile_image": ""
    })
}

/// `components/schemas/LeaderboardEntry`.
pub fn leaderboard_entry(rank: u32) -> Value {
    json!({
        "rank": rank,
        "user_id": WALLET,
        "pnl": 12345.67,
        "volume": 99999.5,
        "user_name": "whale",
        "profile_image": "",
        "x_username": "",
        "verified": true
    })
}
