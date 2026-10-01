//! Markets: `/markets`, `/markets/{id}`, `/markets/slug/{slug}`, `/markets/{id}/tags`,
//! `/markets/{id}/description`, `/markets/keyset`, `/markets/information` and
//! `/markets/abridged`.

use chrono::{DateTime, Utc};
use futures_core::Stream;
use polyoxide_core::{
    Query, Result,
    pagination::{CursorPage, cursor_stream, offset_stream},
    serde_util,
    types::{ConditionId, MarketId, QuestionId, TokenId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    Category, Event, GammaClient, ImageOptimization, Tag, TagId,
    util::{Lookup, integer_id, rfc3339, setters, validate_keyset_limit},
};

/// A market (`components/schemas/Market`).
///
/// Every field is optional because the spec marks none as required. Fields the spec types
/// as `number` are [`Decimal`]s; string-typed amounts (`liquidity`, `volume`, `fee`,
/// `umaBond`, `umaReward`) are parsed into [`Decimal`]s too. Other string fields are kept
/// exactly as sent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Market {
    /// Market id.
    pub id: Option<MarketId>,
    /// The market question.
    pub question: Option<String>,
    /// On-chain condition id.
    pub condition_id: Option<ConditionId>,
    /// URL slug.
    pub slug: Option<String>,
    /// Twitter card image URL.
    pub twitter_card_image: Option<String>,
    /// Resolution source.
    pub resolution_source: Option<String>,
    /// End date.
    #[serde(default, with = "serde_util::datetime_option")]
    pub end_date: Option<DateTime<Utc>>,
    /// Category.
    pub category: Option<String>,
    /// Liquidity (a string on the wire).
    pub liquidity: Option<Decimal>,
    /// Sponsor name.
    pub sponsor_name: Option<String>,
    /// Sponsor image URL.
    pub sponsor_image: Option<String>,
    /// Start date.
    #[serde(default, with = "serde_util::datetime_option")]
    pub start_date: Option<DateTime<Utc>>,
    /// X-axis value.
    pub x_axis_value: Option<String>,
    /// Y-axis value.
    pub y_axis_value: Option<String>,
    /// Denomination token.
    pub denomination_token: Option<String>,
    /// Fee (a string on the wire).
    pub fee: Option<Decimal>,
    /// Image URL.
    pub image: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Lower bound. The spec types this as a plain string.
    pub lower_bound: Option<String>,
    /// Upper bound. The spec types this as a plain string.
    pub upper_bound: Option<String>,
    /// Description, including the resolution criteria.
    pub description: Option<String>,
    /// Outcomes. The spec types this as a plain string and documents no encoding, so it is
    /// kept exactly as sent.
    pub outcomes: Option<String>,
    /// Outcome prices. The spec types this as a plain string and documents no encoding, so
    /// it is kept exactly as sent.
    pub outcome_prices: Option<String>,
    /// Volume (a string on the wire).
    pub volume: Option<Decimal>,
    /// Whether the market is active.
    pub active: Option<bool>,
    /// Market type.
    pub market_type: Option<String>,
    /// Format type.
    pub format_type: Option<String>,
    /// Lower bound date. The spec types this as a plain string with no format.
    pub lower_bound_date: Option<String>,
    /// Upper bound date. The spec types this as a plain string with no format.
    pub upper_bound_date: Option<String>,
    /// Whether the market is closed.
    pub closed: Option<bool>,
    /// Id of the user who created the market.
    pub created_by: Option<i64>,
    /// Id of the user who last updated the market.
    pub updated_by: Option<i64>,
    /// Creation time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub updated_at: Option<DateTime<Utc>>,
    /// Close time. The spec types this as a plain string with no format.
    pub closed_time: Option<String>,
    /// Whether the market uses the wide format.
    pub wide_format: Option<bool>,
    /// Whether the market is flagged as new.
    pub new: Option<bool>,
    /// Mailchimp tag.
    pub mailchimp_tag: Option<String>,
    /// Whether the market is featured.
    pub featured: Option<bool>,
    /// Whether the market is archived.
    pub archived: Option<bool>,
    /// Resolver.
    pub resolved_by: Option<String>,
    /// Whether the market is restricted.
    pub restricted: Option<bool>,
    /// Market group.
    pub market_group: Option<i64>,
    /// Title of this market within its group.
    pub group_item_title: Option<String>,
    /// Threshold of this market within its group. The spec types this as a plain string.
    pub group_item_threshold: Option<String>,
    /// Question id (wire name `questionID`).
    #[serde(rename = "questionID")]
    pub question_id: Option<QuestionId>,
    /// UMA end date. The spec types this as a plain string with no format.
    pub uma_end_date: Option<String>,
    /// Whether the order book is enabled.
    pub enable_order_book: Option<bool>,
    /// Minimum price tick size.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub order_price_min_tick_size: Option<Decimal>,
    /// Minimum order size.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub order_min_size: Option<Decimal>,
    /// UMA resolution status.
    pub uma_resolution_status: Option<String>,
    /// Curation order.
    pub curation_order: Option<i64>,
    /// Volume as a number.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_num: Option<Decimal>,
    /// Liquidity as a number.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub liquidity_num: Option<Decimal>,
    /// End date. The spec types this as a plain string with no format.
    pub end_date_iso: Option<String>,
    /// Start date. The spec types this as a plain string with no format.
    pub start_date_iso: Option<String>,
    /// UMA end date. The spec types this as a plain string with no format.
    pub uma_end_date_iso: Option<String>,
    /// Whether the dates have been reviewed.
    pub has_reviewed_dates: Option<bool>,
    /// Whether the market is ready for cron processing.
    pub ready_for_cron: Option<bool>,
    /// Whether comments are enabled.
    pub comments_enabled: Option<bool>,
    /// 24-hour volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_24hr: Option<Decimal>,
    /// 1-week volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_1wk: Option<Decimal>,
    /// 1-month volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_1mo: Option<Decimal>,
    /// 1-year volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_1yr: Option<Decimal>,
    /// Game start time. The spec types this as a plain string with no format.
    pub game_start_time: Option<String>,
    /// Seconds delay.
    pub seconds_delay: Option<i64>,
    /// CLOB token ids. The spec types this as a plain string and documents no encoding, so
    /// it is kept exactly as sent.
    pub clob_token_ids: Option<String>,
    /// Disqus thread.
    pub disqus_thread: Option<String>,
    /// Short outcomes. The spec types this as a plain string.
    pub short_outcomes: Option<String>,
    /// Team A id (wire name `teamAID`).
    #[serde(rename = "teamAID")]
    pub team_a_id: Option<String>,
    /// Team B id (wire name `teamBID`).
    #[serde(rename = "teamBID")]
    pub team_b_id: Option<String>,
    /// UMA bond (a string on the wire).
    pub uma_bond: Option<Decimal>,
    /// UMA reward (a string on the wire).
    pub uma_reward: Option<Decimal>,
    /// 24-hour CLOB volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_24hr_clob: Option<Decimal>,
    /// 1-week CLOB volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_1wk_clob: Option<Decimal>,
    /// 1-month CLOB volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_1mo_clob: Option<Decimal>,
    /// 1-year CLOB volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_1yr_clob: Option<Decimal>,
    /// CLOB volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_clob: Option<Decimal>,
    /// CLOB liquidity.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub liquidity_clob: Option<Decimal>,
    /// Maker base fee.
    pub maker_base_fee: Option<i64>,
    /// Taker base fee.
    pub taker_base_fee: Option<i64>,
    /// Custom liveness.
    pub custom_liveness: Option<i64>,
    /// Whether the market accepts orders.
    pub accepting_orders: Option<bool>,
    /// Whether notifications are enabled.
    pub notifications_enabled: Option<bool>,
    /// Score.
    pub score: Option<i64>,
    /// Optimized image metadata.
    pub image_optimized: Option<ImageOptimization>,
    /// Optimized icon metadata.
    pub icon_optimized: Option<ImageOptimization>,
    /// Events the market belongs to.
    pub events: Option<Vec<Event>>,
    /// Categories.
    pub categories: Option<Vec<Category>>,
    /// Tags (included by some endpoints only, e.g. with `include_tag=true`).
    pub tags: Option<Vec<Tag>>,
    /// Creator.
    pub creator: Option<String>,
    /// Whether the market is ready.
    pub ready: Option<bool>,
    /// Whether the market is funded.
    pub funded: Option<bool>,
    /// Past slugs. The spec types this as a plain string.
    pub past_slugs: Option<String>,
    /// When the market became ready.
    #[serde(default, with = "serde_util::datetime_option")]
    pub ready_timestamp: Option<DateTime<Utc>>,
    /// When the market was funded.
    #[serde(default, with = "serde_util::datetime_option")]
    pub funded_timestamp: Option<DateTime<Utc>>,
    /// When the market started accepting orders.
    #[serde(default, with = "serde_util::datetime_option")]
    pub accepting_orders_timestamp: Option<DateTime<Utc>>,
    /// Competitiveness score.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub competitive: Option<Decimal>,
    /// Minimum size for liquidity rewards.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rewards_min_size: Option<Decimal>,
    /// Maximum spread for liquidity rewards.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rewards_max_spread: Option<Decimal>,
    /// Current spread.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub spread: Option<Decimal>,
    /// Whether the market resolves automatically.
    pub automatically_resolved: Option<bool>,
    /// Price change over one day.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub one_day_price_change: Option<Decimal>,
    /// Price change over one hour.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub one_hour_price_change: Option<Decimal>,
    /// Price change over one week.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub one_week_price_change: Option<Decimal>,
    /// Price change over one month.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub one_month_price_change: Option<Decimal>,
    /// Price change over one year.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub one_year_price_change: Option<Decimal>,
    /// Last trade price.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub last_trade_price: Option<Decimal>,
    /// Best bid.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub best_bid: Option<Decimal>,
    /// Best ask.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub best_ask: Option<Decimal>,
    /// Whether the market activates automatically.
    pub automatically_active: Option<bool>,
    /// Whether the book is cleared on start.
    pub clear_book_on_start: Option<bool>,
    /// Chart colour.
    pub chart_color: Option<String>,
    /// Series colour.
    pub series_color: Option<String>,
    /// Whether to show the GMP series.
    pub show_gmp_series: Option<bool>,
    /// Whether to show the GMP outcome.
    pub show_gmp_outcome: Option<bool>,
    /// Whether the market is activated manually.
    pub manual_activation: Option<bool>,
    /// Whether this is the "other" market of a negative-risk event.
    pub neg_risk_other: Option<bool>,
    /// Game id.
    pub game_id: Option<String>,
    /// Range of this market within its group.
    pub group_item_range: Option<String>,
    /// Sports market type (see
    /// [`GammaClient::get_sports_market_types`](super::GammaClient::get_sports_market_types)).
    pub sports_market_type: Option<String>,
    /// Line (for sports markets).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub line: Option<Decimal>,
    /// UMA resolution statuses. The spec types this as a plain string.
    pub uma_resolution_statuses: Option<String>,
    /// Whether deployment is pending.
    pub pending_deployment: Option<bool>,
    /// Whether the market is being deployed.
    pub deploying: Option<bool>,
    /// When deployment started.
    #[serde(default, with = "serde_util::datetime_option")]
    pub deploying_timestamp: Option<DateTime<Utc>>,
    /// When deployment is scheduled.
    #[serde(default, with = "serde_util::datetime_option")]
    pub scheduled_deployment_timestamp: Option<DateTime<Utc>>,
    /// Whether RFQ is enabled.
    pub rfq_enabled: Option<bool>,
    /// Event start time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub event_start_time: Option<DateTime<Utc>>,
    /// Whether fees are enabled.
    pub fees_enabled: Option<bool>,
    /// Fee schedule.
    pub fee_schedule: Option<FeeSchedule>,
}

