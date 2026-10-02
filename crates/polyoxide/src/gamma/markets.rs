//! Markets: `/markets`, `/markets/{id}`, `/markets/slug/{slug}`, `/markets/{id}/tags`,
//! `/markets/{id}/description`, `/markets/keyset`, `/markets/information` and
//! `/markets/abridged`.

use crate::Paginated;
use chrono::{DateTime, Utc};
use polyoxide_core::{
    Query, Result,
    pagination::{CursorPage, cursor_stream, offset_stream},
    serde_util,
    types::{ConditionId, MarketId, QuestionId, TokenId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    Category, Event, GammaClient, ImageOptimization, Tag, TagId, TeamId,
    util::{
        Lookup, check_integer_id, check_integer_ids, integer_id, rfc3339, setters,
        validate_keyset_limit,
    },
};

/// A market (`components/schemas/Market`).
///
/// Every field is optional because the spec marks none as required. Fields the spec types
/// as `number` are [`Decimal`]s and serialize back as JSON numbers. The amounts the spec
/// types as `string` (`liquidity`, `volume`, `fee`, `umaBond`, `umaReward`) are parsed
/// into [`Decimal`]s too: an empty string or `null` becomes `None`, any other non-numeric
/// text fails decoding, and they serialize back as JSON strings. Other string fields are
/// kept exactly as sent.
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
    /// Liquidity. The spec types this as a string (`liquidity`); it is parsed as a decimal, and
    /// an empty string becomes `None`.
    #[serde(default, with = "serde_util::string_or_number_option")]
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
    /// Fee. The spec types this as a string (`fee`); it is parsed as a decimal, and
    /// an empty string becomes `None`.
    #[serde(default, with = "serde_util::string_or_number_option")]
    pub fee: Option<Decimal>,
    /// Image URL.
    pub image: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Lower bound. The spec types this as a plain string.
    pub lower_bound: Option<String>,
    /// Upper bound. The spec types this as a plain string.
    pub upper_bound: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Outcomes. The spec types this as a plain string and documents no encoding, so it is
    /// kept exactly as sent.
    pub outcomes: Option<String>,
    /// Outcome prices. The spec types this as a plain string and documents no encoding, so
    /// it is kept exactly as sent.
    pub outcome_prices: Option<String>,
    /// Volume. The spec types this as a string (`volume`); it is parsed as a decimal, and
    /// an empty string becomes `None`.
    #[serde(default, with = "serde_util::string_or_number_option")]
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
    /// Team A id (wire name `teamAID`). The spec types this as a string here (a team's
    /// own `id` is an integer); it is kept exactly as sent.
    #[serde(rename = "teamAID")]
    pub team_a_id: Option<TeamId>,
    /// Team B id (wire name `teamBID`). The spec types this as a string here (a team's
    /// own `id` is an integer); it is kept exactly as sent.
    #[serde(rename = "teamBID")]
    pub team_b_id: Option<TeamId>,
    /// UMA bond. The spec types this as a string (`umaBond`); it is parsed as a decimal, and
    /// an empty string becomes `None`.
    #[serde(default, with = "serde_util::string_or_number_option")]
    pub uma_bond: Option<Decimal>,
    /// UMA reward. The spec types this as a string (`umaReward`); it is parsed as a decimal, and
    /// an empty string becomes `None`.
    #[serde(default, with = "serde_util::string_or_number_option")]
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
    /// Tags. The keyset listing documents them as included only with `include_tag=true`.
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
    /// The `negRiskOther` flag (documented only as a boolean).
    pub neg_risk_other: Option<bool>,
    /// Game id.
    pub game_id: Option<String>,
    /// Range of this market within its group.
    pub group_item_range: Option<String>,
    /// Sports market type (see
    /// [`GammaClient::get_sports_market_types`](super::GammaClient::get_sports_market_types)).
    pub sports_market_type: Option<String>,
    /// Line.
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
    /// Fee schedule (wire name `feeSchedule`). The keyset listings' response description
    /// spells it `fee_schedule`; this field reads only the schema's `feeSchedule`.
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
///
/// The fields keep their wire names; [`items`](Self::items),
/// [`into_items`](Self::into_items) and [`next_cursor()`](Self::next_cursor()) give the same
/// view as every other page type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketsKeysetPage {
    /// The markets on this page (documented as an empty array if none were found).
    pub markets: Option<Vec<Market>>,
    /// Cursor for the next page, passed to [`cursor`](ListMarketsKeyset::cursor). The spec
    /// documents it as present only when the number of returned markets equals the
    /// effective limit, and omitted on the last page.
    pub next_cursor: Option<String>,
}

