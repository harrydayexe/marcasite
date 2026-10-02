//! Boards: `/v2/leaderboard`, `/v2/biggest-winners`, `/v2/builders/leaderboard`,
//! `/v2/builders/volume`.

use crate::Paginated;
use chrono::{DateTime, NaiveDate, Utc};
use polyoxide_core::{Query, Result, serde_util};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    DataClient,
    types::{Page, TimePeriod, check_limit, check_user, page_stream},
};
use crate::types::{Address, ConditionId, EventId, TokenId};

const LEADERBOARD: &[&str] = &["v2", "leaderboard"];
const BIGGEST_WINNERS: &[&str] = &["v2", "biggest-winners"];
const BUILDERS_LEADERBOARD: &[&str] = &["v2", "builders", "leaderboard"];

/// Maximum `limit` of the paginated boards.
const MAX_BOARD_LIMIT: u32 = 1000;
/// Maximum `limit` (in buckets) of `/v2/builders/volume`.
const MAX_BUILDERS_VOLUME_LIMIT: u32 = 90;

polyoxide_core::string_id! {
    /// A builder's stable identifier (`builder_code`) on the builder boards.
    ///
    /// The Data API documents it only as a string ("Stable identifier of the builder"),
    /// with no format, so it is kept exactly as received and never validated. It is a
    /// separate type from the CLOB's `BuilderCode` (documented as `0x` followed by 64 hex
    /// characters) because the Data API does not document the same wire format.
    pub struct BuilderCode;
}

polyoxide_core::string_enum! {
    /// Which trader leaderboard to read (`sort_by`).
    pub enum LeaderboardSortBy {
        /// Ranked by PnL (default).
        Pnl => "PNL",
        /// Ranked by volume.
        Volume => "VOLUME",
    }
}

/// One trader-leaderboard row (`components/schemas/LeaderboardEntry`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LeaderboardEntry {
    /// Competition rank: tied users share a rank and the next one skips. Page with the
    /// cursor; never derive a page from a rank.
    pub rank: u32,
    /// The ranked wallet.
    pub user_id: Address,
    /// Window PnL in USDC: the marked equity change net of flows for finite windows, the
    /// realized-only lifetime ledger for [`TimePeriod::All`].
    #[serde(with = "serde_util::decimal_number")]
    pub pnl: Decimal,
    /// Both-sides traded volume, in shares.
    #[serde(with = "serde_util::decimal_number")]
    pub volume: Decimal,
    /// Profile display name of the wallet.
    pub user_name: String,
    /// Profile image URL.
    pub profile_image: String,
    /// Linked X handle, when one exists.
    pub x_username: String,
    /// Profile verification badge.
    pub verified: bool,
}

/// One wallet's standing on both trader boards
/// (`components/schemas/LeaderboardUserEntry`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LeaderboardUserEntry {
    /// The looked-up wallet.
    pub user_id: Address,
    /// Window PnL in USDC (same semantics as [`LeaderboardEntry::pnl`]).
    #[serde(with = "serde_util::decimal_number")]
    pub pnl: Decimal,
    /// Both-sides traded volume, in shares.
    #[serde(with = "serde_util::decimal_number")]
    pub volume: Decimal,
    /// Profile display name of the wallet.
    pub user_name: String,
    /// Profile image URL.
    pub profile_image: String,
    /// Linked X handle, when one exists.
    pub x_username: String,
    /// Profile verification badge.
    pub verified: bool,
    /// Rank on the PnL board, exposed as served.
    ///
    /// The docs give two readings of "unranked": the schema says a `null` rank (`None`)
    /// means unranked on that board, while the `/v2/leaderboard` description says a rank
    /// of `0` on this arm means unranked, not first. Treat both `None` and `Some(0)` as
    /// unranked.
    pub rank_pnl: Option<u32>,
    /// Rank on the volume board, exposed as served; `None` and `Some(0)` both mean
    /// unranked (see [`rank_pnl`](Self::rank_pnl)).
    pub rank_volume: Option<u32>,
}

polyoxide_core::string_enum! {
    /// Kind of a biggest-winner row (`kind`).
    pub enum WinKind {
        /// A single-market position.
        Market => "market",
        /// A combo position: no Gamma event (`event_id` is `0`, `event_slug` is empty).
        Combo => "combo",
    }
}