/// A market's fee schedule (`components/schemas/FeeSchedule`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FeeSchedule {
    /// Exponent.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub exponent: Option<Decimal>,
    /// Rate.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rate: Option<Decimal>,
    /// Whether only takers pay the fee.
    pub taker_only: Option<bool>,
    /// Rebate rate.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rebate_rate: Option<Decimal>,
}

/// A market's description (`components/schemas/MarketDescription`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MarketDescription {
    /// The description.
    pub description: Option<String>,
}

/// One page of [`GammaClient::list_markets_keyset`] (`components/schemas/KeysetMarketsResponse`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketsKeysetPage {
    /// The markets on this page (empty if none were found).
    pub markets: Option<Vec<Market>>,
    /// Cursor for the next page, passed as
    /// [`after_cursor`](ListMarketsKeyset::after_cursor). Present only when the page is
    /// full; absent on the last page.
    pub next_cursor: Option<String>,
}

impl GammaClient {
    /// Lists markets (offset pagination).
    ///
    /// The server only returns open markets unless [`closed`](ListMarkets::closed) is set
    /// (the spec documents `closed` as defaulting to `false`).
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/list-markets>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let markets = gamma
    ///     .list_markets()
    ///     .limit(10)
    ///     .order("volume_num")
    ///     .ascending(false)
    ///     .send()
    ///     .await?;
    /// for market in markets {
    ///     println!("{:?}: {:?}", market.question, market.volume_num);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_markets(&self) -> ListMarkets {
        ListMarkets {
            client: self.clone(),
            params: ListMarketsParams::default(),
        }
    }

    /// Gets a market by id.
    ///
    /// Fails with [`Error::Api`](crate::Error::Api) (status `404`) if the market does not
    /// exist; see [`Error::is_not_found`](crate::Error::is_not_found).
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-market-by-id>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let market = gamma.get_market("239826").include_tag(true).send().await?;
    /// println!("{:?} closes at {:?}", market.question, market.end_date);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_market(&self, id: impl Into<MarketId>) -> GetMarket {
        GetMarket {
            client: self.clone(),
            lookup: Lookup::Id(id.into()),
            include_tag: None,
        }
    }

    /// Gets a market by slug.
    ///
    /// Fails with [`Error::Api`](crate::Error::Api) (status `404`) if the market does not
    /// exist.
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-market-by-slug>.
    pub fn get_market_by_slug(&self, slug: impl Into<String>) -> GetMarket {
        GetMarket {
            client: self.clone(),
            lookup: Lookup::Slug(slug.into()),
            include_tag: None,
        }
    }

    /// Gets the tags attached to a market.
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id>.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing market is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn get_market_tags(&self, id: impl Into<MarketId>) -> Result<Vec<Tag>> {
        let id = id.into();
        self.transport
            .get(&["markets", id.as_str(), "tags"])
            .send()
            .await
    }

    /// Gets a market's description.
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getMarketDescription` (no
    /// published doc page).
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing market is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn get_market_description(
        &self,
        id: impl Into<MarketId>,
    ) -> Result<MarketDescription> {
        let id = id.into();
        self.transport
            .get(&["markets", id.as_str(), "description"])
            .send()
            .await
    }

    /// Lists markets with cursor-based (keyset) pagination, for stable paging through large
    /// result sets.
    ///
    /// [`send`](ListMarketsKeyset::send) returns one [`MarketsKeysetPage`]; pass its
    /// `next_cursor` to [`after_cursor`](ListMarketsKeyset::after_cursor) for the next
    /// page, or use [`into_stream`](ListMarketsKeyset::into_stream) to walk every page.
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/list-markets-keyset-pagination>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use futures_util::{StreamExt as _, TryStreamExt as _};
    ///
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let markets: Vec<_> = gamma
    ///     .list_markets_keyset()
    ///     .limit(100)
    ///     .into_stream()
    ///     .take(250)
    ///     .try_collect()
    ///     .await?;
    /// # let _ = markets;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_markets_keyset(&self) -> ListMarketsKeyset {
        ListMarketsKeyset {
            client: self.clone(),
            params: ListMarketsKeysetParams::default(),
        }
    }

    /// Queries markets by information filters sent as a JSON body.
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getMarketsInformation` (no
    /// published doc page).
    pub fn get_markets_information(&self) -> GetMarketsInformation {
        GetMarketsInformation {
            client: self.clone(),
            endpoint: "information",
            params: MarketsInformationParams::default(),
        }
    }

    /// Queries abridged markets by information filters sent as a JSON body.
    ///
    /// Takes the same filters and returns the same [`Market`] schema as
    /// [`get_markets_information`](Self::get_markets_information).
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getAbridgedMarkets` (no published
    /// doc page).
    pub fn get_abridged_markets(&self) -> GetMarketsInformation {
        GetMarketsInformation {
            client: self.clone(),
            endpoint: "abridged",
            params: MarketsInformationParams::default(),
        }
    }
}

/// Request builder for [`GammaClient::list_markets`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListMarkets {
    client: GammaClient,
    params: ListMarketsParams,
}

#[derive(Debug, Clone, Default)]
struct ListMarketsParams {
    limit: Option<u64>,
    offset: Option<u64>,
    order: Option<String>,
    ascending: Option<bool>,
    id: Vec<MarketId>,
    slug: Vec<String>,
    clob_token_ids: Vec<TokenId>,
    condition_ids: Vec<ConditionId>,
    liquidity_num_min: Option<Decimal>,
    liquidity_num_max: Option<Decimal>,
    volume_num_min: Option<Decimal>,
    volume_num_max: Option<Decimal>,
    start_date_min: Option<DateTime<Utc>>,
    start_date_max: Option<DateTime<Utc>>,
    end_date_min: Option<DateTime<Utc>>,
    end_date_max: Option<DateTime<Utc>>,
    tag_id: Option<TagId>,
    related_tags: Option<bool>,
    cyom: Option<bool>,
    uma_resolution_status: Option<String>,
    game_id: Option<String>,
    sports_market_types: Vec<String>,
    rewards_min_size: Option<Decimal>,
    question_ids: Vec<QuestionId>,
    include_tag: Option<bool>,
    closed: Option<bool>,
}

impl ListMarketsParams {
    fn query(&self, offset: Option<u64>) -> Query {
        let mut q = Query::new();
        q.push_opt("limit", self.limit)
            .push_opt("offset", offset)
            .push_opt("order", self.order.as_deref())
            .push_opt("ascending", self.ascending)
            .push_all("id", &self.id)
            .push_all("slug", &self.slug)
            .push_all("clob_token_ids", &self.clob_token_ids)
            .push_all("condition_ids", &self.condition_ids)
            .push_opt("liquidity_num_min", self.liquidity_num_min)
            .push_opt("liquidity_num_max", self.liquidity_num_max)
            .push_opt("volume_num_min", self.volume_num_min)
            .push_opt("volume_num_max", self.volume_num_max)
            .push_opt("start_date_min", self.start_date_min.as_ref().map(rfc3339))
            .push_opt("start_date_max", self.start_date_max.as_ref().map(rfc3339))
            .push_opt("end_date_min", self.end_date_min.as_ref().map(rfc3339))
            .push_opt("end_date_max", self.end_date_max.as_ref().map(rfc3339))
            .push_opt("tag_id", self.tag_id.as_ref())
            .push_opt("related_tags", self.related_tags)
            .push_opt("cyom", self.cyom)
            .push_opt(
                "uma_resolution_status",
                self.uma_resolution_status.as_deref(),
            )
            .push_opt("game_id", self.game_id.as_deref())
            .push_all("sports_market_types", &self.sports_market_types)
            .push_opt("rewards_min_size", self.rewards_min_size)
            .push_all("question_ids", &self.question_ids)
            .push_opt("include_tag", self.include_tag)
            .push_opt("closed", self.closed);
        q
    }
}

impl ListMarkets {
    setters! {
        /// Maximum number of markets per page.
        limit: u64;
        /// Number of markets to skip.
        offset: u64;
        /// Comma-separated list of fields to order by.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`).
        ascending: bool;
        /// Only markets with these ids.
        id: many MarketId;
        /// Only markets with these slugs.
        slug: many String;
        /// Only markets with these CLOB token ids.
        clob_token_ids: many TokenId;
        /// Only markets with these condition ids.
        condition_ids: many ConditionId;
        /// Minimum liquidity.
        liquidity_num_min: into Decimal;
        /// Maximum liquidity.
        liquidity_num_max: into Decimal;
        /// Minimum volume.
        volume_num_min: into Decimal;
        /// Maximum volume.
        volume_num_max: into Decimal;
        /// Earliest start date.
        start_date_min: DateTime<Utc>;
        /// Latest start date.
        start_date_max: DateTime<Utc>;
        /// Earliest end date.
        end_date_min: DateTime<Utc>;
        /// Latest end date.
        end_date_max: DateTime<Utc>;
        /// Only markets with this tag.
        tag_id: into TagId;
        /// Include markets with tags related to [`tag_id`](Self::tag_id).
        related_tags: bool;
        /// Only "create your own market" markets (`true`) or only other markets (`false`).
        cyom: bool;
        /// Only markets with this UMA resolution status.
        uma_resolution_status: into String;
        /// Only markets for this game id.
        game_id: into String;
        /// Only markets with these sports market types.
        sports_market_types: many String;
        /// Minimum size for liquidity rewards.
        rewards_min_size: into Decimal;
        /// Only markets with these question ids.
        question_ids: many QuestionId;
        /// Include each market's tags.
        include_tag: bool;
        /// Only closed (`true`) or only open (`false`, the server default) markets.
        closed: bool;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Market>> {
        self.client
            .transport
            .get(&["markets"])
            .query(self.params.query(offset))
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Market>> {
        self.fetch(self.params.offset).await
    }

    /// Streams every market from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page. A page shorter than
    /// [`limit`](Self::limit) does not end it, because the server may cap the page size, so
    /// the last request returns an empty page.
    pub fn into_stream(self) -> impl Stream<Item = Result<Market>> + Send + 'static {
        let start = self.params.offset.unwrap_or(0);
        offset_stream(start, move |offset| {
            let request = self.clone();
            async move { request.fetch(Some(offset)).await }
        })
    }
}

