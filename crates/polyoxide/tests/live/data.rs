//! Data API v2 live tests.
//!
//! Every Data route returns a `{ data, pagination? }` envelope. Paged routes decode as
//! [`Page<T>`] (which is `Serialize`, so drift is checked on the whole envelope); the other
//! routes are decoded from `raw["data"]` with [`check_data`].

use std::collections::HashSet;

use futures_util::{StreamExt as _, TryStreamExt as _};
use polyoxide::{
    Decimal, Error,
    chrono::{DateTime, Duration, Utc},
    data::{
        Activity, ActivitySortBy, ActivityType, Approvals, BiggestWinner, BuilderStanding,
        BuilderVolumePoint, ComboActivity, ComboPosition, ComboPositionSortBy, ComboPositionStatus,
        ErrorCode, FilterType, HolderGroup, LeaderboardEntry, LeaderboardSortBy,
        LeaderboardUserEntry, OpenInterest, Page, PnlFidelity, PnlInterval, PortfolioValue,
        Position, PositionSortBy, PositionStatus, PriceHistoryInterval, PricePoint, Resolution,
        ResolutionSelector, ServiceStatus, SortDirection, TimePeriod, Trade, UserPnlSeries,
        UserStats, UserVolume,
    },
    types::Side,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tokio::sync::OnceCell;

use crate::common::{DATA, GAMMA, Raw, check, get, pm, sample};

// ---------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------

/// Raw `GET /v2{path}`.
async fn raw(path: &str, query: &[(&str, &str)]) -> Raw {
    get(DATA, &format!("/v2{path}"), query).await
}

/// Decodes the `data` member of a raw envelope as `T` and reports drift.
fn check_data<T>(label: &str, raw: &Raw) -> T
where
    T: DeserializeOwned + Serialize + std::fmt::Debug,
{
    let data = raw.json["data"].clone();
    let inner = Raw {
        text: data.to_string(),
        json: data,
    };
    check::<T>(label, &inner)
}

/// A raw non-2xx response: status, `x-trace-id` header and parsed body.
struct RawError {
    status: u16,
    trace_header: Option<String>,
    body: Value,
}

async fn raw_error(path: &str, query: &[(&str, &str)]) -> RawError {
    let response = reqwest::Client::builder()
        .user_agent("polyoxide-live-tests")
        .build()
        .expect("client builds")
        .get(format!("{DATA}/v2{path}"))
        .query(query)
        .send()
        .await
        .expect("request sends");
    let status = response.status().as_u16();
    let trace_header = response
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let body = response.json().await.expect("error body is JSON");
    RawError {
        status,
        trace_header,
        body,
    }
}

/// Identifiers beyond the shared [`sample`], discovered once per run from raw responses.
#[derive(Debug)]
struct Extra {
    /// Condition id of a resolved market.
    resolved_condition: String,
    /// UMA question id of that market.
    resolved_question: Option<String>,
    /// Gamma event id of that market.
    resolved_event: Option<String>,
    /// A wallet with a populated `/v2/user-stats` (top of the all-time board).
    stats_user: String,
    /// A wallet with combo positions, if one was found.
    combo_position_user: Option<String>,
    /// A wallet with combo activity, if one was found.
    combo_activity_user: Option<String>,
    /// A combo condition id of `combo_position_user`.
    combo_condition: Option<String>,
}

static EXTRA: OnceCell<Extra> = OnceCell::const_new();
static CANDIDATES: OnceCell<Vec<String>> = OnceCell::const_new();

/// Active wallets: the distinct top-50 by volume of the day, week and all-time boards.
async fn candidates() -> &'static [String] {
    CANDIDATES
        .get_or_init(|| async {
            let mut wallets: Vec<String> = Vec::new();
            for period in ["day", "week", "all"] {
                let board = raw(
                    "/leaderboard",
                    &[
                        ("time_period", period),
                        ("sort_by", "VOLUME"),
                        ("limit", "50"),
                    ],
                )
                .await
                .json;
                wallets.extend(
                    board["data"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|r| r["user_id"].as_str().map(str::to_owned)),
                );
            }
            let mut seen = HashSet::new();
            wallets.retain(|w| seen.insert(w.clone()));
            wallets
        })
        .await
}

async fn extra() -> &'static Extra {
    EXTRA
        .get_or_init(|| async {
            let markets = get(
                GAMMA,
                "/markets",
                &[
                    ("closed", "true"),
                    ("limit", "5"),
                    ("order", "volume"),
                    ("ascending", "false"),
                ],
            )
            .await
            .json;
            let market = markets
                .as_array()
                .and_then(|m| m.iter().find(|m| m["conditionId"].is_string()))
                .expect("a closed market");
            let text = |v: &Value| v.as_str().map(str::to_owned);

            let all_time = raw("/leaderboard", &[("time_period", "all"), ("limit", "3")])
                .await
                .json;
            let stats_user = all_time["data"][0]["user_id"]
                .as_str()
                .expect("all-time leader")
                .to_owned();

            // Combo users are rare: scan wallets from the boards for one.
            let candidates = candidates().await;
            let mut combo_position_user = None;
            let mut combo_activity_user = None;
            let mut combo_condition = None;
            for user in candidates {
                if combo_position_user.is_none() {
                    let page = raw("/positions/combos", &[("user", user), ("limit", "1")]).await;
                    if let Some(first) = page.json["data"].get(0) {
                        combo_condition = text(&first["combo_condition_id"]);
                        combo_position_user = Some(user.clone());
                    }
                }
                if combo_activity_user.is_none() {
                    let page = raw("/activity/combos", &[("user", user), ("limit", "1")]).await;
                    if page.json["data"].get(0).is_some() {
                        combo_activity_user = Some(user.clone());
                    }
                }
                if combo_position_user.is_some() && combo_activity_user.is_some() {
                    break;
                }
            }

            Extra {
                resolved_condition: text(&market["conditionId"]).expect("condition id"),
                resolved_question: text(&market["questionID"]),
                resolved_event: market["events"].get(0).and_then(|e| text(&e["id"])),
                stats_user,
                combo_position_user,
                combo_activity_user,
                combo_condition,
            }
        })
        .await
}

fn is_unix_seconds(t: DateTime<Utc>) -> bool {
    // 2015-01-01 .. 2100-01-01: catches ms/µs units mistaken for seconds.
    (1_420_070_400..4_102_444_800).contains(&t.timestamp())
}

/// Asserts the pagination envelope of a first page requested with `limit`.
fn assert_first_page<T>(page: &Page<T>, limit: u32) {
    assert!(page.items().len() <= limit as usize);
    assert_eq!(page.pagination.limit, limit);
    assert_eq!(page.pagination.offset, 0);
    assert_eq!(page.has_more(), page.next_cursor().is_some());
    assert!(page.trace_id().is_some(), "x-trace-id header preserved");
}