impl MarketsKeysetPage {
    /// The markets on this page (empty if the page carries none).
    #[must_use]
    pub fn items(&self) -> &[Market] {
        self.markets.as_deref().unwrap_or_default()
    }

    /// Consumes the page and returns its markets.
    #[must_use]
    pub fn into_items(self) -> Vec<Market> {
        self.markets.unwrap_or_default()
    }

    /// The cursor for the next page, or `None` on the last page (an absent or empty
    /// `next_cursor`).
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor
            .as_deref()
            .filter(|cursor| !cursor.is_empty())
    }
}

impl GammaClient {
    /// Lists markets (offset pagination).
    ///
    /// The spec documents the `closed` filter's default as `false`.
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/list-markets>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let markets = gamma.list_markets().limit(10).closed(false).send().await?;
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
    /// - [`Error::Validation`](crate::Error::Validation) (parameter `id`) if `id` is not an
    ///   integer (one or more ASCII digits), checked before sending;
    /// - [`Error::Api`](crate::Error::Api) with status `404` if the market does not exist;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn get_market_tags(&self, id: impl Into<MarketId>) -> Result<Vec<Tag>> {
        let id = id.into();
        check_integer_id("id", id.as_str())?;
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
    /// - [`Error::Validation`](crate::Error::Validation) (parameter `id`) if `id` is not an
    ///   integer (one or more ASCII digits), checked before sending;
    /// - [`Error::Api`](crate::Error::Api) with status `404` if the market does not exist;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn get_market_description(
        &self,
        id: impl Into<MarketId>,
    ) -> Result<MarketDescription> {
        let id = id.into();
        check_integer_id("id", id.as_str())?;
        self.transport
            .get(&["markets", id.as_str(), "description"])
            .send()
            .await
    }

    /// Lists markets with cursor-based (keyset) pagination, for stable paging through large
    /// result sets.
    ///
    /// [`send`](ListMarketsKeyset::send) returns one [`MarketsKeysetPage`]; pass its
    /// [`next_cursor()`](MarketsKeysetPage::next_cursor()) to
    /// [`cursor`](ListMarketsKeyset::cursor) for the next page, or use
    /// [`into_stream`](ListMarketsKeyset::into_stream) to walk every page.
    ///
    /// The spec's response description says nested `clob_rewards` and `fee_schedule` are
    /// populated on each market. The documented `Market` schema has no `clob_rewards`
    /// property, so that data is not modelled (it is dropped when decoding), and it spells
    /// the fee schedule `feeSchedule` ([`Market::fee_schedule`]).
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

    /// Queries markets by information filters sent as a JSON body
    /// (`POST /markets/information`).
    ///
    /// [`get_abridged_markets`](Self::get_abridged_markets) takes the same filters on
    /// `POST /markets/abridged`.
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

    /// Queries abridged markets by information filters sent as a JSON body
    /// (`POST /markets/abridged`).
    ///
    /// Takes the same filters and returns the same [`Market`] schema as
    /// [`get_markets_information`](Self::get_markets_information) (`POST
    /// /markets/information`).
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
    limit: Option<u32>,
    offset: Option<u32>,
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
    fn validate(&self) -> Result<()> {
        check_integer_ids("id", &self.id)?;
        if let Some(tag_id) = &self.tag_id {
            check_integer_id("tag_id", tag_id.as_str())?;
        }
        Ok(())
    }

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
        /// Maximum number of markets per page (`limit`; the docs give a minimum of `0` and
        /// no maximum).
        limit: u32;
        /// Number of markets to skip (`offset`).
        offset: u32;
        /// Comma-separated list of fields to order by (`order`).
        order: into String;
        /// Sort ascending (`true`) or descending (`false`) (`ascending`).
        ascending: bool;
        /// Filter by market ids (`id`, repeated). The spec types them as integers, so an id
        /// that is not one or more ASCII digits is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        ids => id: many MarketId;
        /// Filter by slugs (`slug`, repeated).
        slugs => slug: many String;
        /// Filter by CLOB token ids (`clob_token_ids`, repeated).
        clob_token_ids: many TokenId;
        /// Filter by condition ids (`condition_ids`, repeated).
        condition_ids: many ConditionId;
        /// Minimum liquidity (`liquidity_num_min`).
        liquidity_num_min: into Decimal;
        /// Maximum liquidity (`liquidity_num_max`).
        liquidity_num_max: into Decimal;
        /// Minimum volume (`volume_num_min`).
        volume_num_min: into Decimal;
        /// Maximum volume (`volume_num_max`).
        volume_num_max: into Decimal;
        /// Earliest start date (`start_date_min`).
        start_date_min: DateTime<Utc>;
        /// Latest start date (`start_date_max`).
        start_date_max: DateTime<Utc>;
        /// Earliest end date (`end_date_min`).
        end_date_min: DateTime<Utc>;
        /// Latest end date (`end_date_max`).
        end_date_max: DateTime<Utc>;
        /// Filter by tag id (`tag_id`). The spec types it as an integer, so an id that is not
        /// one or more ASCII digits is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        tag_id: into TagId;
        /// The `related_tags` flag (documented only as a boolean).
        related_tags: bool;
        /// The `cyom` filter (documented only as a boolean).
        cyom: bool;
        /// Filter by UMA resolution status (`uma_resolution_status`; the spec documents no
        /// values).
        uma_resolution_status: into String;
        /// Filter by game id (`game_id`, a string on this endpoint).
        game_id: into String;
        /// Filter by sports market types (`sports_market_types`, repeated).
        sports_market_types: many String;
        /// Minimum size for liquidity rewards (`rewards_min_size`).
        rewards_min_size: into Decimal;
        /// Filter by question ids (`question_ids`, repeated).
        question_ids: many QuestionId;
        /// The `include_tag` flag (documented only as a boolean on this endpoint).
        include_tag: bool;
        /// The `closed` filter (documented only as a boolean, defaulting to `false`).
        closed: bool;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Market>> {
        self.params.validate()?;
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
    /// - [`Error::Validation`](crate::Error::Validation) if an
    ///   [`ids`](Self::ids) entry or [`tag_id`](Self::tag_id) is not an integer, checked
    ///   before sending;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Market>> {
        self.fetch(self.params.offset.map(u64::from)).await
    }

    /// Streams every market from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page, or right after yielding the first error
    /// (the errors of [`send`](Self::send)). A page shorter than [`limit`](Self::limit)
    /// does not end it, because the docs give no maximum `limit` and the server may return
    /// fewer markets, so the last request returns an empty page.
    pub fn into_stream(self) -> Paginated<Market> {
        let start = self.params.offset.map_or(0, u64::from);
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
    /// The `include_tag` flag (documented only as a boolean on this endpoint).
    pub fn include_tag(mut self, include_tag: bool) -> Self {
        self.include_tag = Some(include_tag);
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation), checked before sending, if the id
    ///   is not an integer (parameter `id`) or the slug is empty, `.` or `..` (parameter
    ///   `slug`);
    /// - [`Error::Api`](crate::Error::Api) with status `404` if the market does not exist
    ///   (see [`Error::is_not_found`](crate::Error::is_not_found));
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Market> {
        let segments = self.lookup.path("markets", &[])?;
        let mut query = Query::new();
        query.push_opt("include_tag", self.include_tag);
        self.client
            .transport
            .get(&segments)
            .query(query)
            .send()
            .await
    }
}