/// Request builder for [`GammaClient::get_market`] and
/// [`GammaClient::get_market_by_slug`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetMarket {
    client: GammaClient,
    lookup: Lookup<MarketId>,
    include_tag: Option<bool>,
}

impl GetMarket {
    /// Include the market's tags.
    pub fn include_tag(mut self, include_tag: bool) -> Self {
        self.include_tag = Some(include_tag);
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing market is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn send(self) -> Result<Market> {
        let mut query = Query::new();
        query.push_opt("include_tag", self.include_tag);
        let transport = &self.client.transport;
        let request = match &self.lookup {
            Lookup::Id(id) => transport.get(&["markets", id.as_str()]),
            Lookup::Slug(slug) => transport.get(&["markets", "slug", slug.as_str()]),
        };
        request.query(query).send().await
    }
}

/// Request builder for [`GammaClient::list_markets_keyset`].
///
/// The endpoint rejects `offset`, so there is no setter for it; page with
/// [`after_cursor`](Self::after_cursor) instead.
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListMarketsKeyset {
    client: GammaClient,
    params: ListMarketsKeysetParams,
}

#[derive(Debug, Clone, Default)]
struct ListMarketsKeysetParams {
    limit: Option<u64>,
    order: Option<String>,
    ascending: Option<bool>,
    after_cursor: Option<String>,
    id: Vec<MarketId>,
    slug: Vec<String>,
    closed: Option<bool>,
    decimalized: Option<bool>,
    clob_token_ids: Vec<TokenId>,
    condition_ids: Vec<ConditionId>,
    question_ids: Vec<QuestionId>,
    liquidity_num_min: Option<Decimal>,
    liquidity_num_max: Option<Decimal>,
    volume_num_min: Option<Decimal>,
    volume_num_max: Option<Decimal>,
    start_date_min: Option<DateTime<Utc>>,
    start_date_max: Option<DateTime<Utc>>,
    end_date_min: Option<DateTime<Utc>>,
    end_date_max: Option<DateTime<Utc>>,
    tag_id: Vec<TagId>,
    related_tags: Option<bool>,
    tag_match: Option<String>,
    cyom: Option<bool>,
    rfq_enabled: Option<bool>,
    uma_resolution_status: Option<String>,
    game_id: Option<String>,
    sports_market_types: Vec<String>,
    include_tag: Option<bool>,
    locale: Option<String>,
}

impl ListMarketsKeysetParams {
    fn query(&self, after_cursor: Option<&str>) -> Query {
        let mut q = Query::new();
        q.push_opt("limit", self.limit)
            .push_opt("order", self.order.as_deref())
            .push_opt("ascending", self.ascending)
            .push_opt("after_cursor", after_cursor)
            .push_all("id", &self.id)
            .push_all("slug", &self.slug)
            .push_opt("closed", self.closed)
            .push_opt("decimalized", self.decimalized)
            .push_all("clob_token_ids", &self.clob_token_ids)
            .push_all("condition_ids", &self.condition_ids)
            .push_all("question_ids", &self.question_ids)
            .push_opt("liquidity_num_min", self.liquidity_num_min)
            .push_opt("liquidity_num_max", self.liquidity_num_max)
            .push_opt("volume_num_min", self.volume_num_min)
            .push_opt("volume_num_max", self.volume_num_max)
            .push_opt("start_date_min", self.start_date_min.as_ref().map(rfc3339))
            .push_opt("start_date_max", self.start_date_max.as_ref().map(rfc3339))
            .push_opt("end_date_min", self.end_date_min.as_ref().map(rfc3339))
            .push_opt("end_date_max", self.end_date_max.as_ref().map(rfc3339))
            .push_all("tag_id", &self.tag_id)
            .push_opt("related_tags", self.related_tags)
            .push_opt("tag_match", self.tag_match.as_deref())
            .push_opt("cyom", self.cyom)
            .push_opt("rfq_enabled", self.rfq_enabled)
            .push_opt(
                "uma_resolution_status",
                self.uma_resolution_status.as_deref(),
            )
            .push_opt("game_id", self.game_id.as_deref())
            .push_all("sports_market_types", &self.sports_market_types)
            .push_opt("include_tag", self.include_tag)
            .push_opt("locale", self.locale.as_deref());
        q
    }
}

impl ListMarketsKeyset {
    setters! {
        /// Maximum number of markets per page, between 1 and 100 (server default 20).
        /// Values outside that range are rejected client-side with
        /// [`Error::Validation`](crate::Error::Validation).
        limit: u64;
        /// Comma-separated list of JSON field names to order by, e.g.
        /// `volume_num,liquidity_num`.
        order: into String;
        /// Sort direction (server default ascending). Only used when
        /// [`order`](Self::order) is set.
        ascending: bool;
        /// Opaque cursor from a previous page's
        /// [`next_cursor`](MarketsKeysetPage::next_cursor).
        after_cursor: into String;
        /// Only markets with these ids.
        id: many MarketId;
        /// Only markets with these slugs.
        slug: many String;
        /// Only closed (`true`) or only open (`false`, the server default) markets.
        closed: bool;
        /// The `decimalized` flag (undocumented beyond its boolean type).
        decimalized: bool;
        /// Only markets with these CLOB token ids.
        clob_token_ids: many TokenId;
        /// Only markets with these condition ids.
        condition_ids: many ConditionId;
        /// Only markets with these question ids.
        question_ids: many QuestionId;
        /// Minimum liquidity.
        liquidity_num_min: into Decimal;
        /// Maximum liquidity.
        liquidity_num_max: into Decimal;
        /// Minimum volume.
        volume_num_min: into Decimal;
        /// Maximum volume.
        volume_num_max: into Decimal;
        /// Earliest start date.
        start_date_min: DateTime<Utc>;
        /// Latest start date.
        start_date_max: DateTime<Utc>;
        /// Earliest end date.
        end_date_min: DateTime<Utc>;
        /// Latest end date.
        end_date_max: DateTime<Utc>;
        /// Only markets with these tags.
        tag_id: many TagId;
        /// Include markets with tags related to [`tag_id`](Self::tag_id).
        related_tags: bool;
        /// How to match tags (the spec documents no values).
        tag_match: into String;
        /// Only "create your own market" markets (`true`) or only other markets (`false`).
        cyom: bool;
        /// Only markets with RFQ enabled (`true`) or disabled (`false`).
        rfq_enabled: bool;
        /// Only markets with this UMA resolution status.
        uma_resolution_status: into String;
        /// Only markets for this game id.
        game_id: into String;
        /// Only markets with these sports market types.
        sports_market_types: many String;
        /// Include each market's tags.
        include_tag: bool;
        /// Locale.
        locale: into String;
    }