// ---------------------------------------------------------------------------------------
// Wallet
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_positions() {
    let s = sample().await;
    let page = pm()
        .data()
        .list_positions()
        .user(s.user.as_str())
        .limit(5)
        .send()
        .await
        .unwrap();
    assert_first_page(&page, 5);
    for p in page.items() {
        assert_eq!(p.proxy_wallet.as_str(), s.user);
    }
    let raw = raw("/positions", &[("user", &s.user), ("limit", "100")]).await;
    check::<Page<Position>>("GET /v2/positions?user", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_positions_by_market() {
    let s = sample().await;
    let page = pm()
        .data()
        .list_positions()
        .conditions([s.condition_id.as_str()])
        .limit(5)
        .send()
        .await
        .unwrap();
    assert_first_page(&page, 5);
    assert!(!page.items().is_empty(), "an active market has holders");
    for p in page.items() {
        assert_eq!(p.condition_id.as_str(), s.condition_id);
    }
    let raw = raw(
        "/positions",
        &[("condition", &s.condition_id), ("limit", "50")],
    )
    .await;
    check::<Page<Position>>("GET /v2/positions?condition", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_positions_filters() {
    let s = sample().await;
    let client = pm();
    let data = client.data();
    for status in [
        PositionStatus::Open,
        PositionStatus::Redeemable,
        PositionStatus::RedeemableLost,
        PositionStatus::Mergeable,
        PositionStatus::Closed,
    ] {
        let page = data
            .list_positions()
            .user(s.user.as_str())
            .status(status.clone())
            .limit(10)
            .send()
            .await
            .unwrap_or_else(|e| panic!("status {status:?}: {e}"));
        assert!(page.items().len() <= 10);
    }
    // CLOSED rows have their own shape quirks (zeroed current_*): check them against live.
    let closed = raw(
        "/positions",
        &[("user", &s.user), ("status", "CLOSED"), ("limit", "50")],
    )
    .await;
    check::<Page<Position>>("GET /v2/positions?status=CLOSED", &closed);
    let redeemable = raw(
        "/positions",
        &[("user", &s.user), ("status", "REDEEMABLE"), ("limit", "50")],
    )
    .await;
    check::<Page<Position>>("GET /v2/positions?status=REDEEMABLE", &redeemable);

    for sort_by in [
        PositionSortBy::CurrentValue,
        PositionSortBy::Price,
        PositionSortBy::Tokens,
        PositionSortBy::UnrealizedPnl,
        PositionSortBy::RealizedPnl,
        PositionSortBy::TotalPnl,
        PositionSortBy::Timestamp,
    ] {
        data.list_positions()
            .user(s.user.as_str())
            .sort_by(sort_by.clone())
            .sort_direction(SortDirection::Asc)
            .limit(3)
            .send()
            .await
            .unwrap_or_else(|e| panic!("sort_by {sort_by:?}: {e}"));
    }
    data.list_positions()
        .user(s.user.as_str())
        .include_archived(true)
        .filter_type(FilterType::Cash)
        .filter_amount(Decimal::from(1))
        .limit(3)
        .send()
        .await
        .unwrap();
    data.list_positions()
        .user(s.user.as_str())
        .event_ids([s.event_id.as_str()])
        .limit(3)
        .send()
        .await
        .unwrap();
    data.list_positions()
        .user(s.user.as_str())
        .conditions([s.condition_id.as_str()])
        .limit(3)
        .send()
        .await
        .unwrap();
    data.list_positions()
        .user(s.user.as_str())
        .title("a")
        .limit(3)
        .send()
        .await
        .unwrap();
    data.list_positions()
        .user(s.user.as_str())
        .status(PositionStatus::Closed)
        .start(Utc::now() - Duration::days(30))
        .end(Utc::now())
        .limit(3)
        .send()
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "live network"]
async fn list_positions_stream_pages() {
    let s = sample().await;
    let positions: Vec<Position> = pm()
        .data()
        .list_positions()
        .user(s.user.as_str())
        .status(PositionStatus::Closed)
        .limit(2)
        .into_stream()
        .take(5)
        .try_collect()
        .await
        .unwrap();
    assert!(positions.len() <= 5);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_combo_positions() {
    let x = extra().await;
    let Some(user) = &x.combo_position_user else {
        eprintln!("NOTE no wallet with combo positions found; checking the empty page only");
        let s = sample().await;
        let page = pm()
            .data()
            .list_combo_positions(s.user.as_str())
            .limit(5)
            .send()
            .await
            .unwrap();
        assert_first_page(&page, 5);
        return;
    };
    let page = pm()
        .data()
        .list_combo_positions(user.as_str())
        .limit(5)
        .send()
        .await
        .unwrap();
    assert_first_page(&page, 5);
    assert!(!page.items().is_empty());
    for p in page.items() {
        assert_eq!(p.legs.len(), p.legs_total as usize);
        assert_eq!(p.legs_total, p.legs_resolved + p.legs_pending);
    }
    let raw = raw("/positions/combos", &[("user", user), ("limit", "1000")]).await;
    check::<Page<ComboPosition>>("GET /v2/positions/combos", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_combo_positions_filters_and_stream() {
    let x = extra().await;
    let Some(user) = &x.combo_position_user else {
        return;
    };
    let client = pm();
    let data = client.data();
    for status in [
        ComboPositionStatus::Open,
        ComboPositionStatus::Redeemable,
        ComboPositionStatus::Partial,
        ComboPositionStatus::ResolvedWin,
        ComboPositionStatus::ResolvedLoss,
        ComboPositionStatus::ResolvedPartial,
    ] {
        data.list_combo_positions(user.as_str())
            .statuses([status.clone()])
            .limit(3)
            .send()
            .await
            .unwrap_or_else(|e| panic!("combo status {status:?}: {e}"));
    }
    for sort_by in [
        ComboPositionSortBy::FirstEntry,
        ComboPositionSortBy::EntryCost,
        ComboPositionSortBy::CurrentValue,
        ComboPositionSortBy::Updated,
    ] {
        for direction in [SortDirection::Asc, SortDirection::Desc] {
            data.list_combo_positions(user.as_str())
                .sort_by(sort_by.clone())
                .sort_direction(direction.clone())
                .limit(3)
                .send()
                .await
                .unwrap_or_else(|e| panic!("combo sort {sort_by:?} {direction:?}: {e}"));
        }
    }
    data.list_combo_positions(user.as_str())
        .updated_after(Utc::now() - Duration::days(365))
        .updated_before(Utc::now() - Duration::minutes(10))
        .limit(3)
        .send()
        .await
        .unwrap();
    let rows: Vec<ComboPosition> = data
        .list_combo_positions(user.as_str())
        .limit(2)
        .into_stream()
        .take(5)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(rows.len(), 5, "this wallet has more than 5 combo positions");
}

/// Pins SPEC_DEVIATIONS.md "Combo condition ids are 62 hex digits": live combo condition
/// ids are `0x` + 62 hex digits (31 bytes), the filter accepts exactly those, and the SDK
/// lets them through client-side.
#[tokio::test]
#[ignore = "live network"]
async fn combo_condition_filter_accepts_live_ids() {
    let x = extra().await;
    let (Some(user), Some(condition)) = (&x.combo_position_user, &x.combo_condition) else {
        panic!("no wallet with combo positions among the board wallets");
    };
    assert_eq!(condition.len(), 2 + 62, "combo condition id {condition}");
    let page = pm()
        .data()
        .list_combo_positions(user.as_str())
        .conditions([condition.as_str()])
        .send()
        .await
        .unwrap_or_else(|e| {
            panic!(
                "combo condition {condition} ({} hex digits): {e}",
                condition.len() - 2
            )
        });
    assert!(!page.items().is_empty());
    assert!(
        page.items()
            .iter()
            .all(|p| p.combo_condition_id.as_str() == condition)
    );
    // Within a combo position, the leg ids on this route are 62 digits too (and are not the
    // legs' market condition ids: see the activity route below).
    for leg in page.items().iter().flat_map(|p| &p.legs) {
        assert_eq!(leg.leg_condition_id.as_str().len(), 2 + 62, "{leg:?}");
    }

    // The same filter on `/v2/activity/combos`.
    let activity = pm()
        .data()
        .list_combo_activity(user.as_str())
        .conditions([condition.as_str()])
        .send()
        .await
        .unwrap();
    assert!(
        activity
            .items()
            .iter()
            .all(|a| a.combo_condition_id.as_str() == condition)
    );

    // The server accepts exactly 62 digits: a bytes32 (64 digits), a short id and a
    // non-hex id are `400 invalid combo condition id` naming `condition`.
    let digits = &condition[2..];
    let too_long = format!("0x{digits}00");
    let too_short = format!("0x{}", &digits[..61]);
    for (what, bad) in [
        ("64 digits", too_long.as_str()),
        ("61 digits", too_short.as_str()),
        ("0x03", "0x03"),
        ("no prefix", digits),
    ] {
        let e = raw_error(
            "/positions/combos",
            &[("user", user.as_str()), ("condition", bad)],
        )
        .await;
        assert_eq!(e.status, 400, "{what}");
        assert_eq!(e.body["parameter"], "condition", "{what}");
        assert!(
            e.body["error"]
                .as_str()
                .unwrap()
                .contains("invalid combo condition id"),
            "{what}: {}",
            e.body
        );
    }
    // Case does not matter.
    let upper = format!("0x{}", digits.to_uppercase());
    let ok = raw(
        "/positions/combos",
        &[("user", user.as_str()), ("condition", &upper)],
    )
    .await;
    assert!(!ok.json["data"].as_array().unwrap().is_empty());
}

/// Pins SPEC_DEVIATIONS.md "Combo `leg_condition_id` differs between routes": the same
/// leg has a 62-digit `0x01`/`0x02`-prefixed id on `/positions/combos` and the market's
/// bytes32 condition id on `/activity/combos`, with the same `leg_position_id`.
#[tokio::test]
#[ignore = "live network"]
async fn combo_leg_condition_ids_differ_between_routes() {
    let x = extra().await;
    let (Some(user), Some(_)) = (&x.combo_position_user, &x.combo_condition) else {
        panic!("no wallet with combo positions among the board wallets");
    };
    let positions = pm()
        .data()
        .list_combo_positions(user.as_str())
        .limit(100)
        .send()
        .await
        .unwrap();
    for leg in positions.items().iter().flat_map(|p| &p.legs) {
        let id = leg.leg_condition_id.as_str();
        assert_eq!(id.len(), 2 + 62, "{id}");
        assert!(id.starts_with("0x01") || id.starts_with("0x02"), "{id}");
    }
    let Some(activity_user) = &x.combo_activity_user else {
        return;
    };
    let activity = pm()
        .data()
        .list_combo_activity(activity_user.as_str())
        .limit(100)
        .send()
        .await
        .unwrap();
    assert!(!activity.items().is_empty());
    for leg in activity.items().iter().flat_map(|a| &a.legs) {
        assert_eq!(leg.leg_condition_id.as_str().len(), 2 + 64, "{leg:?}");
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn get_portfolio_value() {
    let s = sample().await;
    let value = pm()
        .data()
        .get_portfolio_value(s.user.as_str())
        .send()
        .await
        .unwrap();
    assert_eq!(value.proxy_wallet.as_str(), s.user);
    check_data::<PortfolioValue>("GET /v2/value", &raw("/value", &[("user", &s.user)]).await);
    check_data::<PortfolioValue>(
        "GET /v2/value?condition",
        &raw(
            "/value",
            &[("user", &s.user), ("condition", &s.condition_id)],
        )
        .await,
    );
    pm().data()
        .get_portfolio_value(s.user.as_str())
        .conditions([s.condition_id.as_str()])
        .send()
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "live network"]
async fn get_approvals() {
    let s = sample().await;
    let approvals = pm().data().get_approvals(s.user.as_str()).await.unwrap();
    assert!(!approvals.contracts.is_empty());
    assert_eq!(approvals.chain_id, 137);
    check_data::<Approvals>(
        "GET /v2/approvals",
        &raw("/approvals", &[("user", &s.user)]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_user_pnl() {
    let s = sample().await;
    let client = pm();
    let data = client.data();
    let series = data.get_user_pnl(s.user.as_str()).send().await.unwrap();
    assert!(!series.points.is_empty());
    for pair in series.points.windows(2) {
        assert!(pair[0].timestamp < pair[1].timestamp, "oldest first");
    }
    check_data::<UserPnlSeries>(
        "GET /v2/user-pnl",
        &raw("/user-pnl", &[("user", &s.user)]).await,
    );
    check_data::<UserPnlSeries>(
        "GET /v2/user-pnl?interval=1w&fidelity=1d",
        &raw(
            "/user-pnl",
            &[("user", &s.user), ("interval", "1w"), ("fidelity", "1d")],
        )
        .await,
    );
    for interval in [
        PnlInterval::Max,
        PnlInterval::All,
        PnlInterval::OneMonth,
        PnlInterval::OneWeek,
        PnlInterval::OneDay,
        PnlInterval::TwelveHours,
        PnlInterval::SixHours,
    ] {
        data.get_user_pnl(s.user.as_str())
            .interval(interval.clone())
            .send()
            .await
            .unwrap_or_else(|e| panic!("interval {interval:?}: {e}"));
    }
    for fidelity in [
        PnlFidelity::OneDay,
        PnlFidelity::EighteenHours,
        PnlFidelity::TwelveHours,
        PnlFidelity::ThreeHours,
        PnlFidelity::OneHour,
    ] {
        data.get_user_pnl(s.user.as_str())
            .interval(PnlInterval::OneMonth)
            .fidelity(fidelity.clone())
            .send()
            .await
            .unwrap_or_else(|e| panic!("fidelity {fidelity:?}: {e}"));
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn get_user_stats() {
    let x = extra().await;
    let stats = pm()
        .data()
        .get_user_stats(x.stats_user.as_str())
        .await
        .unwrap()
        .expect("the all-time leader has stats");
    assert_eq!(stats.proxy_wallet.as_str(), x.stats_user);
    check_data::<Option<UserStats>>(
        "GET /v2/user-stats",
        &raw("/user-stats", &[("user", &x.stats_user)]).await,
    );
}

/// `data: null` is "no such user", returned as `Ok(None)`.
#[tokio::test]
#[ignore = "live network"]
async fn get_user_stats_miss_is_none() {
    let client = pm();
    let data = client.data();
    let stats = data
        .get_user_stats("0x1f3a646ce5cbe2c70270e7a46161d89b7c7b895e")
        .await
        .unwrap();
    assert!(stats.is_none());
    // A known wallet with no activity is a row of zeros and `null` join_date/all_time_pnl,
    // not `data: null`.
    let zeros = data
        .get_user_stats("0x0000000000000000000000000000000000000001")
        .await
        .unwrap()
        .expect("known wallet with zero stats");
    assert_eq!(zeros.trades, 0);
    assert!(zeros.join_date.is_none());
    assert!(zeros.all_time_pnl.is_none());
    check_data::<Option<UserStats>>(
        "GET /v2/user-stats (zero row)",
        &raw(
            "/user-stats",
            &[("user", "0x0000000000000000000000000000000000000001")],
        )
        .await,
    );
    // The day-board leader trades and ranks but live answers `data: null` for it (the
    // docs say null means "not a known user"). Either way the SDK must decode it.
    let s = sample().await;
    let leader = data.get_user_stats(s.user.as_str()).await.unwrap();
    eprintln!(
        "NOTE user-stats for the day leader {}: {}",
        s.user,
        if leader.is_some() { "row" } else { "null" }
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_user_volume() {
    let s = sample().await;
    let volume = pm()
        .data()
        .get_user_volume(s.user.as_str())
        .send()
        .await
        .unwrap();
    let _ = volume;
    check_data::<UserVolume>(
        "GET /v2/user-volume",
        &raw("/user-volume", &[("user", &s.user)]).await,
    );
    let start = (Utc::now() - Duration::days(7)).timestamp().to_string();
    let end = Utc::now().timestamp().to_string();
    check_data::<UserVolume>(
        "GET /v2/user-volume?start&end",
        &raw(
            "/user-volume",
            &[("user", &s.user), ("start", &start), ("end", &end)],
        )
        .await,
    );
    pm().data()
        .get_user_volume(s.user.as_str())
        .start(Utc::now() - Duration::days(7))
        .end(Utc::now())
        .send()
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------------------
// Feeds
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_trades() {
    let s = sample().await;
    let page = pm()
        .data()
        .list_trades()
        .user(s.user.as_str())
        .limit(5)
        .send()
        .await
        .unwrap();
    assert_first_page(&page, 5);
    assert!(!page.items().is_empty());
    for t in page.items() {
        assert_eq!(t.proxy_wallet.as_str(), s.user);
        assert!(is_unix_seconds(t.timestamp), "{}", t.timestamp);
    }
    for pair in page.items().windows(2) {
        assert!(pair[0].timestamp >= pair[1].timestamp, "newest first");
    }
    check::<Page<Trade>>(
        "GET /v2/trades?user",
        &raw("/trades", &[("user", &s.user), ("limit", "100")]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_trades_other_shapes() {
    let s = sample().await;
    let client = pm();
    let data = client.data();
    let by_condition = data
        .list_trades()
        .conditions([s.condition_id.as_str()])
        .limit(5)
        .send()
        .await
        .unwrap();
    assert!(!by_condition.items().is_empty());
    assert!(
        by_condition
            .items()
            .iter()
            .all(|t| t.condition_id.as_str() == s.condition_id)
    );
    check::<Page<Trade>>(
        "GET /v2/trades?condition",
        &raw(
            "/trades",
            &[("condition", &s.condition_id), ("limit", "100")],
        )
        .await,
    );
    let by_event = data
        .list_trades()
        .event_ids([s.event_id.as_str()])
        .limit(5)
        .send()
        .await
        .unwrap();
    assert!(!by_event.items().is_empty());
    check::<Page<Trade>>(
        "GET /v2/trades?event_id",
        &raw("/trades", &[("event_id", &s.event_id), ("limit", "100")]).await,
    );
    let bare = data.list_trades().limit(5).send().await.unwrap();
    assert!(!bare.items().is_empty());
    check::<Page<Trade>>("GET /v2/trades", &raw("/trades", &[("limit", "100")]).await);

    let sells = data
        .list_trades()
        .user(s.user.as_str())
        .side(Side::Sell)
        .taker_only(false)
        .filter_type(FilterType::Tokens)
        .filter_amount(Decimal::from(1))
        .limit(5)
        .send()
        .await
        .unwrap();
    assert!(sells.items().iter().all(|t| t.side == Side::Sell));
    let windowed = data
        .list_trades()
        .user(s.user.as_str())
        .start(Utc::now() - Duration::days(7))
        .end(Utc::now())
        .limit(5)
        .send()
        .await
        .unwrap();
    assert!(windowed.items().len() <= 5);
    data.list_trades()
        .user(s.user.as_str())
        .full_history()
        .limit(5)
        .send()
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "live network"]
async fn list_trades_stream_pages() {
    let s = sample().await;
    let trades: Vec<Trade> = pm()
        .data()
        .list_trades()
        .user(s.user.as_str())
        .limit(3)
        .into_stream()
        .take(8)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(trades.len(), 8, "three pages of 3 give 8 rows");
    let unique: HashSet<_> = trades
        .iter()
        .map(|t| {
            (
                t.transaction_hash.clone(),
                t.token_id.as_str().to_owned(),
                t.size,
            )
        })
        .collect();
    assert!(unique.len() >= 2);
    for pair in trades.windows(2) {
        assert!(pair[0].timestamp >= pair[1].timestamp, "stays newest first");
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn list_trades_manual_cursor_walk() {
    let s = sample().await;
    let client = pm();
    let data = client.data();
    let first = data
        .list_trades()
        .user(s.user.as_str())
        .limit(3)
        .send()
        .await
        .unwrap();
    let cursor = first.next_cursor().expect("more than 3 trades").to_owned();
    let second = data
        .list_trades()
        .user(s.user.as_str())
        .limit(3)
        .cursor(cursor)
        .send()
        .await
        .unwrap();
    assert_eq!(second.items().len(), 3);
    assert!(
        first
            .items()
            .iter()
            .all(|a| second.items().iter().all(|b| a != b)),
        "pages do not overlap"
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_activity() {
    let s = sample().await;
    let page = pm()
        .data()
        .list_activity(s.user.as_str())
        .limit(5)
        .send()
        .await
        .unwrap();
    assert_first_page(&page, 5);
    assert!(!page.items().is_empty());
    for a in page.items() {
        assert_eq!(a.proxy_wallet.as_str(), s.user);
        assert!(is_unix_seconds(a.timestamp), "{}", a.timestamp);
    }
    check::<Page<Activity>>(
        "GET /v2/activity",
        &raw("/activity", &[("user", &s.user), ("limit", "200")]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_activity_type_filters() {
    let s = sample().await;
    let client = pm();
    let data = client.data();
    for ty in [
        ActivityType::Trade,
        ActivityType::Split,
        ActivityType::Merge,
        ActivityType::Redeem,
        ActivityType::Reward,
        ActivityType::Conversion,
        ActivityType::Tip,
    ] {
        let page = data
            .list_activity(s.user.as_str())
            .types([ty.clone()])
            .limit(20)
            .send()
            .await
            .unwrap_or_else(|e| panic!("type {ty:?}: {e}"));
        assert!(
            page.items().iter().all(|a| a.activity_type == ty),
            "type filter {ty:?} returned other types"
        );
        let wire = ty.as_str().to_owned();
        check::<Page<Activity>>(
            &format!("GET /v2/activity?type={wire}"),
            &raw(
                "/activity",
                &[("user", &s.user), ("type", &wire), ("limit", "100")],
            )
            .await,
        );
    }
    let multi = data
        .list_activity(s.user.as_str())
        .types([ActivityType::Trade, ActivityType::Redeem])
        .limit(20)
        .send()
        .await
        .unwrap();
    assert!(
        multi
            .items()
            .iter()
            .all(|a| matches!(a.activity_type, ActivityType::Trade | ActivityType::Redeem))
    );
    data.list_activity(s.user.as_str())
        .event_ids([s.event_id.as_str()])
        .limit(5)
        .send()
        .await
        .unwrap();
    data.list_activity(s.user.as_str())
        .conditions([s.condition_id.as_str()])
        .side(Side::Buy)
        .start(Utc::now() - Duration::days(30))
        .end(Utc::now())
        .sort_by(ActivitySortBy::Timestamp)
        .sort_direction(SortDirection::Asc)
        .exclude_deposits_withdrawals(true)
        .limit(5)
        .send()
        .await
        .unwrap();
    data.list_activity(s.user.as_str())
        .full_history()
        .exclude_deposits_withdrawals(false)
        .limit(5)
        .send()
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "live network"]
async fn list_activity_stream_pages() {
    let s = sample().await;
    let rows: Vec<Activity> = pm()
        .data()
        .list_activity(s.user.as_str())
        .limit(3)
        .into_stream()
        .take(8)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(rows.len(), 8);
    for pair in rows.windows(2) {
        assert!(pair[0].timestamp >= pair[1].timestamp, "newest first");
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn list_combo_activity() {
    let x = extra().await;
    let Some(user) = &x.combo_activity_user else {
        eprintln!("NOTE no wallet with combo activity found; checking the empty page only");
        let s = sample().await;
        let page = pm()
            .data()
            .list_combo_activity(s.user.as_str())
            .limit(5)
            .send()
            .await
            .unwrap();
        assert_first_page(&page, 5);
        return;
    };
    let page = pm()
        .data()
        .list_combo_activity(user.as_str())
        .limit(5)
        .send()
        .await
        .unwrap();
    assert_first_page(&page, 5);
    assert!(!page.items().is_empty());
    for a in page.items() {
        assert_eq!(a.proxy_wallet.as_str(), user.as_str());
        assert!(is_unix_seconds(a.timestamp), "{}", a.timestamp);
    }
    check::<Page<ComboActivity>>(
        "GET /v2/activity/combos",
        &raw("/activity/combos", &[("user", user), ("limit", "1000")]).await,
    );
    let rows: Vec<ComboActivity> = pm()
        .data()
        .list_combo_activity(user.as_str())
        .limit(2)
        .into_stream()
        .take(5)
        .try_collect()
        .await
        .unwrap();
    assert!(!rows.is_empty());
}

// ---------------------------------------------------------------------------------------
// Markets
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_holders() {
    let s = sample().await;
    let page = pm()
        .data()
        .list_holders([s.condition_id.as_str()])
        .limit(3)
        .send()
        .await
        .unwrap();
    assert_first_page(&page, 3);
    assert!(!page.items().is_empty());
    for group in page.items() {
        assert!(s.token_ids.iter().any(|t| t == group.token_id.as_str()));
    }
    check::<Page<HolderGroup>>(
        "GET /v2/holders",
        &raw(
            "/holders",
            &[("condition", &s.condition_id), ("limit", "20")],
        )
        .await,
    );
    check::<Page<HolderGroup>>(
        "GET /v2/holders?include_pnl",
        &raw(
            "/holders",
            &[
                ("condition", &s.condition_id),
                ("limit", "20"),
                ("include_pnl", "true"),
            ],
        )
        .await,
    );
    pm().data()
        .list_holders([s.condition_id.as_str()])
        .include_pnl(true)
        .min_balance(Decimal::from(10))
        .limit(3)
        .send()
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "live network"]
async fn list_holders_stream_pages() {
    let s = sample().await;
    let groups: Vec<HolderGroup> = pm()
        .data()
        .list_holders([s.condition_id.as_str()])
        .limit(1)
        .into_stream()
        .take(2)
        .try_collect()
        .await
        .unwrap();
    assert!(!groups.is_empty());
}

#[tokio::test]
#[ignore = "live network"]
async fn get_open_interest() {
    let s = sample().await;
    let global = pm().data().get_open_interest().send().await.unwrap();
    assert!(global.iter().any(OpenInterest::is_global));
    check_data::<Vec<OpenInterest>>("GET /v2/oi", &raw("/oi", &[]).await);
    let one = pm()
        .data()
        .get_open_interest()
        .conditions([s.condition_id.as_str()])
        .send()
        .await
        .unwrap();
    assert_eq!(one.len(), 1);
    assert!(!one[0].is_global());
    check_data::<Vec<OpenInterest>>(
        "GET /v2/oi?condition",
        &raw("/oi", &[("condition", &s.condition_id)]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_live_volume() {
    let s = sample().await;
    let volume = pm()
        .data()
        .get_live_volume([s.event_id.as_str()])
        .await
        .unwrap();
    assert!(
        volume
            .conditions
            .iter()
            .any(|c| c.condition_id.as_str() == s.condition_id)
            || volume.conditions.is_empty()
    );
    check_data::<polyoxide::data::LiveVolume>(
        "GET /v2/live-volume",
        &raw("/live-volume", &[("event_id", &s.event_id)]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_prices_history() {
    let s = sample().await;
    let token = s.token_ids[0].as_str();
    let client = pm();
    let data = client.data();
    let page = data
        .list_prices_history(token)
        .interval(PriceHistoryInterval::OneDay)
        .limit(5)
        .send()
        .await
        .unwrap();
    assert_first_page(&page, 5);
    assert!(!page.items().is_empty());
    for pair in page.items().windows(2) {
        assert!(pair[0].timestamp < pair[1].timestamp, "oldest first");
    }
    check::<Page<PricePoint>>(
        "GET /v2/prices-history?interval=1d",
        &raw(
            "/prices-history",
            &[("token_id", token), ("interval", "1d"), ("limit", "500")],
        )
        .await,
    );
    for interval in [
        PriceHistoryInterval::Max,
        PriceHistoryInterval::All,
        PriceHistoryInterval::OneMonth,
        PriceHistoryInterval::OneWeek,
        PriceHistoryInterval::SixHours,
        PriceHistoryInterval::OneHour,
    ] {
        data.list_prices_history(token)
            .interval(interval.clone())
            .limit(5)
            .send()
            .await
            .unwrap_or_else(|e| panic!("interval {interval:?}: {e}"));
    }
    let end = Utc::now();
    let start = end - Duration::days(1);
    let windowed = data
        .list_prices_history(token)
        .start(start)
        .end(end)
        .bucket_seconds(3600)
        .limit(100)
        .send()
        .await
        .unwrap();
    assert!(!windowed.items().is_empty());
    assert!(
        windowed
            .items()
            .iter()
            .all(|p| p.timestamp >= start - Duration::seconds(3600)),
        "points are bucket-aligned: the first may precede `start` by under one bucket"
    );
    check::<Page<PricePoint>>(
        "GET /v2/prices-history?start&end&bucket_seconds",
        &raw(
            "/prices-history",
            &[
                ("token_id", token),
                ("start", &start.timestamp().to_string()),
                ("end", &end.timestamp().to_string()),
                ("bucket_seconds", "3600"),
            ],
        )
        .await,
    );
    let point = data
        .list_prices_history(token)
        .as_of(Utc::now() - Duration::hours(1))
        .send()
        .await
        .unwrap();
    assert_eq!(point.items().len(), 1, "as_of is a single observation");
    check::<Page<PricePoint>>(
        "GET /v2/prices-history?as_of",
        &raw(
            "/prices-history",
            &[
                ("token_id", token),
                (
                    "as_of",
                    &(Utc::now() - Duration::hours(1)).timestamp().to_string(),
                ),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_prices_history_stream_pages() {
    let s = sample().await;
    let end = Utc::now();
    let points: Vec<PricePoint> = pm()
        .data()
        .list_prices_history(s.token_ids[0].as_str())
        .start(end - Duration::days(1))
        .end(end)
        .bucket_seconds(60)
        .limit(10)
        .into_stream()
        .take(25)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(points.len(), 25, "three pages of 10");
    for pair in points.windows(2) {
        assert!(
            pair[0].timestamp < pair[1].timestamp,
            "oldest first across pages"
        );
    }
}

/// Open question 20: does a cursor page need the window restated?
#[tokio::test]
#[ignore = "live network"]
async fn prices_history_bare_cursor_and_window_cap() {
    let s = sample().await;
    let token = s.token_ids[0].as_str();
    let end = Utc::now().timestamp().to_string();
    let start = (Utc::now() - Duration::days(1)).timestamp().to_string();
    let first = raw(
        "/prices-history",
        &[
            ("token_id", token),
            ("start", &start),
            ("end", &end),
            ("bucket_seconds", "60"),
            ("limit", "3"),
        ],
    )
    .await;
    let cursor = first.json["pagination"]["next_cursor"]
        .as_str()
        .expect("a next page")
        .to_owned();
    // Cursor plus token only.
    let bare = raw_error(
        "/prices-history",
        &[("token_id", token), ("cursor", &cursor)],
    )
    .await;
    eprintln!(
        "NOTE prices-history cursor with token only: HTTP {} {}",
        bare.status, bare.body
    );
    // Restating the window with the cursor works.
    let restated = get(
        DATA,
        "/v2/prices-history",
        &[
            ("token_id", token),
            ("start", &start),
            ("end", &end),
            ("bucket_seconds", "60"),
            ("limit", "3"),
            ("cursor", &cursor),
        ],
    )
    .await;
    assert!(!restated.json["data"].as_array().unwrap().is_empty());
    // The 15-day cap: a window of 20 days.
    let far = (Utc::now() - Duration::days(20)).timestamp().to_string();
    let capped = raw_error(
        "/prices-history",
        &[("token_id", token), ("start", &far), ("end", &end)],
    )
    .await;
    eprintln!(
        "NOTE prices-history 20-day window: HTTP {} {}",
        capped.status, capped.body
    );
    // `start` alone older than 15 days.
    let alone = raw_error("/prices-history", &[("token_id", token), ("start", &far)]).await;
    eprintln!(
        "NOTE prices-history start 20 days back, no end: HTTP {} {}",
        alone.status, alone.body
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_resolutions() {
    let x = extra().await;
    let client = pm();
    let data = client.data();
    let by_condition = data
        .get_resolutions(ResolutionSelector::conditions([x
            .resolved_condition
            .as_str()]))
        .await
        .unwrap();
    assert_eq!(by_condition.len(), 1);
    assert_eq!(
        by_condition[0].condition_id.as_ref().map(|c| c.as_str()),
        Some(x.resolved_condition.as_str())
    );
    assert!(by_condition[0].last_update_time().is_some());
    check_data::<Vec<Resolution>>(
        "GET /v2/resolutions?condition (resolved)",
        &raw("/resolutions", &[("condition", &x.resolved_condition)]).await,
    );

    let s = sample().await;
    let active = data
        .get_resolutions(ResolutionSelector::conditions([s.condition_id.as_str()]))
        .await
        .unwrap();
    assert_eq!(active.len(), 1);
    assert!(active[0].last_update_time().is_some() || active[0].last_update_timestamp.is_empty());
    check_data::<Vec<Resolution>>(
        "GET /v2/resolutions?condition (active)",
        &raw("/resolutions", &[("condition", &s.condition_id)]).await,
    );

    if let Some(question) = &x.resolved_question {
        let by_question = data
            .get_resolutions(ResolutionSelector::question(question.as_str()))
            .await
            .unwrap();
        assert!(!by_question.is_empty());
        assert!(by_question[0].condition_id.is_none());
        assert!(by_question[0].last_update_time().is_some());
        check_data::<Vec<Resolution>>(
            "GET /v2/resolutions?question_id",
            &raw("/resolutions", &[("question_id", question)]).await,
        );
    }
    if let Some(event) = &x.resolved_event {
        let by_event = data
            .get_resolutions(ResolutionSelector::events([event.as_str()]))
            .await
            .unwrap();
        assert!(!by_event.is_empty());
        check_data::<Vec<Resolution>>(
            "GET /v2/resolutions?event_id",
            &raw("/resolutions", &[("event_id", event)]).await,
        );
    }
    let both = data
        .get_resolutions(ResolutionSelector::conditions([
            x.resolved_condition.as_str(),
            s.condition_id.as_str(),
        ]))
        .await
        .unwrap();
    assert_eq!(both.len(), 2);
}

// ---------------------------------------------------------------------------------------
// Boards
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_leaderboard() {
    let client = pm();
    let data = client.data();
    let page = data.list_leaderboard().limit(5).send().await.unwrap();
    assert_first_page(&page, 5);
    assert_eq!(page.items().len(), 5);
    assert_eq!(page.items()[0].rank, 1);
    check::<Page<LeaderboardEntry>>(
        "GET /v2/leaderboard",
        &raw("/leaderboard", &[("limit", "100")]).await,
    );
    for period in [
        TimePeriod::Day,
        TimePeriod::Week,
        TimePeriod::Month,
        TimePeriod::All,
    ] {
        for sort_by in [LeaderboardSortBy::Pnl, LeaderboardSortBy::Volume] {
            let page = data
                .list_leaderboard()
                .time_period(period.clone())
                .sort_by(sort_by.clone())
                .limit(5)
                .send()
                .await
                .unwrap_or_else(|e| panic!("{period:?} {sort_by:?}: {e}"));
            assert_eq!(page.items().len(), 5);
            check::<Page<LeaderboardEntry>>(
                &format!(
                    "GET /v2/leaderboard?time_period={}&sort_by={}",
                    period.as_str(),
                    sort_by.as_str()
                ),
                &raw(
                    "/leaderboard",
                    &[
                        ("time_period", period.as_str()),
                        ("sort_by", sort_by.as_str()),
                        ("limit", "20"),
                    ],
                )
                .await,
            );
        }
    }
    data.list_leaderboard()
        .category("politics")
        .limit(5)
        .send()
        .await
        .unwrap();
    check::<Page<LeaderboardEntry>>(
        "GET /v2/leaderboard?category=politics",
        &raw("/leaderboard", &[("category", "politics"), ("limit", "20")]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_leaderboard_stream_pages() {
    let rows: Vec<LeaderboardEntry> = pm()
        .data()
        .list_leaderboard()
        .limit(3)
        .into_stream()
        .take(8)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(rows.len(), 8);
    for pair in rows.windows(2) {
        assert!(
            pair[0].rank <= pair[1].rank,
            "ranks never decrease across pages"
        );
    }
    let wallets: HashSet<_> = rows.iter().map(|r| r.user_id.as_str().to_owned()).collect();
    assert_eq!(wallets.len(), 8, "no row repeats across pages");
}

#[tokio::test]
#[ignore = "live network"]
async fn get_leaderboard_standing() {
    let s = sample().await;
    let standing = pm()
        .data()
        .get_leaderboard_standing(s.user.as_str())
        .send()
        .await
        .unwrap()
        .expect("the day leader is ranked");
    assert_eq!(standing.user_id.as_str(), s.user);
    check_data::<Option<LeaderboardUserEntry>>(
        "GET /v2/leaderboard?user",
        &raw("/leaderboard", &[("user", &s.user)]).await,
    );
    pm().data()
        .get_leaderboard_standing(s.user.as_str())
        .time_period(TimePeriod::All)
        .category("overall")
        .send()
        .await
        .unwrap();
}

/// Open question 19: what an unranked wallet looks like.
#[tokio::test]
#[ignore = "live network"]
async fn get_leaderboard_standing_unranked() {
    let s = sample().await;
    let unknown = pm()
        .data()
        .get_leaderboard_standing("0x0000000000000000000000000000000000000001")
        .send()
        .await
        .unwrap();
    assert!(unknown.is_none(), "an unknown wallet is `data: null`");

    // The day leader on the *volume* or category boards may be unranked on one side.
    let mut seen_ranks = Vec::new();
    for category in ["overall", "politics", "sports", "crypto"] {
        let raw = raw(
            "/leaderboard",
            &[
                ("user", &s.user),
                ("time_period", "week"),
                ("category", category),
            ],
        )
        .await;
        let entry: Option<LeaderboardUserEntry> = check_data(
            &format!("GET /v2/leaderboard?user&category={category}"),
            &raw,
        );
        if let Some(entry) = entry {
            seen_ranks.push((category, entry.rank_pnl, entry.rank_volume));
        }
    }
    eprintln!("NOTE standing ranks (category, rank_pnl, rank_volume): {seen_ranks:?}");
}

#[tokio::test]
#[ignore = "live network"]
async fn list_biggest_winners() {
    let client = pm();
    let data = client.data();
    let page = data.list_biggest_winners().limit(5).send().await.unwrap();
    assert_first_page(&page, 5);
    assert_eq!(page.items().len(), 5);
    assert_eq!(page.items()[0].win_rank, 1);
    check::<Page<BiggestWinner>>(
        "GET /v2/biggest-winners",
        &raw("/biggest-winners", &[("limit", "100")]).await,
    );
    for period in [
        TimePeriod::Day,
        TimePeriod::Week,
        TimePeriod::Month,
        TimePeriod::All,
    ] {
        check::<Page<BiggestWinner>>(
            &format!("GET /v2/biggest-winners?time_period={}", period.as_str()),
            &raw(
                "/biggest-winners",
                &[("time_period", period.as_str()), ("limit", "100")],
            )
            .await,
        );
    }
    data.list_biggest_winners()
        .time_period(TimePeriod::Week)
        .category("sports")
        .limit(3)
        .send()
        .await
        .unwrap();
    let rows: Vec<BiggestWinner> = data
        .list_biggest_winners()
        .limit(3)
        .into_stream()
        .take(8)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(rows.len(), 8);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_builders_leaderboard() {
    let client = pm();
    let data = client.data();
    let page = data
        .list_builders_leaderboard()
        .limit(3)
        .send()
        .await
        .unwrap();
    assert_first_page(&page, 3);
    assert!(!page.items().is_empty());
    check::<Page<BuilderStanding>>(
        "GET /v2/builders/leaderboard",
        &raw("/builders/leaderboard", &[("limit", "50")]).await,
    );
    for period in [
        TimePeriod::Day,
        TimePeriod::Week,
        TimePeriod::Month,
        TimePeriod::All,
    ] {
        data.list_builders_leaderboard()
            .time_period(period.clone())
            .limit(3)
            .send()
            .await
            .unwrap_or_else(|e| panic!("{period:?}: {e}"));
        check::<Page<BuilderStanding>>(
            &format!(
                "GET /v2/builders/leaderboard?time_period={}",
                period.as_str()
            ),
            &raw(
                "/builders/leaderboard",
                &[("time_period", period.as_str()), ("limit", "50")],
            )
            .await,
        );
    }
    let rows: Vec<BuilderStanding> = data
        .list_builders_leaderboard()
        .limit(2)
        .into_stream()
        .take(5)
        .try_collect()
        .await
        .unwrap();
    assert!(!rows.is_empty());
}

#[tokio::test]
#[ignore = "live network"]
async fn get_builders_volume() {
    let client = pm();
    let data = client.data();
    let points = data.get_builders_volume().limit(5).send().await.unwrap();
    assert!(!points.is_empty());
    check_data::<Vec<BuilderVolumePoint>>(
        "GET /v2/builders/volume",
        &raw("/builders/volume", &[("limit", "50")]).await,
    );
    for interval in [
        TimePeriod::Day,
        TimePeriod::Week,
        TimePeriod::Month,
        TimePeriod::All,
    ] {
        data.get_builders_volume()
            .interval(interval.clone())
            .limit(5)
            .send()
            .await
            .unwrap_or_else(|e| panic!("{interval:?}: {e}"));
        check_data::<Vec<BuilderVolumePoint>>(
            &format!("GET /v2/builders/volume?interval={}", interval.as_str()),
            &raw(
                "/builders/volume",
                &[("interval", interval.as_str()), ("limit", "50")],
            )
            .await,
        );
    }
}

// ---------------------------------------------------------------------------------------
// Wide sweeps (drift coverage over varied, large pages)
// ---------------------------------------------------------------------------------------

/// Decodes large pages for several heavy wallets so rarer row shapes (other activity
/// types, closed positions, redemptions) are checked for decode failures and drift.
#[tokio::test]
#[ignore = "live network"]
async fn sweep_heavy_wallets() {
    let board = raw(
        "/leaderboard",
        &[
            ("time_period", "all"),
            ("sort_by", "VOLUME"),
            ("limit", "4"),
        ],
    )
    .await;
    let wallets: Vec<String> = board.json["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["user_id"].as_str().unwrap().to_owned())
        .collect();
    for user in &wallets {
        check::<Page<Activity>>(
            "SWEEP GET /v2/activity",
            &raw("/activity", &[("user", user), ("limit", "1000")]).await,
        );
        check::<Page<Activity>>(
            "SWEEP GET /v2/activity?exclude_deposits_withdrawals=false",
            &raw(
                "/activity",
                &[
                    ("user", user),
                    ("exclude_deposits_withdrawals", "false"),
                    ("limit", "1000"),
                ],
            )
            .await,
        );
        check::<Page<Activity>>(
            "SWEEP GET /v2/activity?type=SPLIT,MERGE,REWARD,CONVERSION,TIP",
            &raw(
                "/activity",
                &[
                    ("user", user),
                    ("type", "SPLIT,MERGE,REWARD,CONVERSION,TIP"),
                    ("limit", "1000"),
                ],
            )
            .await,
        );
        check::<Page<Trade>>(
            "SWEEP GET /v2/trades",
            &raw("/trades", &[("user", user), ("limit", "1000")]).await,
        );
        for status in [
            "OPEN",
            "CLOSED",
            "REDEEMABLE",
            "REDEEMABLE_LOST",
            "MERGEABLE",
        ] {
            check::<Page<Position>>(
                &format!("SWEEP GET /v2/positions?status={status}"),
                &raw(
                    "/positions",
                    &[("user", user), ("status", status), ("limit", "1000")],
                )
                .await,
            );
        }
        check_data::<UserPnlSeries>(
            "SWEEP GET /v2/user-pnl?interval=max",
            &raw("/user-pnl", &[("user", user), ("interval", "max")]).await,
        );
        check_data::<UserPnlSeries>(
            "SWEEP GET /v2/user-pnl?interval=1d&fidelity=1h",
            &raw(
                "/user-pnl",
                &[("user", user), ("interval", "1d"), ("fidelity", "1h")],
            )
            .await,
        );
        check_data::<UserVolume>(
            "SWEEP GET /v2/user-volume",
            &raw("/user-volume", &[("user", user)]).await,
        );
        check_data::<PortfolioValue>(
            "SWEEP GET /v2/value",
            &raw("/value", &[("user", user)]).await,
        );
        check_data::<Option<UserStats>>(
            "SWEEP GET /v2/user-stats",
            &raw("/user-stats", &[("user", user)]).await,
        );
        check_data::<Approvals>(
            "SWEEP GET /v2/approvals",
            &raw("/approvals", &[("user", user)]).await,
        );
    }
}

/// Decodes the resolution state of many closed markets (varied lifecycles and market
/// types) in one request.
#[tokio::test]
#[ignore = "live network"]
async fn sweep_resolutions() {
    let markets = get(
        GAMMA,
        "/markets",
        &[("closed", "true"), ("limit", "20"), ("order", "volume")],
    )
    .await
    .json;
    let conditions: Vec<&str> = markets
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["conditionId"].as_str())
        .collect();
    let rows = pm()
        .data()
        .get_resolutions(ResolutionSelector::conditions(conditions.iter().copied()))
        .await
        .unwrap();
    assert_eq!(rows.len(), conditions.len());
    check_data::<Vec<Resolution>>(
        "SWEEP GET /v2/resolutions?condition=<20 closed>",
        &raw("/resolutions", &[("condition", &conditions.join(","))]).await,
    );
    // Unresolved UMA questions in flight (proposed / disputed) use the UMA lifecycle.
    let open = get(
        GAMMA,
        "/markets",
        &[
            ("closed", "false"),
            ("limit", "20"),
            ("order", "volume24hr"),
            ("ascending", "false"),
        ],
    )
    .await
    .json;
    let conditions: Vec<&str> = open
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["conditionId"].as_str())
        .collect();
    check_data::<Vec<Resolution>>(
        "SWEEP GET /v2/resolutions?condition=<20 active>",
        &raw("/resolutions", &[("condition", &conditions.join(","))]).await,
    );
    for market in open.as_array().unwrap().iter().take(10) {
        if let Some(question) = market["questionID"].as_str() {
            check_data::<Vec<Resolution>>(
                "SWEEP GET /v2/resolutions?question_id=<active>",
                &raw("/resolutions", &[("question_id", question)]).await,
            );
        }
    }
}

// ---------------------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn get_status() {
    let status = pm().data().get_status().await.unwrap();
    assert!(status.age_seconds >= 0);
    assert_eq!(status.ingestion.chain_id, 137);
    check_data::<ServiceStatus>("GET /v2/status", &raw("/status", &[]).await);
}

// ---------------------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------------------

/// `offset` is not a parameter: documented `400`. The SDK has no such setter, so this is
/// fetched raw and decoded as the documented error body.
#[tokio::test]
#[ignore = "live network"]
async fn raw_offset_param_is_400() {
    let e = raw_error("/leaderboard", &[("offset", "1")]).await;
    assert_eq!(e.status, 400);
    assert_eq!(e.body["code"], "invalid_request");
    assert_eq!(e.body["retryable"], false);
    let trace = e.body["trace_id"].as_str().expect("trace_id in body");
    assert_eq!(
        e.trace_header.as_deref(),
        Some(trace),
        "x-trace-id matches the body"
    );
    assert!(e.body["error"].as_str().unwrap().contains("offset"));
}

#[tokio::test]
#[ignore = "live network"]
async fn raw_invalid_address_is_400_with_parameter() {
    let e = raw_error("/approvals", &[("user", "nope")]).await;
    assert_eq!(e.status, 400);
    assert_eq!(e.body["code"], "invalid_request");
    assert_eq!(e.body["parameter"], "user");
    assert_eq!(e.trace_header.as_deref(), e.body["trace_id"].as_str());
}

#[tokio::test]
#[ignore = "live network"]
async fn raw_unknown_route_is_404() {
    let e = raw_error("/does-not-exist", &[]).await;
    assert_eq!(e.status, 404);
    assert_eq!(e.body["code"], "not_found");
    assert_eq!(e.trace_header.as_deref(), e.body["trace_id"].as_str());
}

/// A malformed query string (not a validation error of a named parameter) is still the
/// documented body shape.
#[tokio::test]
#[ignore = "live network"]
async fn raw_malformed_limit_is_400() {
    let e = raw_error("/leaderboard", &[("limit", "-1")]).await;
    assert_eq!(e.status, 400);
    assert_eq!(e.body["code"], "invalid_request");
    assert!(e.body.get("parameter").is_none());
}

/// An invalid cursor passes SDK validation (it is opaque) and is rejected by the server.
#[tokio::test]
#[ignore = "live network"]
async fn sdk_invalid_cursor_surfaces_typed_error() {
    let err = pm()
        .data()
        .list_leaderboard()
        .cursor("garbage")
        .send()
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 400);
    assert_eq!(api.code(), Some("invalid_request"));
    assert_eq!(api.retryable(), Some(false));
    assert!(api.message().is_some());
    let trace = api.trace_id().expect("trace id");
    assert_eq!(err.trace_id(), Some(trace));
    assert_eq!(ErrorCode::from_error(&err), Some(ErrorCode::InvalidRequest));
    assert!(!err.is_retryable());
    assert_eq!(err.retry_after(), None);
}

/// A `user` that is non-empty but not an address is only validated on two routes; the
/// others forward it. Live, `/v2/positions` answers `200` with an empty page.
#[tokio::test]
#[ignore = "live network"]
async fn sdk_malformed_user_forwarded_to_server() {
    let page = pm()
        .data()
        .list_positions()
        .user("0x1234")
        .send()
        .await
        .unwrap();
    assert!(page.items().is_empty());
    let page = pm()
        .data()
        .list_trades()
        .user("0x1234")
        .send()
        .await
        .unwrap();
    assert!(page.items().is_empty());
}

/// `/v2/prices-history` documents a `0` bound as a `400` naming the parameter; the SDK
/// rejects a bound at or before the Unix epoch client-side, so the server's answer is read
/// raw.
#[tokio::test]
#[ignore = "live network"]
async fn prices_history_zero_start_is_400_with_parameter() {
    let s = sample().await;
    let token = s.token_ids[0].as_str();
    let e = raw_error("/prices-history", &[("token_id", token), ("start", "0")]).await;
    assert_eq!(e.status, 400);
    assert_eq!(e.body["code"], "invalid_request");
    assert_eq!(e.body["parameter"], "start");
    assert_eq!(e.trace_header.as_deref(), e.body["trace_id"].as_str());

    let err = pm()
        .data()
        .list_prices_history(token)
        .start(DateTime::<Utc>::UNIX_EPOCH)
        .end(Utc::now())
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
}

/// `condition` plus `event_id` on `/v2/positions`: the spec says `event_id` is
/// "user-anchored only" and `condition` narrows a user's positions, but live rejects the
/// pair. The SDK forwards it (it validates the pair on trades and activity), so the
/// server's typed error is what surfaces.
#[tokio::test]
#[ignore = "live network"]
async fn list_positions_condition_with_event_id_is_rejected_live() {
    let s = sample().await;
    let err = pm()
        .data()
        .list_positions()
        .user(s.user.as_str())
        .conditions([s.condition_id.as_str()])
        .event_ids([s.event_id.as_str()])
        .send()
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 400);
    assert_eq!(ErrorCode::from_error(&err), Some(ErrorCode::InvalidRequest));
    assert!(err.trace_id().is_some());
}

/// Violations of documented formats are caught before any request is sent.
#[tokio::test]
#[ignore = "live network"]
async fn sdk_client_side_validation() {
    let client = pm();
    let data = client.data();
    let err = data.get_approvals("nope").await.unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
    let err = data
        .get_open_interest()
        .conditions(["0x12"])
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
    let err = data.list_trades().limit(5000).send().await.unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
}