/// One winning position (`components/schemas/BiggestWinner`). One row per position, not
/// per user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BiggestWinner {
    /// Unique 1-based ordinal within the window and category (equal PnL does not share a
    /// rank).
    pub win_rank: u32,
    /// Whether this is a market or a combo position; check it before building an event
    /// link.
    pub kind: WinKind,
    /// The winning wallet.
    pub user_id: Address,
    /// `final_value - initial_value`, in USDC.
    #[serde(with = "serde_util::decimal_number")]
    pub pnl: Decimal,
    /// Cost basis of the winning position, in USDC.
    #[serde(with = "serde_util::decimal_number")]
    pub initial_value: Decimal,
    /// Value at resolution, in USDC.
    #[serde(with = "serde_util::decimal_number")]
    pub final_value: Decimal,
    /// When the position resolved.
    #[serde(with = "serde_util::timestamp_seconds")]
    pub resolved_at: DateTime<Utc>,
    /// On-chain condition id of the market (the combo condition on combo rows).
    pub condition_id: ConditionId,
    /// Token id of the winning position.
    pub position_id: TokenId,
    /// Gamma event id of the parent event. Served as a JSON integer here (unlike the
    /// string event ids of the other routes), and serialized back as one.
    ///
    /// Combo rows have no Gamma event and carry the sentinel `0`, kept as served: branch
    /// on [`kind`](Self::kind) before building an event link.
    #[serde(with = "serde_util::integer_id")]
    pub event_id: EventId,
    /// Parent event slug; empty on combo rows.
    pub event_slug: String,
    /// Parent event title; on combo rows, the `" / "`-joined leg questions.
    pub event_title: String,
    /// Profile display name of the wallet.
    pub user_name: String,
    /// Profile image URL.
    pub profile_image: String,
}

/// One builder's standing for the window (`components/schemas/BuilderStanding`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BuilderStanding {
    /// Board rank; ties share a rank and the next one skips.
    pub rank: u64,
    /// Display name, falling back to `builder_code`. Cosmetic: key on
    /// [`builder_code`](Self::builder_code).
    pub builder_name: String,
    /// Stable identifier of the builder.
    pub builder_code: BuilderCode,
    /// Builder profile image URL.
    pub profile_image: String,
    /// Whether the builder is verified.
    pub verified: bool,
    /// Volume attributed to the builder in the window, in shares.
    #[serde(with = "serde_util::decimal_number")]
    pub volume: Decimal,
    /// Distinct active users (makers) attributed to the builder in the window.
    pub active_users: u64,
}

/// A builder's volume in one bucket of the series
/// (`components/schemas/BuilderVolumePoint`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BuilderVolumePoint {
    /// Bucket start date (UTC); the request's interval sets the width.
    pub date: NaiveDate,
    /// The builder's rank within that bucket.
    pub rank: u64,
    /// Display name, falling back to `builder_code`. Cosmetic: key on
    /// [`builder_code`](Self::builder_code).
    pub builder_name: String,
    /// Stable identifier of the builder.
    pub builder_code: BuilderCode,
    /// Builder profile image URL.
    pub profile_image: String,
    /// Whether the builder is verified.
    pub verified: bool,
    /// Volume attributed in that bucket, in shares.
    #[serde(with = "serde_util::decimal_number")]
    pub volume: Decimal,
    /// Distinct active users attributed in that bucket.
    pub active_users: u64,
}

impl DataClient {
    /// Lists the trader leaderboard of PnL or volume (`GET /v2/leaderboard`,
    /// cursor-paginated). For one wallet's standing use
    /// [`get_leaderboard_standing`](Self::get_leaderboard_standing), the `user=` arm of the
    /// same route.
    ///
    /// See <https://docs.polymarket.com/api-reference/boards/get-the-trader-leaderboard>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use polyoxide::data::{DataClient, LeaderboardSortBy, TimePeriod};
    ///
    /// let data = DataClient::new()?;
    /// let page = data
    ///     .list_leaderboard()
    ///     .time_period(TimePeriod::Week)
    ///     .sort_by(LeaderboardSortBy::Volume)
    ///     .limit(25)
    ///     .send()
    ///     .await?;
    /// for entry in page.items() {
    ///     let _ = (entry.rank, &entry.user_name, entry.volume);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_leaderboard(&self) -> ListLeaderboard {
        ListLeaderboard {
            client: self.clone(),
            time_period: None,
            category: None,
            sort_by: None,
            limit: None,
            cursor: None,
        }
    }

    /// Looks up one wallet's standing on both trader boards
    /// (`GET /v2/leaderboard?user=`). For the board itself use
    /// [`list_leaderboard`](Self::list_leaderboard), the other arm of the same route.
    ///
    /// See <https://docs.polymarket.com/api-reference/boards/get-the-trader-leaderboard>.
    pub fn get_leaderboard_standing(&self, user: impl Into<Address>) -> GetLeaderboardStanding {
        GetLeaderboardStanding {
            client: self.clone(),
            user: user.into(),
            time_period: None,
            category: None,
        }
    }

    /// Lists the biggest single winning positions (`GET /v2/biggest-winners`,
    /// cursor-paginated).
    ///
    /// See <https://docs.polymarket.com/api-reference/boards/list-the-biggest-wins>.
    pub fn list_biggest_winners(&self) -> ListBiggestWinners {
        ListBiggestWinners {
            client: self.clone(),
            time_period: None,
            category: None,
            limit: None,
            cursor: None,
        }
    }

    /// Lists the builders leaderboard, ranked by volume
    /// (`GET /v2/builders/leaderboard`, cursor-paginated).
    ///
    /// See <https://docs.polymarket.com/api-reference/boards/get-the-builders-leaderboard>.
    pub fn list_builders_leaderboard(&self) -> ListBuildersLeaderboard {
        ListBuildersLeaderboard {
            client: self.clone(),
            time_period: None,
            limit: None,
            cursor: None,
        }
    }

    /// Gets the per-builder volume time series, newest bucket first
    /// (`GET /v2/builders/volume`).
    ///
    /// See <https://docs.polymarket.com/api-reference/boards/get-builder-volume-over-time>.
    pub fn get_builders_volume(&self) -> GetBuildersVolume {
        GetBuildersVolume {
            client: self.clone(),
            interval: None,
            limit: None,
        }
    }
}