    async fn fetch(&self, after_cursor: Option<&str>) -> Result<MarketsKeysetPage> {
        validate_keyset_limit(self.params.limit)?;
        self.client
            .transport
            .get(&["markets", "keyset"])
            .query(self.params.query(after_cursor))
            .send()
            .await
    }

    /// Fetches one page, starting at [`after_cursor`](Self::after_cursor) if set.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if [`limit`](Self::limit) is out of
    /// range; otherwise see [`Error`](crate::Error). The server answers `422` (an
    /// [`Error::Api`](crate::Error::Api) whose
    /// [`error_type`](crate::ApiError::error_type) is `"validation error"`) for an invalid
    /// cursor, order field or filter.
    pub async fn send(self) -> Result<MarketsKeysetPage> {
        self.fetch(self.params.after_cursor.as_deref()).await
    }

    /// Streams every market from [`after_cursor`](Self::after_cursor) (or the beginning)
    /// onwards, fetching pages lazily until a page has no `next_cursor`.
    pub fn into_stream(self) -> impl Stream<Item = Result<Market>> + Send + 'static {
        let start = self.params.after_cursor.clone();
        cursor_stream(start, move |cursor| {
            let request = self.clone();
            async move {
                let page = request.fetch(cursor.as_deref()).await?;
                Ok(CursorPage::new(
                    page.markets.unwrap_or_default(),
                    page.next_cursor,
                ))
            }
        })
    }
}