/// Request builder for [`GammaClient::list_markets_keyset`].
///
/// The endpoint rejects `offset`, so there is no setter for it; page with
/// [`cursor`](Self::cursor) instead.
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListMarketsKeyset {
    client: GammaClient,
    params: ListMarketsKeysetParams,
}

#[derive(Debug, Clone, Default)]
struct ListMarketsKeysetParams {
    limit: Option<u32>,
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
    fn validate(&self) -> Result<()> {
        validate_keyset_limit(self.limit)?;
        check_integer_ids("id", &self.id)?;
        check_integer_ids("tag_id", &self.tag_id)
    }

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
        /// Maximum number of markets per page (`limit`), between 1 and 100 (server default
        /// 20). Values outside that range are rejected client-side with
        /// [`Error::Validation`](crate::Error::Validation).
        limit: u32;
        /// Comma-separated list of JSON field names to order by (`order`). The spec's
        /// example is `volume_num,liquidity_num`.
        order: into String;
        /// Sort direction (`ascending`, server default `true`). Only used when
        /// [`order`](Self::order) is set.
        ascending: bool;
        /// Opaque cursor from a previous page's
        /// [`next_cursor()`](MarketsKeysetPage::next_cursor()), sent as `after_cursor`.
        cursor => after_cursor: into String;
        /// Filter by market ids (`id`, repeated). The spec types them as integers, so an id
        /// that is not one or more ASCII digits is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        ids => id: many MarketId;
        /// Filter by slugs (`slug`, repeated).
        slugs => slug: many String;
        /// The `closed` filter (documented only as a boolean, defaulting to `false`).
        closed: bool;
        /// The `decimalized` flag (documented only as a boolean).
        decimalized: bool;
        /// Filter by CLOB token ids (`clob_token_ids`, repeated).
        clob_token_ids: many TokenId;
        /// Filter by condition ids (`condition_ids`, repeated).
        condition_ids: many ConditionId;
        /// Filter by question ids (`question_ids`, repeated).
        question_ids: many QuestionId;
        /// Minimum liquidity (`liquidity_num_min`).
        liquidity_num_min: into Decimal;
        /// Maximum liquidity (`liquidity_num_max`).
        liquidity_num_max: into Decimal;
        /// Minimum volume (`volume_num_min`).
        volume_num_min: into Decimal;
        /// Maximum volume (`volume_num_max`).
        volume_num_max: into Decimal;
        /// Earliest start date (`start_date_min`).
        start_date_min: DateTime<Utc>;
        /// Latest start date (`start_date_max`).
        start_date_max: DateTime<Utc>;
        /// Earliest end date (`end_date_min`).
        end_date_min: DateTime<Utc>;
        /// Latest end date (`end_date_max`).
        end_date_max: DateTime<Utc>;
        /// Filter by tag ids (`tag_id`, repeated). The spec types them as integers, so an id
        /// that is not one or more ASCII digits is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        tag_ids => tag_id: many TagId;
        /// The `related_tags` flag (documented only as a boolean).
        related_tags: bool;
        /// The `tag_match` parameter (a string; the spec documents no values).
        tag_match: into String;
        /// The `cyom` filter (documented only as a boolean).
        cyom: bool;
        /// The `rfq_enabled` filter (documented only as a boolean).
        rfq_enabled: bool;
        /// Filter by UMA resolution status (`uma_resolution_status`; the spec documents no
        /// values).
        uma_resolution_status: into String;
        /// Filter by game id (`game_id`, a string on this endpoint).
        game_id: into String;
        /// Filter by sports market types (`sports_market_types`, repeated).
        sports_market_types: many String;
        /// When `true`, includes the `Tags` relation on each market (`include_tag`).
        include_tag: bool;
        /// The `locale` parameter (a string; the spec documents no values).
        locale: into String;
    }

    async fn fetch(&self, after_cursor: Option<&str>) -> Result<MarketsKeysetPage> {
        self.params.validate()?;
        self.client
            .transport
            .get(&["markets", "keyset"])
            .query(self.params.query(after_cursor))
            .send()
            .await
    }

    /// Fetches one page, starting at [`cursor`](Self::cursor) if set.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation), checked before sending, if
    ///   [`limit`](Self::limit) is out of range or an [`ids`](Self::ids) or
    ///   [`tag_ids`](Self::tag_ids) entry is not an integer;
    /// - [`Error::Api`](crate::Error::Api) with status `422` (whose
    ///   [`error_type`](crate::ApiError::error_type) is `"validation error"`) for an
    ///   invalid cursor, order field or filter;
    /// - [`Error::Api`](crate::Error::Api) with status `500` (`"internal error"`) for a
    ///   server-side failure;
    /// - [`Error::Api`](crate::Error::Api) with status `503` and
    ///   [`error_type`](crate::ApiError::error_type) `"service unavailable"`, which the
    ///   spec documents as "keyset pagination is not configured". That is a server
    ///   configuration state rather than a transient failure, so retrying is unlikely to
    ///   help; [`GammaClient::list_markets`] is the offset-paginated alternative;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<MarketsKeysetPage> {
        self.fetch(self.params.after_cursor.as_deref()).await
    }

    /// Streams every market from [`cursor`](Self::cursor) (or the beginning) onwards,
    /// fetching pages lazily until a page has no (or an empty) `next_cursor`.
    ///
    /// The stream ends right after yielding the first error (the errors of
    /// [`send`](Self::send)).
    pub fn into_stream(self) -> Paginated<Market> {
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

/// Request builder for [`GammaClient::get_markets_information`]
/// (`POST /markets/information`) and [`GammaClient::get_abridged_markets`]
/// (`POST /markets/abridged`).
///
/// Every setter maps to one field of the JSON body
/// (`components/schemas/MarketsInformationBody`); unset fields are omitted.
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
#[doc(alias = "GetAbridgedMarkets")]
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
        /// Filter by market ids (`id`). The spec types them as integers, so an id that is
        /// not one or more ASCII digits (or does not fit in an `i64`) is rejected before
        /// sending with [`Error::Validation`](crate::Error::Validation).
        ids => id: many MarketId;
        /// Filter by slugs (`slug`).
        slugs => slug: many String;
        /// The `closed` filter (documented only as a nullable boolean).
        closed: bool;
        /// Filter by CLOB token ids (`clobTokenIds`).
        clob_token_ids: many TokenId;
        /// Filter by condition ids (`conditionIds`).
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
        /// The `relatedTags` flag (documented only as a nullable boolean).
        related_tags: bool;
        /// Filter by tag id (`tagId`). The spec types it as an integer, so an id that is not
        /// one or more ASCII digits (or does not fit in an `i64`) is rejected before sending
        /// with [`Error::Validation`](crate::Error::Validation).
        tag_id: into TagId;
        /// The `cyom` filter (documented only as a nullable boolean).
        cyom: bool;
        /// Filter by UMA resolution status (`umaResolutionStatus`; the spec documents no
        /// values).
        uma_resolution_status: into String;
        /// Filter by game id (`gameId`).
        game_id: into String;
        /// Filter by sports market types (`sportsMarketTypes`).
        sports_market_types: many String;
        /// Minimum size for liquidity rewards (`rewardsMinSize`).
        rewards_min_size: into Decimal;
        /// Filter by question ids (`questionIds`).
        question_ids: many QuestionId;
        /// The `includeTags` flag (documented only as a nullable boolean).
        include_tags: bool;
    }

    /// Sends the request.
    ///
    /// The endpoint is read-only, so it is retried like a `GET` on transient failures.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation), checked before sending, if an
    ///   [`ids`](Self::ids) entry (parameter `id`) or [`tag_id`](Self::tag_id) (parameter
    ///   `tagId`) is not an integer;
    /// - [`Error::Api`](crate::Error::Api) with status `422` for a body the server rejects
    ///   (documented only as "Validation error");
    /// - otherwise see [`Error`](crate::Error).
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
    /// `docs/specs/gamma-openapi.yaml` (the docs publish no example body): `liquidity`,
    /// `volume` and `teamAID` are strings, `volume24hr`, `volume1wk` and the prices are
    /// numbers.
    #[test]
    fn deserializes_market_wire_names_and_types() {
        let json = r#"{
            "id": "239826",
            "question": "Will it rain?",
            "conditionId": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
            "endDate": "2024-11-05T12:00:00Z",
            "liquidity": "1500.25",
            "volume": "2000",
            "outcomes": "x",
            "questionID": "0xabc",
            "teamAID": "7",
            "volume24hr": 12.5,
            "volume1wk": 13,
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
        assert_eq!(market.fee, None);
        assert_eq!(market.question_id, Some(QuestionId::from("0xabc")));
        assert_eq!(market.team_a_id, Some(TeamId::from("7")));
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

        // String-typed fields stay strings and number-typed fields stay numbers.
        let value = serde_json::to_value(&market).unwrap();
        assert_eq!(value["liquidity"], serde_json::json!("1500.25"));
        assert_eq!(value["volume"], serde_json::json!("2000"));
        assert_eq!(value["teamAID"], serde_json::json!("7"));
        assert_eq!(value["volume1wk"], serde_json::json!(13));
        assert_eq!(value["volume24hr"], serde_json::json!(12.5));
    }

    /// The string-typed amounts (`liquidity`, `volume`, `fee`, `umaBond`, `umaReward`):
    /// `""` and `null` are "absent", other non-numeric text is a decode error.
    #[test]
    fn string_amounts_treat_empty_as_none() {
        let market: Market = serde_json::from_str(
            r#"{"liquidity":"","volume":"","fee":null,"umaBond":"  ","umaReward":"5"}"#,
        )
        .unwrap();
        assert_eq!(market.liquidity, None);
        assert_eq!(market.volume, None);
        assert_eq!(market.fee, None);
        assert_eq!(market.uma_bond, None);
        assert_eq!(market.uma_reward, Some(Decimal::from(5)));

        for field in ["liquidity", "volume", "fee", "umaBond", "umaReward"] {
            let json = format!(r#"{{"{field}":"n/a"}}"#);
            let err = serde_json::from_str::<Market>(&json).unwrap_err();
            assert!(err.to_string().contains("n/a"), "{field}: {err}");
        }
    }

    /// Lenient decoding beyond the spec: a string-typed amount sent as a JSON number and a
    /// number-typed field sent as a numeric string are accepted, and serialize back in
    /// their documented wire type.
    #[test]
    fn amounts_accept_the_other_json_representation() {
        let market: Market =
            serde_json::from_str(r#"{"liquidity":1.5,"liquidityNum":"1.5"}"#).unwrap();
        assert_eq!(market.liquidity, Some(Decimal::new(15, 1)));
        assert_eq!(market.liquidity_num, Some(Decimal::new(15, 1)));
        let value = serde_json::to_value(&market).unwrap();
        assert_eq!(value["liquidity"], serde_json::json!("1.5"));
        assert_eq!(value["liquidityNum"], serde_json::json!(1.5));
    }

    #[test]
    fn deserializes_keyset_page_and_accessors() {
        let page: MarketsKeysetPage =
            serde_json::from_str(r#"{"markets":[{"id":"1"}],"next_cursor":"abc"}"#).unwrap();
        assert_eq!(page.items().len(), 1);
        assert_eq!(page.next_cursor(), Some("abc"));
        assert_eq!(page.next_cursor.as_deref(), Some("abc"));
        assert_eq!(page.clone().into_items()[0].id, Some(MarketId::from("1")));

        let last: MarketsKeysetPage = serde_json::from_str(r#"{"markets":[]}"#).unwrap();
        assert_eq!(last.next_cursor(), None);
        assert!(last.items().is_empty());

        let empty: MarketsKeysetPage = serde_json::from_str(r#"{"next_cursor":""}"#).unwrap();
        assert_eq!(empty.next_cursor(), None);
        assert!(empty.items().is_empty());
        assert!(empty.into_items().is_empty());
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

        for (params, parameter) in [
            (
                MarketsInformationParams {
                    id: vec![MarketId::from("abc")],
                    ..MarketsInformationParams::default()
                },
                "id",
            ),
            (
                MarketsInformationParams {
                    tag_id: Some(TagId::from("-1")),
                    ..MarketsInformationParams::default()
                },
                "tagId",
            ),
        ] {
            let err = params.into_body().unwrap_err();
            let crate::Error::Validation(v) = &err else {
                panic!("expected a validation error, got {err:?}")
            };
            assert_eq!(v.parameter(), parameter);
        }
    }

    #[test]
    fn integer_typed_query_filters_are_validated() {
        let ok = ListMarketsParams {
            id: vec![MarketId::from("1")],
            tag_id: Some(TagId::from("2")),
            ..ListMarketsParams::default()
        };
        assert!(ok.validate().is_ok());
        let bad = ListMarketsParams {
            tag_id: Some(TagId::from("politics")),
            ..ListMarketsParams::default()
        };
        assert!(
            matches!(bad.validate(), Err(crate::Error::Validation(v)) if v.parameter() == "tag_id")
        );

        let bad = ListMarketsKeysetParams {
            tag_id: vec![TagId::from("1"), TagId::from("x")],
            ..ListMarketsKeysetParams::default()
        };
        assert!(
            matches!(bad.validate(), Err(crate::Error::Validation(v)) if v.parameter() == "tag_id")
        );
        let bad = ListMarketsKeysetParams {
            id: vec![MarketId::from("")],
            ..ListMarketsKeysetParams::default()
        };
        assert!(
            matches!(bad.validate(), Err(crate::Error::Validation(v)) if v.parameter() == "id")
        );
    }
}