/// Request builder for [`DataClient::list_leaderboard`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListLeaderboard {
    client: DataClient,
    time_period: Option<TimePeriod>,
    category: Option<String>,
    sort_by: Option<LeaderboardSortBy>,
    limit: Option<u32>,
    cursor: Option<String>,
}

impl ListLeaderboard {
    /// The window (`time_period`, default [`TimePeriod::Day`]).
    pub fn time_period(mut self, time_period: TimePeriod) -> Self {
        self.time_period = Some(time_period);
        self
    }

    /// The category (`category`): `overall` (default), a Gamma market category (e.g.
    /// `sports`), or the synthetic `combos` (PnL board only) or `esports`.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Which board (`sort_by`, default [`LeaderboardSortBy::Pnl`]).
    pub fn sort_by(mut self, sort_by: LeaderboardSortBy) -> Self {
        self.sort_by = Some(sort_by);
        self
    }

    /// First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
    /// supplied (the cursor's own page size wins).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Resumes from a previous page's [`next_cursor`](Page::next_cursor) (`cursor`). The
    /// cursor pins the board (sort, window, category) it was minted on.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        check_limit(self.limit, MAX_BOARD_LIMIT)?;
        let mut q = Query::new();
        q.push_opt("time_period", self.time_period.as_ref())
            .push_opt("category", self.category.as_deref())
            .push_opt("sort_by", self.sort_by.as_ref())
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor);
        Ok(q)
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if `limit` is above 1000;
    /// otherwise see [`Error`](crate::Error) (a parameter contradicting the cursor's
    /// board is an [`Error::Api`](crate::Error::Api) with status `400`).
    pub async fn send(self) -> Result<Page<LeaderboardEntry>> {
        let query = self.query(self.cursor.as_deref())?;
        self.client.fetch_page(LEADERBOARD, query).await
    }

    /// Streams every row from the configured cursor onwards, fetching pages lazily.
    ///
    /// Every page restates the same board parameters with the cursor (allowed, since they
    /// agree with the board the cursor pins). The stream ends when the server reports
    /// no further page.
    pub fn into_stream(self) -> Paginated<LeaderboardEntry> {
        let client = self.client.clone();
        let start = self.cursor.clone();
        page_stream(client, LEADERBOARD, start, move |cursor| self.query(cursor))
    }
}

/// Request builder for [`DataClient::get_leaderboard_standing`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetLeaderboardStanding {
    client: DataClient,
    user: Address,
    time_period: Option<TimePeriod>,
    category: Option<String>,
}

impl GetLeaderboardStanding {
    /// The window (`time_period`, default [`TimePeriod::Day`]).
    pub fn time_period(mut self, time_period: TimePeriod) -> Self {
        self.time_period = Some(time_period);
        self
    }

    /// The category (`category`): `overall` (default), a Gamma market category, or the
    /// synthetic `combos` or `esports`.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Sends the request. Returns `Ok(None)` when the API answers `data: null`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if `user` is empty (an empty
    /// `user` would select the board arm of the route instead); an invalid `time_period`
    /// or a known protocol contract address is an [`Error::Api`](crate::Error::Api) with
    /// status `400`; otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Option<LeaderboardUserEntry>> {
        check_user(&self.user)?;
        let mut query = Query::new();
        query
            .push_opt("time_period", self.time_period.as_ref())
            .push_opt("category", self.category.as_deref())
            .push("user", &self.user);
        self.client.fetch_data(LEADERBOARD, query).await
    }
}