/// Request builder for [`GammaClient::get_markets_information`] and
/// [`GammaClient::get_abridged_markets`].
///
/// Every setter maps to one field of the JSON body
/// (`components/schemas/MarketsInformationBody`); unset fields are omitted.
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetMarketsInformation {
    client: GammaClient,
    endpoint: &'static str,
    params: MarketsInformationParams,
}

#[derive(Debug, Clone, Default)]
struct MarketsInformationParams {
    id: Vec<MarketId>,
    slug: Vec<String>,
    closed: Option<bool>,
    clob_token_ids: Vec<TokenId>,
    condition_ids: Vec<ConditionId>,
    liquidity_num_min: Option<Decimal>,
    liquidity_num_max: Option<Decimal>,
    volume_num_min: Option<Decimal>,
    volume_num_max: Option<Decimal>,
    start_date_min: Option<DateTime<Utc>>,
    start_date_max: Option<DateTime<Utc>>,
    end_date_min: Option<DateTime<Utc>>,
    end_date_max: Option<DateTime<Utc>>,
    related_tags: Option<bool>,
    tag_id: Option<TagId>,
    cyom: Option<bool>,
    uma_resolution_status: Option<String>,
    game_id: Option<String>,
    sports_market_types: Vec<String>,
    rewards_min_size: Option<Decimal>,
    question_ids: Vec<QuestionId>,
    include_tags: Option<bool>,
}

/// The wire form of `components/schemas/MarketsInformationBody`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MarketsInformationBody {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    id: Vec<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    slug: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    closed: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    clob_token_ids: Vec<TokenId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    condition_ids: Vec<ConditionId>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::decimal_number_option"
    )]
    liquidity_num_min: Option<Decimal>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::decimal_number_option"
    )]
    liquidity_num_max: Option<Decimal>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::decimal_number_option"
    )]
    volume_num_min: Option<Decimal>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::decimal_number_option"
    )]
    volume_num_max: Option<Decimal>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::datetime_option"
    )]
    start_date_min: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::datetime_option"
    )]
    start_date_max: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::datetime_option"
    )]
    end_date_min: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::datetime_option"
    )]
    end_date_max: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    related_tags: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cyom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uma_resolution_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    game_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    sports_market_types: Vec<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::decimal_number_option"
    )]
    rewards_min_size: Option<Decimal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    question_ids: Vec<QuestionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_tags: Option<bool>,
}

impl MarketsInformationParams {
    /// Builds the JSON body, converting ids the spec types as integers.
    fn into_body(self) -> Result<MarketsInformationBody> {
        let id = self
            .id
            .iter()
            .map(|id| integer_id("id", id.as_str()))
            .collect::<Result<Vec<_>>>()?;
        let tag_id = self
            .tag_id
            .as_ref()
            .map(|tag| integer_id("tagId", tag.as_str()))
            .transpose()?;
        Ok(MarketsInformationBody {
            id,
            slug: self.slug,
            closed: self.closed,
            clob_token_ids: self.clob_token_ids,
            condition_ids: self.condition_ids,
            liquidity_num_min: self.liquidity_num_min,
            liquidity_num_max: self.liquidity_num_max,
            volume_num_min: self.volume_num_min,
            volume_num_max: self.volume_num_max,
            start_date_min: self.start_date_min,
            start_date_max: self.start_date_max,
            end_date_min: self.end_date_min,
            end_date_max: self.end_date_max,
            related_tags: self.related_tags,
            tag_id,
            cyom: self.cyom,
            uma_resolution_status: self.uma_resolution_status,
            game_id: self.game_id,
            sports_market_types: self.sports_market_types,
            rewards_min_size: self.rewards_min_size,
            question_ids: self.question_ids,
            include_tags: self.include_tags,
        })
    }
}