/// Request builder for [`DataClient::list_biggest_winners`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListBiggestWinners {
    client: DataClient,
    time_period: Option<TimePeriod>,
    category: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
}

impl ListBiggestWinners {
    /// Window on the resolution time (`time_period`, default [`TimePeriod::Day`]).
    pub fn time_period(mut self, time_period: TimePeriod) -> Self {
        self.time_period = Some(time_period);
        self
    }

    /// The category (`category`): `overall` (default), a Gamma market category, or the
    /// synthetic `combos` or `esports`.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
    /// supplied (the cursor's own page size wins).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Resumes from a previous page's [`next_cursor`](Page::next_cursor) (`cursor`). The
    /// cursor pins the window and category it was minted on.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        check_limit(self.limit, MAX_BOARD_LIMIT)?;
        let mut q = Query::new();
        q.push_opt("time_period", self.time_period.as_ref())
            .push_opt("category", self.category.as_deref())
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor);
        Ok(q)
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if `limit` is above 1000;
    /// otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Page<BiggestWinner>> {
        let query = self.query(self.cursor.as_deref())?;
        self.client.fetch_page(BIGGEST_WINNERS, query).await
    }

    /// Streams every row from the configured cursor onwards, fetching pages lazily.
    ///
    /// Every page restates the same window and category with the cursor. The stream ends
    /// when the server reports no further page.
    pub fn into_stream(self) -> Paginated<BiggestWinner> {
        let client = self.client.clone();
        let start = self.cursor.clone();
        page_stream(client, BIGGEST_WINNERS, start, move |cursor| {
            self.query(cursor)
        })
    }
}

/// Request builder for [`DataClient::list_builders_leaderboard`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListBuildersLeaderboard {
    client: DataClient,
    time_period: Option<TimePeriod>,
    limit: Option<u32>,
    cursor: Option<String>,
}

impl ListBuildersLeaderboard {
    /// The window (`time_period`, default [`TimePeriod::Day`]).
    pub fn time_period(mut self, time_period: TimePeriod) -> Self {
        self.time_period = Some(time_period);
        self
    }

    /// First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
    /// supplied (the cursor's own page size wins).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Resumes from a previous page's [`next_cursor`](Page::next_cursor) (`cursor`). The
    /// cursor pins the window it was minted on.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        check_limit(self.limit, MAX_BOARD_LIMIT)?;
        let mut q = Query::new();
        q.push_opt("time_period", self.time_period.as_ref())
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor);
        Ok(q)
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if `limit` is above 1000;
    /// otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Page<BuilderStanding>> {
        let query = self.query(self.cursor.as_deref())?;
        self.client.fetch_page(BUILDERS_LEADERBOARD, query).await
    }

    /// Streams every row from the configured cursor onwards, fetching pages lazily.
    ///
    /// Every page restates the same window with the cursor. The stream ends when
    /// the server reports no further page.
    pub fn into_stream(self) -> Paginated<BuilderStanding> {
        let client = self.client.clone();
        let start = self.cursor.clone();
        page_stream(client, BUILDERS_LEADERBOARD, start, move |cursor| {
            self.query(cursor)
        })
    }
}

/// Request builder for [`DataClient::get_builders_volume`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetBuildersVolume {
    client: DataClient,
    interval: Option<TimePeriod>,
    limit: Option<u32>,
}

impl GetBuildersVolume {
    /// Bucket width (`interval`, default [`TimePeriod::Day`]; [`TimePeriod::All`] buckets
    /// by calendar year).
    pub fn interval(mut self, interval: TimePeriod) -> Self {
        self.interval = Some(interval);
        self
    }

    /// How many of the most recent **buckets** to return (`limit`, default 30, at most
    /// 90).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if `limit` is above 90;
    /// otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<BuilderVolumePoint>> {
        check_limit(self.limit, MAX_BUILDERS_VOLUME_LIMIT)?;
        let mut query = Query::new();
        query
            .push_opt("interval", self.interval.as_ref())
            .push_opt("limit", self.limit);
        self.client
            .fetch_data(&["v2", "builders", "volume"], query)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::types::Envelope;
    use polyoxide_core::Error;