impl GetMarketsInformation {
    setters! {
        /// Only markets with these ids (`id`). The spec types ids as integers, so
        /// non-numeric ids are rejected client-side with
        /// [`Error::Validation`](crate::Error::Validation).
        id: many MarketId;
        /// Only markets with these slugs (`slug`).
        slug: many String;
        /// Only closed (`true`) or only open (`false`) markets (`closed`).
        closed: bool;
        /// Only markets with these CLOB token ids (`clobTokenIds`).
        clob_token_ids: many TokenId;
        /// Only markets with these condition ids (`conditionIds`).
        condition_ids: many ConditionId;
        /// Minimum liquidity (`liquidityNumMin`).
        liquidity_num_min: into Decimal;
        /// Maximum liquidity (`liquidityNumMax`).
        liquidity_num_max: into Decimal;
        /// Minimum volume (`volumeNumMin`).
        volume_num_min: into Decimal;
        /// Maximum volume (`volumeNumMax`).
        volume_num_max: into Decimal;
        /// Earliest start date (`startDateMin`).
        start_date_min: DateTime<Utc>;
        /// Latest start date (`startDateMax`).
        start_date_max: DateTime<Utc>;
        /// Earliest end date (`endDateMin`).
        end_date_min: DateTime<Utc>;
        /// Latest end date (`endDateMax`).
        end_date_max: DateTime<Utc>;
        /// Include markets with tags related to [`tag_id`](Self::tag_id) (`relatedTags`).
        related_tags: bool;
        /// Only markets with this tag (`tagId`). The spec types it as an integer, so a
        /// non-numeric id is rejected client-side with
        /// [`Error::Validation`](crate::Error::Validation).
        tag_id: into TagId;
        /// Only "create your own market" markets (`true`) or only other markets (`false`)
        /// (`cyom`).
        cyom: bool;
        /// Only markets with this UMA resolution status (`umaResolutionStatus`).
        uma_resolution_status: into String;
        /// Only markets for this game id (`gameId`).
        game_id: into String;
        /// Only markets with these sports market types (`sportsMarketTypes`).
        sports_market_types: many String;
        /// Minimum size for liquidity rewards (`rewardsMinSize`).
        rewards_min_size: into Decimal;
        /// Only markets with these question ids (`questionIds`).
        question_ids: many QuestionId;
        /// Include each market's tags (`includeTags`).
        include_tags: bool;
    }

    /// Sends the request.
    ///
    /// The endpoint is read-only, so it is retried like a `GET` on transient failures.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if an id is not an integer;
    /// otherwise see [`Error`](crate::Error). The server answers `422` for invalid filters.
    pub async fn send(self) -> Result<Vec<Market>> {
        let body = self.params.into_body()?;
        self.client
            .transport
            .post(&["markets", self.endpoint])
            .json(&body)
            .idempotent(true)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names and types from `components/schemas/Market` in
    /// `docs/specs/gamma-openapi.yaml` (the docs publish no example body).
    #[test]
    fn deserializes_market_wire_names_and_types() {
        let json = r#"{
            "id": "239826",
            "question": "Will it rain?",
            "conditionId": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
            "endDate": "2024-11-05T12:00:00Z",
            "liquidity": "1500.25",
            "volume": 2000,
            "fee": "20000000000000000",
            "outcomes": "x",
            "questionID": "0xabc",
            "teamAID": "7",
            "volume24hr": 12.5,
            "volume1wk": "13",
            "volume24hrClob": 1.25,
            "orderPriceMinTickSize": 0.01,
            "bestBid": 0.47,
            "bestAsk": 0.48,
            "createdBy": 3,
            "closedTime": "2024-11-06 00:00:00+00",
            "feeSchedule": {"exponent": 2, "rate": 0.02, "takerOnly": true, "rebateRate": null},
            "events": [{"id": "1"}],
            "tags": [{"id": "2", "label": "Weather"}]
        }"#;
        let market: Market = serde_json::from_str(json).unwrap();
        assert_eq!(market.id, Some(MarketId::from("239826")));
        assert_eq!(market.liquidity, Some(Decimal::new(150_025, 2)));
        assert_eq!(market.volume, Some(Decimal::from(2000)));
        assert_eq!(market.fee, Some(Decimal::from(20_000_000_000_000_000_u64)));
        assert_eq!(market.question_id, Some(QuestionId::from("0xabc")));
        assert_eq!(market.team_a_id.as_deref(), Some("7"));
        assert_eq!(market.volume_24hr, Some(Decimal::new(125, 1)));
        assert_eq!(market.volume_1wk, Some(Decimal::from(13)));
        assert_eq!(market.volume_24hr_clob, Some(Decimal::new(125, 2)));
        assert_eq!(market.order_price_min_tick_size, Some(Decimal::new(1, 2)));
        assert_eq!(market.best_bid, Some(Decimal::new(47, 2)));
        assert_eq!(
            market.closed_time.as_deref(),
            Some("2024-11-06 00:00:00+00")
        );
        assert_eq!(market.end_date.map(|d| d.timestamp()), Some(1_730_808_000));
        let fees = market.fee_schedule.as_ref().unwrap();
        assert_eq!(fees.rate, Some(Decimal::new(2, 2)));
        assert_eq!(fees.taker_only, Some(true));
        assert_eq!(fees.rebate_rate, None);
        assert_eq!(market.events.as_ref().unwrap().len(), 1);
        assert_eq!(market.tags.as_ref().unwrap()[0].id, Some(TagId::from("2")));
    }

    #[test]
    fn number_fields_serialize_as_numbers_and_strings_as_strings() {
        let market: Market =
            serde_json::from_str(r#"{"liquidity":"1.5","liquidityNum":1.5}"#).unwrap();
        let value = serde_json::to_value(&market).unwrap();
        assert_eq!(value["liquidity"], serde_json::json!("1.5"));
        assert_eq!(value["liquidityNum"], serde_json::json!(1.5));
    }

    #[test]
    fn deserializes_keyset_page() {
        let page: MarketsKeysetPage =
            serde_json::from_str(r#"{"markets":[{"id":"1"}],"next_cursor":"abc"}"#).unwrap();
        assert_eq!(page.markets.unwrap().len(), 1);
        assert_eq!(page.next_cursor.as_deref(), Some("abc"));
        let last: MarketsKeysetPage = serde_json::from_str(r#"{"markets":[]}"#).unwrap();
        assert_eq!(last.next_cursor, None);
    }

    #[test]
    fn information_body_uses_spec_names_and_integer_ids() {
        let params = MarketsInformationParams {
            id: vec![MarketId::from("12"), MarketId::from("13")],
            tag_id: Some(TagId::from("5")),
            liquidity_num_min: Some(Decimal::new(15, 1)),
            clob_token_ids: vec![TokenId::from("123")],
            include_tags: Some(true),
            ..MarketsInformationParams::default()
        };
        let body = serde_json::to_value(params.into_body().unwrap()).unwrap();
        assert_eq!(
            body,
            serde_json::json!({
                "id": [12, 13],
                "clobTokenIds": ["123"],
                "liquidityNumMin": 1.5,
                "tagId": 5,
                "includeTags": true
            })
        );

        let bad = MarketsInformationParams {
            id: vec![MarketId::from("abc")],
            ..MarketsInformationParams::default()
        };
        let err = bad.into_body().unwrap_err();
        assert!(matches!(err, crate::Error::Validation(_)), "{err:?}");
    }
}