    /// Field names and types from `components/schemas/LeaderboardEntry`.
    #[test]
    fn deserializes_leaderboard_entry() {
        let json = r#"{
            "rank": 1,
            "user_id": "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748",
            "pnl": 12345.67,
            "volume": 99999.5,
            "user_name": "whale",
            "profile_image": "",
            "x_username": "",
            "verified": true
        }"#;
        let entry: LeaderboardEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.rank, 1);
        assert_eq!(entry.pnl.to_string(), "12345.67");
        let value = serde_json::to_value(&entry).unwrap();
        assert_eq!(value["pnl"], serde_json::json!(12345.67));
    }

    /// `components/schemas/Envelope_Option_LeaderboardUserEntry`: `data` may be `null`,
    /// and ranks are exposed as served (`null`, or `0` per the endpoint description).
    #[test]
    fn deserializes_leaderboard_standing() {
        let none: Envelope<Option<LeaderboardUserEntry>> =
            serde_json::from_str(r#"{"data":null}"#).unwrap();
        assert_eq!(none.data, None);
        let json = r#"{"data":{
            "user_id": "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748",
            "pnl": -5,
            "volume": 10,
            "user_name": "",
            "profile_image": "",
            "x_username": "",
            "verified": false,
            "rank_pnl": null,
            "rank_volume": 0
        }}"#;
        let entry = serde_json::from_str::<Envelope<Option<LeaderboardUserEntry>>>(json)
            .unwrap()
            .data
            .unwrap();
        assert_eq!(entry.rank_pnl, None);
        assert_eq!(entry.rank_volume, Some(0));
    }

    /// Field names and types from `components/schemas/BiggestWinner`: a combo row carries
    /// the combo condition, a `' / '`-joined leg title, `event_id` `0` and an empty
    /// `event_slug`.
    #[test]
    fn deserializes_biggest_winner() {
        let json = r#"{
            "win_rank": 1,
            "kind": "combo",
            "user_id": "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748",
            "pnl": 900,
            "initial_value": 100,
            "final_value": 1000,
            "resolved_at": 1787133600,
            "condition_id": "0x03aa000000000000000000000000000000000000000000000000000000000001",
            "position_id": "123",
            "event_id": 0,
            "event_slug": "",
            "event_title": "Will A win? / Will B win?",
            "user_name": "",
            "profile_image": ""
        }"#;
        let row: BiggestWinner = serde_json::from_str(json).unwrap();
        assert_eq!(row.kind, WinKind::Combo);
        assert_eq!(row.event_id, EventId::from("0"));
        assert_eq!(row.resolved_at.timestamp(), 1_787_133_600);
        // The integer id and the amounts serialize back as JSON numbers.
        let value = serde_json::to_value(&row).unwrap();
        assert_eq!(value["event_id"], serde_json::json!(0));
        assert_eq!(value["pnl"], serde_json::json!(900));
        assert_eq!(value["resolved_at"], serde_json::json!(1_787_133_600));
        assert_eq!(serde_json::from_value::<BiggestWinner>(value).unwrap(), row);
    }

    /// Field names and types from `components/schemas/BuilderStanding` and
    /// `BuilderVolumePoint`.
    #[test]
    fn deserializes_builders() {
        let standing: BuilderStanding = serde_json::from_str(
            r#"{"rank":2,"builder_name":"acme","builder_code":"acme","profile_image":"",
                "verified":false,"volume":1000.5,"active_users":7}"#,
        )
        .unwrap();
        assert_eq!(standing.builder_code, BuilderCode::from("acme"));
        assert_eq!(standing.active_users, 7);

        let points: Envelope<Vec<BuilderVolumePoint>> = serde_json::from_str(
            r#"{"data":[{"date":"2026-08-19","rank":1,"builder_name":"acme","builder_code":"acme",
                "profile_image":"","verified":true,"volume":5,"active_users":1}]}"#,
        )
        .unwrap();
        let point = points.data.first().unwrap();
        assert_eq!(point.date, NaiveDate::from_ymd_opt(2026, 8, 19).unwrap());
        assert_eq!(serde_json::to_value(point).unwrap()["volume"], 5);
    }

    #[test]
    fn boards_query() {
        let client = DataClient::new().unwrap();
        assert!(matches!(
            client.list_leaderboard().limit(1001).query(None),
            Err(Error::Validation(v)) if v.parameter() == "limit"
        ));
        let q = client
            .list_leaderboard()
            .time_period(TimePeriod::Month)
            .category("sports")
            .sort_by(LeaderboardSortBy::Pnl)
            .query(Some("c"))
            .unwrap();
        assert_eq!(
            q.to_string(),
            "time_period=month&category=sports&sort_by=PNL&cursor=c"
        );
    }
}
