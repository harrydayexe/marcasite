# Module `marcasite::gamma`

> Generated from marcasite 0.1.1 (all features) by `just docs-md`. Do not edit.

Gamma API client (`https://gamma-api.polymarket.com`).

Covers events, markets, tags, series, comments, sports, search and public profiles.
Start from [`GammaClient`](gamma.md#struct.GammaClient); every endpoint is a method on it.

| Endpoint | Method |
|---|---|
| `GET /status` | [`get_status`](gamma.md#GammaClient.fn.get_status) |
| `GET /markets` | [`list_markets`](gamma.md#GammaClient.fn.list_markets) |
| `GET /markets/keyset` | [`list_markets_keyset`](gamma.md#GammaClient.fn.list_markets_keyset) |
| `GET /markets/{id}` | [`get_market`](gamma.md#GammaClient.fn.get_market) |
| `GET /markets/slug/{slug}` | [`get_market_by_slug`](gamma.md#GammaClient.fn.get_market_by_slug) |
| `GET /markets/{id}/tags` | [`get_market_tags`](gamma.md#GammaClient.fn.get_market_tags) |
| `GET /markets/{id}/description` | [`get_market_description`](gamma.md#GammaClient.fn.get_market_description) |
| `POST /markets/information` | [`get_markets_information`](gamma.md#GammaClient.fn.get_markets_information) |
| `POST /markets/abridged` | [`get_abridged_markets`](gamma.md#GammaClient.fn.get_abridged_markets) |
| `GET /events` | [`list_events`](gamma.md#GammaClient.fn.list_events) |
| `GET /events/keyset` | [`list_events_keyset`](gamma.md#GammaClient.fn.list_events_keyset) |
| `GET /events/pagination` | [`list_events_paginated`](gamma.md#GammaClient.fn.list_events_paginated) |
| `GET /events/results` | [`list_sport_event_results`](gamma.md#GammaClient.fn.list_sport_event_results) |
| `GET /events/{id}` | [`get_event`](gamma.md#GammaClient.fn.get_event) |
| `GET /events/slug/{slug}` | [`get_event_by_slug`](gamma.md#GammaClient.fn.get_event_by_slug) |
| `GET /events/{id}/tags` | [`get_event_tags`](gamma.md#GammaClient.fn.get_event_tags) |
| `GET /events/{id}/tweet-count` | [`get_event_tweet_count`](gamma.md#GammaClient.fn.get_event_tweet_count) |
| `GET /events/{id}/comments/count` | [`get_event_comment_count`](gamma.md#GammaClient.fn.get_event_comment_count) |
| `GET /events/creators` | [`list_event_creators`](gamma.md#GammaClient.fn.list_event_creators) |
| `GET /events/creators/{id}` | [`get_event_creator`](gamma.md#GammaClient.fn.get_event_creator) |
| `GET /tags` | [`list_tags`](gamma.md#GammaClient.fn.list_tags) |
| `GET /tags/{id}` | [`get_tag`](gamma.md#GammaClient.fn.get_tag) |
| `GET /tags/slug/{slug}` | [`get_tag_by_slug`](gamma.md#GammaClient.fn.get_tag_by_slug) |
| `GET /tags/{id}/related-tags` | [`get_related_tag_relationships`](gamma.md#GammaClient.fn.get_related_tag_relationships) |
| `GET /tags/slug/{slug}/related-tags` | [`get_related_tag_relationships_by_slug`](gamma.md#GammaClient.fn.get_related_tag_relationships_by_slug) |
| `GET /tags/{id}/related-tags/tags` | [`get_related_tags`](gamma.md#GammaClient.fn.get_related_tags) |
| `GET /tags/slug/{slug}/related-tags/tags` | [`get_related_tags_by_slug`](gamma.md#GammaClient.fn.get_related_tags_by_slug) |
| `GET /series` | [`list_series`](gamma.md#GammaClient.fn.list_series) |
| `GET /series/{id}` | [`get_series`](gamma.md#GammaClient.fn.get_series) |
| `GET /series/{id}/comments/count` | [`get_series_comment_count`](gamma.md#GammaClient.fn.get_series_comment_count) |
| `GET /series-summary/{id}` | [`get_series_summary`](gamma.md#GammaClient.fn.get_series_summary) |
| `GET /series-summary/slug/{slug}` | [`get_series_summary_by_slug`](gamma.md#GammaClient.fn.get_series_summary_by_slug) |
| `GET /comments` | [`list_comments`](gamma.md#GammaClient.fn.list_comments) |
| `GET /comments/{id}` | [`get_comments_by_id`](gamma.md#GammaClient.fn.get_comments_by_id) |
| `GET /comments/user_address/{user_address}` | [`list_comments_by_user`](gamma.md#GammaClient.fn.list_comments_by_user) |
| `GET /teams` | [`list_teams`](gamma.md#GammaClient.fn.list_teams) |
| `GET /teams/{id}` | [`get_team`](gamma.md#GammaClient.fn.get_team) |
| `GET /sports` | [`get_sports_metadata`](gamma.md#GammaClient.fn.get_sports_metadata) |
| `GET /sports/market-types` | [`get_sports_market_types`](gamma.md#GammaClient.fn.get_sports_market_types) |
| `GET /public-search` | [`search`](gamma.md#GammaClient.fn.search) |
| `GET /public-profile` | [`get_public_profile`](gamma.md#GammaClient.fn.get_public_profile) |
| `GET /profiles/user_address/{user_address}` | [`get_profile`](gamma.md#GammaClient.fn.get_profile) |

## Conventions

- **Responses.** The Gamma spec marks no response field as required, so every field of
  the response types is an `Option`. Fields the spec types as `number` are
  `Decimal`s that serialize back as JSON numbers; the few amounts the
  spec types as `string` (such as [`Market::liquidity`](gamma.md#struct.Market)) are decimals that serialize back
  as JSON strings. Date-time fields (`format: date-time`) are parsed leniently: besides
  RFC 3339, a value without a UTC offset, or a bare date, is read as UTC.
- **Live over docs.** Where the live API differs from the spec, this module follows the
  live API: the market lists `outcomes`, `outcomePrices`, `clobTokenIds` and
  `umaResolutionStatuses` are typed lists (decoded from the JSON-encoded string the API
  sends, or from the real array of the optimized search), `order` takes camelCase field
  names, offsets are capped (2000 for markets and events, 200 for comments), and fields the
  spec omits are modelled (marked "undocumented; observed live"). Every departure is listed
  in `SPEC_DEVIATIONS.md` in the repository root.
- **Pagination.** Offset listings return a plain `Vec` from `send()` and walk every page
  with `into_stream()`. The keyset listings return a page type with `items()` and
  `next_cursor()`; pass the cursor back with `cursor()`.
- **Ids.** Path and query ids the spec types as `integer` (market, event, tag, series,
  comment, team and creator ids) are id newtypes holding the digits as a string. They are
  checked before sending: an id that is not one or more ASCII digits is an
  [`Error::Validation`](marcasite.md#enum.Error) naming the parameter. Slugs must be
  non-empty and not `.` or `..`.
- **Errors.** Where the spec documents an error body (`422`, `500` and `503` of the
  keyset listings, `400` and `404` of [`get_public_profile`](gamma.md#GammaClient.fn.get_public_profile)),
  it carries a `type` and an `error` field, exposed as
  [`ApiError::error_type`](marcasite.md#ApiError.fn.error_type) and
  [`ApiError::message`](marcasite.md#ApiError.fn.message). Other errors, such as the `404` of the
  lookups by id or slug, are documented by status only.

See <https://docs.polymarket.com/api-reference/predictions/overview>.

## Index

- **Re-exports:** `EventId`, `MarketId`, `QuestionId`
- **Structs:** [`Category`](#struct.Category), [`Chat`](#struct.Chat), [`ClobReward`](#struct.ClobReward), [`Collection`](#struct.Collection), [`Comment`](#struct.Comment), [`CommentCount`](#struct.CommentCount), [`CommentId`](#struct.CommentId), [`CommentMedia`](#struct.CommentMedia), [`CommentPosition`](#struct.CommentPosition), [`CommentProfile`](#struct.CommentProfile), [`Event`](#struct.Event), [`EventCreator`](#struct.EventCreator), [`EventCreatorId`](#struct.EventCreatorId), [`EventTweetCount`](#struct.EventTweetCount), [`EventsKeysetPage`](#struct.EventsKeysetPage), [`EventsPage`](#struct.EventsPage), [`FeeSchedule`](#struct.FeeSchedule), [`GammaClient`](#struct.GammaClient), [`GammaClientBuilder`](#struct.GammaClientBuilder), [`GetCommentsById`](#struct.GetCommentsById), [`GetEvent`](#struct.GetEvent), [`GetMarket`](#struct.GetMarket), [`GetMarketsInformation`](#struct.GetMarketsInformation), [`GetRelatedTagRelationships`](#struct.GetRelatedTagRelationships), [`GetRelatedTags`](#struct.GetRelatedTags), [`GetSeries`](#struct.GetSeries), [`GetTag`](#struct.GetTag), [`ImageOptimization`](#struct.ImageOptimization), [`ListComments`](#struct.ListComments), [`ListCommentsByUser`](#struct.ListCommentsByUser), [`ListEventCreators`](#struct.ListEventCreators), [`ListEvents`](#struct.ListEvents), [`ListEventsKeyset`](#struct.ListEventsKeyset), [`ListEventsPaginated`](#struct.ListEventsPaginated), [`ListMarkets`](#struct.ListMarkets), [`ListMarketsKeyset`](#struct.ListMarketsKeyset), [`ListSeries`](#struct.ListSeries), [`ListSportEventResults`](#struct.ListSportEventResults), [`ListTags`](#struct.ListTags), [`ListTeams`](#struct.ListTeams), [`Market`](#struct.Market), [`MarketDescription`](#struct.MarketDescription), [`MarketsKeysetPage`](#struct.MarketsKeysetPage), [`Pagination`](#struct.Pagination), [`Profile`](#struct.Profile), [`PublicProfile`](#struct.PublicProfile), [`PublicProfileUser`](#struct.PublicProfileUser), [`Reaction`](#struct.Reaction), [`RelatedTag`](#struct.RelatedTag), [`Search`](#struct.Search), [`SearchResults`](#struct.SearchResults), [`SearchTag`](#struct.SearchTag), [`Series`](#struct.Series), [`SeriesId`](#struct.SeriesId), [`SeriesSummary`](#struct.SeriesSummary), [`SportsMarketTypes`](#struct.SportsMarketTypes), [`SportsMetadata`](#struct.SportsMetadata), [`Tag`](#struct.Tag), [`TagId`](#struct.TagId), [`Team`](#struct.Team), [`TeamId`](#struct.TeamId), [`Template`](#struct.Template)
- **Enums:** [`CommentParentEntityType`](#enum.CommentParentEntityType), [`RelatedTagsStatus`](#enum.RelatedTagsStatus)

## Re-exports

- `EventId`: re-export of [`marcasite::types::EventId`](types.md#struct.EventId).
- `MarketId`: re-export of [`marcasite::types::MarketId`](types.md#struct.MarketId).
- `QuestionId`: re-export of [`marcasite::types::QuestionId`](types.md#struct.QuestionId).

## Structs

### <a id="struct.Category"></a>`struct Category`

```rust
#[non_exhaustive]
pub struct Category {
    /// Category id.
    pub id: Option<String>,
    /// Display label.
    pub label: Option<String>,
    /// Parent category.
    pub parent_category: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Publication time. The spec types this as a plain string with no format.
    pub published_at: Option<String>,
    /// Creator. The spec types this as a string here (an integer on other schemas).
    pub created_by: Option<String>,
    /// Last updater. The spec types this as a string here (an integer on other schemas).
    pub updated_by: Option<String>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    pub updated_at: Option<DateTime<Utc>>,
}
```

A category (`components/schemas/Category`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Chat"></a>`struct Chat`

```rust
#[non_exhaustive]
pub struct Chat {
    /// Chat id.
    pub id: Option<String>,
    /// Channel id.
    pub channel_id: Option<String>,
    /// Channel name.
    pub channel_name: Option<String>,
    /// Channel image URL.
    pub channel_image: Option<String>,
    /// Whether the chat is live.
    pub live: Option<bool>,
    /// Start time.
    pub start_time: Option<DateTime<Utc>>,
    /// End time.
    pub end_time: Option<DateTime<Utc>>,
}
```

A chat channel attached to an event or series (`components/schemas/Chat`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ClobReward"></a>`struct ClobReward`

```rust
#[non_exhaustive]
pub struct ClobReward {
    /// Reward id.
    pub id: Option<String>,
    /// Condition id of the market.
    pub condition_id: Option<ConditionId>,
    /// Address of the reward asset (token contract), in either letter case.
    pub asset_address: Option<Address>,
    /// Total reward amount.
    pub rewards_amount: Option<Decimal>,
    /// Daily reward rate.
    pub rewards_daily_rate: Option<Decimal>,
    /// First day of the program, `YYYY-MM-DD` (kept as sent).
    pub start_date: Option<String>,
    /// Last day of the program, `YYYY-MM-DD` (kept as sent; `2500-12-31` means open-ended).
    pub end_date: Option<String>,
}
```

A liquidity-reward program of a market (element of [`Market::clob_rewards`](gamma.md#struct.Market)).

Not in the Gamma spec; the shape is as observed live (see `SPEC_DEVIATIONS.md`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Collection"></a>`struct Collection`

```rust
#[non_exhaustive]
pub struct Collection {
    /// Collection id.
    pub id: Option<String>,
    /// Ticker.
    pub ticker: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Subtitle.
    pub subtitle: Option<String>,
    /// Collection type.
    pub collection_type: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Tags. The spec types this as a single string, not an array of tags.
    pub tags: Option<String>,
    /// Image URL.
    pub image: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Header image URL.
    pub header_image: Option<String>,
    /// Layout.
    pub layout: Option<String>,
    /// Whether the collection is active.
    pub active: Option<bool>,
    /// Whether the collection is closed.
    pub closed: Option<bool>,
    /// Whether the collection is archived.
    pub archived: Option<bool>,
    /// Whether the collection is flagged as new.
    pub new: Option<bool>,
    /// Whether the collection is featured.
    pub featured: Option<bool>,
    /// Whether the collection is restricted.
    pub restricted: Option<bool>,
    /// Whether the collection is a template.
    pub is_template: Option<bool>,
    /// Template variables. The spec types this as a plain string.
    pub template_variables: Option<String>,
    /// Publication time. The spec types this as a plain string with no format.
    pub published_at: Option<String>,
    /// Creator.
    pub created_by: Option<String>,
    /// Last updater.
    pub updated_by: Option<String>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    pub updated_at: Option<DateTime<Utc>>,
    /// Whether comments are enabled.
    pub comments_enabled: Option<bool>,
    /// Optimized image metadata.
    pub image_optimized: Option<ImageOptimization>,
    /// Optimized icon metadata.
    pub icon_optimized: Option<ImageOptimization>,
    /// Optimized header image metadata.
    pub header_image_optimized: Option<ImageOptimization>,
}
```

A collection of events (`components/schemas/Collection`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Comment"></a>`struct Comment`

```rust
#[non_exhaustive]
pub struct Comment {
    /// Comment id.
    pub id: Option<CommentId>,
    /// Comment text.
    pub body: Option<String>,
    /// Type of the entity the comment is attached to. The spec types this as a plain
    /// string with no values; live sends `Event`, `Series` or `PerpsAsset`, and any other
    /// value is kept as [`CommentParentEntityType::Unknown`](gamma.md#enum.CommentParentEntityType).
    pub parent_entity_type: Option<CommentParentEntityType>,
    /// Id of the entity the comment is attached to (wire name `parentEntityID`).
    pub parent_entity_id: Option<i64>,
    /// Id of the comment this one replies to (wire name `parentCommentID`).
    pub parent_comment_id: Option<CommentId>,
    /// Author's address.
    pub user_address: Option<Address>,
    /// Address being replied to.
    pub reply_address: Option<Address>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    pub updated_at: Option<DateTime<Utc>>,
    /// Author's profile.
    pub profile: Option<CommentProfile>,
    /// Reactions to the comment.
    pub reactions: Option<Vec<Reaction>>,
    /// Number of reports.
    pub report_count: Option<i64>,
    /// Number of reactions.
    pub reaction_count: Option<i64>,
    /// Media attached to the comment, e.g. GIFs (undocumented; observed live; absent on
    /// comments without media).
    pub media: Option<Vec<CommentMedia>>,
}
```

A comment (`components/schemas/Comment`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.CommentCount"></a>`struct CommentCount`

```rust
#[non_exhaustive]
pub struct CommentCount {
    /// Number of comments.
    pub count: Option<i64>,
}
```

A comment count (`components/schemas/Count`), returned by
[`GammaClient::get_event_comment_count`](gamma.md#GammaClient.fn.get_event_comment_count) and [`GammaClient::get_series_comment_count`](gamma.md#GammaClient.fn.get_series_comment_count).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.CommentId"></a>`struct CommentId`

```rust
pub struct CommentId(/* private fields */);
```

A Gamma comment id (sent as a string in responses; an integer in paths).

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&CommentId>`, `From<&String>`, `From<&str>`, `From<CommentId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="CommentId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="CommentId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="CommentId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.CommentMedia"></a>`struct CommentMedia`

```rust
#[non_exhaustive]
pub struct CommentMedia {
    /// Media id.
    pub id: Option<String>,
    /// Id of the comment the media belongs to (wire name `commentID`, an integer).
    pub comment_id: Option<CommentId>,
    /// Media provider, e.g. `giphy`.
    pub provider: Option<String>,
    /// Id of the media at the provider.
    pub provider_media_id: Option<String>,
    /// Media URL.
    pub url: Option<String>,
    /// Media type, e.g. `gif` (kept as sent).
    pub media_type: Option<String>,
    /// Alternative text.
    pub alt_text: Option<String>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
}
```

Media attached to a comment (element of [`Comment::media`](gamma.md#struct.Comment)).

Not in the Gamma spec; the shape is as observed live (see `SPEC_DEVIATIONS.md`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.CommentPosition"></a>`struct CommentPosition`

```rust
#[non_exhaustive]
pub struct CommentPosition {
    /// Outcome token id.
    pub token_id: Option<TokenId>,
    /// Position size. The spec types this as a string (`positionSize`); it is parsed as a
    /// decimal (an empty string or `null` becomes `None`, other non-numeric text fails
    /// decoding) and serializes back as a JSON string.
    pub position_size: Option<Decimal>,
}
```

A position held by a comment's author (`components/schemas/CommentPosition`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.CommentProfile"></a>`struct CommentProfile`

```rust
#[non_exhaustive]
pub struct CommentProfile {
    /// Display name.
    pub name: Option<String>,
    /// Pseudonym.
    pub pseudonym: Option<String>,
    /// Whether the username is displayed publicly.
    pub display_username_public: Option<bool>,
    /// Bio.
    pub bio: Option<String>,
    /// Whether the author is a moderator.
    pub is_mod: Option<bool>,
    /// Whether the author is a creator.
    pub is_creator: Option<bool>,
    /// Proxy wallet address.
    pub proxy_wallet: Option<Address>,
    /// Base address.
    pub base_address: Option<Address>,
    /// Profile image URL.
    pub profile_image: Option<String>,
    /// Optimized profile image metadata.
    pub profile_image_optimized: Option<ImageOptimization>,
    /// The author's positions. See the `get_positions` request flag (documented only as a
    /// boolean).
    pub positions: Option<Vec<CommentPosition>>,
}
```

The profile of a comment's author (`components/schemas/CommentProfile`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Event"></a>`struct Event`

```rust
#[non_exhaustive]
pub struct Event {
    /// Event id.
    pub id: Option<EventId>,
    /// Ticker.
    pub ticker: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Subtitle.
    pub subtitle: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Resolution source.
    pub resolution_source: Option<String>,
    /// Start date.
    pub start_date: Option<DateTime<Utc>>,
    /// Creation date.
    pub creation_date: Option<DateTime<Utc>>,
    /// End date.
    pub end_date: Option<DateTime<Utc>>,
    /// Image URL.
    pub image: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Whether the event is active.
    pub active: Option<bool>,
    /// Whether the event is closed.
    pub closed: Option<bool>,
    /// Whether the event is archived.
    pub archived: Option<bool>,
    /// Whether the event is flagged as new.
    pub new: Option<bool>,
    /// Whether the event is featured.
    pub featured: Option<bool>,
    /// Whether the event is restricted.
    pub restricted: Option<bool>,
    /// Liquidity.
    pub liquidity: Option<Decimal>,
    /// Volume.
    pub volume: Option<Decimal>,
    /// Open interest.
    pub open_interest: Option<Decimal>,
    /// The `sortBy` value (a string; the spec documents no values).
    pub sort_by: Option<String>,
    /// Category.
    pub category: Option<String>,
    /// Subcategory.
    pub subcategory: Option<String>,
    /// Whether the event is a template.
    pub is_template: Option<bool>,
    /// Template variables. The spec types this as a plain string.
    pub template_variables: Option<String>,
    /// Publication time (wire name `published_at`). The spec types this as a plain string
    /// with no format.
    pub published_at: Option<String>,
    /// Creator. The spec types this as a string on events.
    pub created_by: Option<String>,
    /// Last updater. The spec types this as a string on events.
    pub updated_by: Option<String>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    pub updated_at: Option<DateTime<Utc>>,
    /// Whether comments are enabled.
    pub comments_enabled: Option<bool>,
    /// Competitiveness score.
    pub competitive: Option<Decimal>,
    /// 24-hour volume.
    pub volume_24hr: Option<Decimal>,
    /// 1-week volume.
    pub volume_1wk: Option<Decimal>,
    /// 1-month volume.
    pub volume_1mo: Option<Decimal>,
    /// 1-year volume.
    pub volume_1yr: Option<Decimal>,
    /// Featured image URL.
    pub featured_image: Option<String>,
    /// Disqus thread.
    pub disqus_thread: Option<String>,
    /// Parent event. The spec types this as a plain string.
    pub parent_event: Option<String>,
    /// Whether the order book is enabled.
    pub enable_order_book: Option<bool>,
    /// CLOB liquidity.
    pub liquidity_clob: Option<Decimal>,
    /// Whether the event uses negative risk.
    pub neg_risk: Option<bool>,
    /// Negative-risk market id (wire name `negRiskMarketID`).
    pub neg_risk_market_id: Option<String>,
    /// Negative-risk fee in basis points.
    pub neg_risk_fee_bips: Option<i64>,
    /// Number of comments.
    pub comment_count: Option<i64>,
    /// Optimized image metadata.
    pub image_optimized: Option<ImageOptimization>,
    /// Optimized icon metadata.
    pub icon_optimized: Option<ImageOptimization>,
    /// Optimized featured image metadata.
    pub featured_image_optimized: Option<ImageOptimization>,
    /// Sub-events.
    pub sub_events: Option<Vec<String>>,
    /// The event's markets.
    pub markets: Option<Vec<Market>>,
    /// The series the event belongs to.
    pub series: Option<Vec<Series>>,
    /// Categories.
    pub categories: Option<Vec<Category>>,
    /// Collections.
    pub collections: Option<Vec<Collection>>,
    /// Tags.
    pub tags: Option<Vec<Tag>>,
    /// The `cyom` flag (documented only as a boolean).
    pub cyom: Option<bool>,
    /// Close time.
    pub closed_time: Option<DateTime<Utc>>,
    /// Whether all outcomes are shown.
    pub show_all_outcomes: Option<bool>,
    /// Whether market images are shown.
    pub show_market_images: Option<bool>,
    /// Whether the event resolves automatically.
    pub automatically_resolved: Option<bool>,
    /// Whether negative risk is enabled.
    pub enable_neg_risk: Option<bool>,
    /// Whether the event activates automatically.
    pub automatically_active: Option<bool>,
    /// Event date. The spec types this as a plain string with no format.
    pub event_date: Option<String>,
    /// Start time.
    pub start_time: Option<DateTime<Utc>>,
    /// Event week.
    pub event_week: Option<i64>,
    /// Series slug.
    pub series_slug: Option<String>,
    /// Score. The spec types this as a plain string.
    pub score: Option<String>,
    /// Elapsed time. The spec types this as a plain string.
    pub elapsed: Option<String>,
    /// Period. The spec types this as a plain string.
    pub period: Option<String>,
    /// Whether the event is live.
    pub live: Option<bool>,
    /// Whether the event has ended.
    pub ended: Option<bool>,
    /// When the event finished.
    pub finished_timestamp: Option<DateTime<Utc>>,
    /// GMP chart mode.
    pub gmp_chart_mode: Option<String>,
    /// Event creators.
    pub event_creators: Option<Vec<EventCreator>>,
    /// Number of tweets.
    pub tweet_count: Option<i64>,
    /// Chats. The keyset listing documents them as included only with
    /// `include_chat=true`.
    pub chats: Option<Vec<Chat>>,
    /// Featured order.
    pub featured_order: Option<i64>,
    /// The `estimateValue` flag (documented only as a boolean).
    pub estimate_value: Option<bool>,
    /// The `cantEstimate` flag (documented only as a boolean).
    pub cant_estimate: Option<bool>,
    /// Estimated value. The spec types this as a plain string.
    pub estimated_value: Option<String>,
    /// Templates. The keyset listing documents them as included only with
    /// `include_template=true`.
    pub templates: Option<Vec<Template>>,
    /// Main spreads line.
    pub spreads_main_line: Option<Decimal>,
    /// Main totals line.
    pub totals_main_line: Option<Decimal>,
    /// Carousel map. The spec types this as a plain string.
    pub carousel_map: Option<String>,
    /// Whether deployment is pending.
    pub pending_deployment: Option<bool>,
    /// Whether the event is being deployed.
    pub deploying: Option<bool>,
    /// When deployment started.
    pub deploying_timestamp: Option<DateTime<Utc>>,
    /// When deployment is scheduled.
    pub scheduled_deployment_timestamp: Option<DateTime<Utc>>,
    /// Game status. The spec types this as a plain string.
    pub game_status: Option<String>,
    /// Country name (undocumented; observed live on election events).
    pub country_name: Option<String>,
    /// The `cumulativeMarkets` flag (undocumented; observed live).
    pub cumulative_markets: Option<bool>,
    /// Election type, e.g. `Presidential` (undocumented; observed live on election events).
    pub election_type: Option<String>,
    /// Provider-specific metadata, a free-form object (undocumented; observed live with keys
    /// such as `context_requires_regen`, `opticOddsFixtureId` and `league`; the set of keys
    /// varies).
    pub event_metadata: Option<Value>,
    /// Game id (undocumented; observed live as an integer on sports events; the nested
    /// markets' [`game_id`](gamma.md#struct.Market) is documented as a string).
    pub game_id: Option<i64>,
    /// The `negRiskAugmented` flag (undocumented; observed live).
    pub neg_risk_augmented: Option<bool>,
    /// Id of the parent event (undocumented; observed live as an integer on sports
    /// sub-events; the documented [`parent_event`](gamma.md#struct.Event) is a string).
    pub parent_event_id: Option<i64>,
    /// The sport the event belongs to (undocumented; observed live on sports events, with the
    /// shape of an entry of [`GammaClient::get_sports_metadata`](gamma.md#GammaClient.fn.get_sports_metadata)).
    pub sport: Option<SportsMetadata>,
    /// Turn provider id (undocumented; observed live on a few events, as a string).
    pub turn_provider_id: Option<String>,
    /// Sports-data id, e.g. `nfl-pit-cle-2026-10-01` (undocumented; observed live on sports
    /// events).
    pub us_id: Option<String>,
    /// Data version, e.g. `v1` (undocumented; observed live).
    pub version: Option<String>,
    /// AMM liquidity (undocumented; observed live as a JSON number).
    pub liquidity_amm: Option<Decimal>,
    /// The teams of a sports event (undocumented; observed live).
    pub teams: Option<Vec<Team>>,
}
```

An event: a group of related markets (`components/schemas/Event`).

Every field is optional because the spec marks none as required. Fields the spec types
as `number` are `Decimal`s and serialize back as JSON numbers.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.EventCreator"></a>`struct EventCreator`

```rust
#[non_exhaustive]
pub struct EventCreator {
    /// Creator id.
    pub id: Option<EventCreatorId>,
    /// Creator name.
    pub creator_name: Option<String>,
    /// Creator handle.
    pub creator_handle: Option<String>,
    /// Creator URL.
    pub creator_url: Option<String>,
    /// Creator image URL.
    pub creator_image: Option<String>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    pub updated_at: Option<DateTime<Utc>>,
}
```

The creator of an event (`components/schemas/EventCreator`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.EventCreatorId"></a>`struct EventCreatorId`

```rust
pub struct EventCreatorId(/* private fields */);
```

An event creator id.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&EventCreatorId>`, `From<&String>`, `From<&str>`, `From<EventCreatorId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="EventCreatorId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="EventCreatorId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="EventCreatorId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.EventTweetCount"></a>`struct EventTweetCount`

```rust
#[non_exhaustive]
pub struct EventTweetCount {
    /// Number of tweets.
    pub tweet_count: Option<i64>,
}
```

An event's tweet count (`components/schemas/EventTweetCount`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.EventsKeysetPage"></a>`struct EventsKeysetPage`

```rust
#[non_exhaustive]
pub struct EventsKeysetPage {
    /// The events on this page (documented as an empty array if none were found).
    pub events: Option<Vec<Event>>,
    /// Cursor for the next page, passed to [`cursor`](gamma.md#ListEventsKeyset.fn.cursor). The spec
    /// documents it as present only when the number of returned events equals the
    /// effective limit, and omitted on the last page.
    pub next_cursor: Option<String>,
}
```

One page of [`GammaClient::list_events_keyset`](gamma.md#GammaClient.fn.list_events_keyset) (`components/schemas/KeysetEventsResponse`).

The fields keep their wire names; [`items`](gamma.md#EventsKeysetPage.fn.items),
[`into_items`](gamma.md#EventsKeysetPage.fn.into_items) and [`next_cursor()`](gamma.md#EventsKeysetPage.fn.next_cursor) give the
same view as every other page type.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="EventsKeysetPage.fn.items"></a>`items`

```rust
#[must_use]
pub fn items(&self) -> &[Event]
```

The events on this page (empty if the page carries none).

##### <a id="EventsKeysetPage.fn.into_items"></a>`into_items`

```rust
#[must_use]
pub fn into_items(self) -> Vec<Event>
```

Consumes the page and returns its events.

##### <a id="EventsKeysetPage.fn.next_cursor"></a>`next_cursor`

```rust
#[must_use]
pub fn next_cursor(&self) -> Option<&str>
```

The cursor for the next page, or `None` on the last page (an absent or empty
`next_cursor`).

### <a id="struct.EventsPage"></a>`struct EventsPage`

```rust
#[non_exhaustive]
pub struct EventsPage {
    /// The events on this page.
    pub data: Option<Vec<Event>>,
    /// Pagination metadata.
    pub pagination: Option<Pagination>,
}
```

One page of [`GammaClient::list_events_paginated`](gamma.md#GammaClient.fn.list_events_paginated) (`components/schemas/EventsPagination`).

The fields keep their wire names; [`items`](gamma.md#EventsPage.fn.items),
[`into_items`](gamma.md#EventsPage.fn.into_items) and [`has_more`](gamma.md#EventsPage.fn.has_more) give a uniform view.
The endpoint is offset-paginated, so there is no cursor.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="EventsPage.fn.items"></a>`items`

```rust
#[must_use]
pub fn items(&self) -> &[Event]
```

The events on this page (empty if the page carries none).

##### <a id="EventsPage.fn.into_items"></a>`into_items`

```rust
#[must_use]
pub fn into_items(self) -> Vec<Event>
```

Consumes the page and returns its events.

##### <a id="EventsPage.fn.has_more"></a>`has_more`

```rust
#[must_use]
pub fn has_more(&self) -> Option<bool>
```

The page's `pagination.hasMore`, or `None` if the server did not send it.

### <a id="struct.FeeSchedule"></a>`struct FeeSchedule`

```rust
#[non_exhaustive]
pub struct FeeSchedule {
    /// Exponent.
    pub exponent: Option<Decimal>,
    /// Rate.
    pub rate: Option<Decimal>,
    /// Whether only takers pay the fee.
    pub taker_only: Option<bool>,
    /// Rebate rate.
    pub rebate_rate: Option<Decimal>,
}
```

A market's fee schedule (`components/schemas/FeeSchedule`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.GammaClient"></a>`struct GammaClient`

```rust
pub struct GammaClient { /* private fields */ }
```

Client for the Gamma API (`https://gamma-api.polymarket.com`).

Covers events, markets, tags, series, comments, sports, search and public profiles. Cheap to clone: clones share one connection pool.

```rust
use marcasite::gamma::GammaClient;

let client = GammaClient::new()?;
```

**Implements:** `Clone`, `Debug`

#### Associated items

##### <a id="GammaClient.constant.DEFAULT_BASE_URL"></a>`DEFAULT_BASE_URL`

```rust
pub const DEFAULT_BASE_URL: &'static str = "https://gamma-api.polymarket.com";
```

The production base URL.

##### <a id="GammaClient.fn.new"></a>`new`

```rust
pub fn new() -> Result<Self>
```

Creates a client with the default HTTP settings and base URL.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the HTTP client cannot be built.

##### <a id="GammaClient.fn.builder"></a>`builder`

```rust
pub fn builder() -> GammaClientBuilder
```

Returns a builder for setting a custom base URL or HTTP client.

##### <a id="GammaClient.fn.base_url"></a>`base_url`

```rust
#[must_use]
pub fn base_url(&self) -> &Url
```

The base URL requests are sent to.

##### <a id="GammaClient.fn.list_comments"></a>`list_comments`

```rust
pub fn list_comments(&self, parent_entity_type: impl Into<CommentParentEntityType>, parent_entity_id: i64) -> ListComments
```

Lists the comments on one entity (offset pagination).

`parent_entity_type` and `parent_entity_id` are required: live answers a request
without either with a `422`. The spec documents neither as required, and lists
`market` (not `PerpsAsset`) as a type; see `SPEC_DEVIATIONS.md`.

See <https://docs.polymarket.com/api-reference/comments/list-comments>.

```rust
use marcasite::gamma::CommentParentEntityType;

let gamma = marcasite::gamma::GammaClient::new()?;
let comments = gamma
    .list_comments(CommentParentEntityType::Event, 16167)
    .limit(20)
    .send()
    .await?;
```

##### <a id="GammaClient.fn.get_comments_by_id"></a>`get_comments_by_id`

```rust
pub fn get_comments_by_id(&self, id: impl Into<CommentId>) -> GetCommentsById
```

Gets comments by comment id (`GET /comments/{id}`).

The docs do not describe which comments the list holds beyond the summary "Get
comments by comment id".

See <https://docs.polymarket.com/api-reference/comments/get-comments-by-comment-id>.

##### <a id="GammaClient.fn.list_comments_by_user"></a>`list_comments_by_user`

```rust
pub fn list_comments_by_user(&self, user_address: impl Into<Address>) -> ListCommentsByUser
```

Lists the comments of a user address (offset pagination).

The spec types `user_address` as a plain string with no pattern, so it is not
checked against the EVM address pattern; it must only be non-empty and not `.` or
`..`.

See <https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address>.

##### <a id="GammaClient.fn.list_events"></a>`list_events`

```rust
pub fn list_events(&self) -> ListEvents
```

Lists events (offset pagination).

See <https://docs.polymarket.com/api-reference/events/list-events>.

```rust
let gamma = marcasite::gamma::GammaClient::new()?;
let events = gamma
    .list_events()
    .active(true)
    .closed(false)
    .limit(20)
    .send()
    .await?;
for event in events {
    println!("{:?}: {} markets", event.title, event.markets.map_or(0, |m| m.len()));
}
```

##### <a id="GammaClient.fn.list_events_paginated"></a>`list_events_paginated`

```rust
pub fn list_events_paginated(&self) -> ListEventsPaginated
```

Lists events with pagination metadata (`hasMore`, `totalResults`; offset
pagination).

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `listEventsPagination` (no published
doc page).

##### <a id="GammaClient.fn.list_sport_event_results"></a>`list_sport_event_results`

```rust
pub fn list_sport_event_results(&self) -> ListSportEventResults
```

Lists sport events with their results (offset pagination).

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `listSportEventsResults` (no
published doc page).

##### <a id="GammaClient.fn.get_event"></a>`get_event`

```rust
pub fn get_event(&self, id: impl Into<EventId>) -> GetEvent
```

Gets an event by id.

See <https://docs.polymarket.com/api-reference/events/get-event-by-id>.

```rust
let gamma = marcasite::gamma::GammaClient::new()?;
let event = gamma.get_event("16167").send().await?;
for market in event.markets.unwrap_or_default() {
    println!("{:?}", market.question);
}
```

##### <a id="GammaClient.fn.get_event_by_slug"></a>`get_event_by_slug`

```rust
pub fn get_event_by_slug(&self, slug: impl Into<String>) -> GetEvent
```

Gets an event by slug.

See <https://docs.polymarket.com/api-reference/events/get-event-by-slug>.

##### <a id="GammaClient.fn.get_event_tags"></a>`get_event_tags`

```rust
pub async fn get_event_tags(&self, id: impl Into<EventId>) -> Result<Vec<Tag>>
```

Gets the tags attached to an event.

See <https://docs.polymarket.com/api-reference/events/get-event-tags>.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if `id` is not an
  integer (one or more ASCII digits), checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the event does not exist;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.get_event_tweet_count"></a>`get_event_tweet_count`

```rust
pub async fn get_event_tweet_count(&self, id: impl Into<EventId>) -> Result<EventTweetCount>
```

Gets an event's tweet count.

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getEventTweetCount` (no published
doc page).

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if `id` is not an
  integer (one or more ASCII digits), checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the event does not exist;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.get_event_comment_count"></a>`get_event_comment_count`

```rust
pub async fn get_event_comment_count(&self, id: impl Into<EventId>) -> Result<CommentCount>
```

Gets the number of comments on an event.

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getEventCommentsCount` (no
published doc page).

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if `id` is not an
  integer (one or more ASCII digits), checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the event does not exist;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.list_event_creators"></a>`list_event_creators`

```rust
pub fn list_event_creators(&self) -> ListEventCreators
```

Lists event creators (offset pagination).

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `listEventCreators` (no published
doc page).

##### <a id="GammaClient.fn.get_event_creator"></a>`get_event_creator`

```rust
pub async fn get_event_creator(&self, id: impl Into<EventCreatorId>) -> Result<EventCreator>
```

Gets an event creator by id.

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getEventCreator` (no published doc
page).

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if `id` is not an
  integer (one or more ASCII digits), checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the creator does not exist;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.list_events_keyset"></a>`list_events_keyset`

```rust
pub fn list_events_keyset(&self) -> ListEventsKeyset
```

Lists events with cursor-based (keyset) pagination, for stable paging through large
result sets.

[`send`](gamma.md#ListEventsKeyset.fn.send) returns one [`EventsKeysetPage`](gamma.md#struct.EventsKeysetPage); pass its
[`next_cursor()`](gamma.md#EventsKeysetPage.fn.next_cursor) to
[`cursor`](gamma.md#ListEventsKeyset.fn.cursor) for the next page, or use
[`into_stream`](gamma.md#ListEventsKeyset.fn.into_stream) to walk every page.

The spec's response description mentions relations that the documented `Event` and
`Market` schemas do not contain: `BestLines` (see
[`include_best_lines`](gamma.md#ListEventsKeyset.fn.include_best_lines)), `external_partners`
(see [`partner_slug`](gamma.md#ListEventsKeyset.fn.partner_slug)), `Teams`, and the nested
markets' `clob_rewards`. That data is not modelled, so it is dropped when decoding.
The description also spells the markets' fee schedule `fee_schedule`, while the
schema spells it `feeSchedule` ([`Market::fee_schedule`](gamma.md#struct.Market)).

See <https://docs.polymarket.com/api-reference/events/list-events-keyset-pagination>.

```rust
let gamma = marcasite::gamma::GammaClient::new()?;
let first = gamma.list_events_keyset().limit(50).closed(false).send().await?;
if let Some(cursor) = first.next_cursor() {
    let second = gamma
        .list_events_keyset()
        .limit(50)
        .closed(false)
        .cursor(cursor)
        .send()
        .await?;
}
```

##### <a id="GammaClient.fn.list_markets"></a>`list_markets`

```rust
pub fn list_markets(&self) -> ListMarkets
```

Lists markets (offset pagination).

The spec documents the `closed` filter's default as `false`.

See <https://docs.polymarket.com/api-reference/markets/list-markets>.

```rust
let gamma = marcasite::gamma::GammaClient::new()?;
let markets = gamma.list_markets().limit(10).closed(false).send().await?;
for market in markets {
    println!("{:?}: {:?}", market.question, market.volume_num);
}
```

##### <a id="GammaClient.fn.get_market"></a>`get_market`

```rust
pub fn get_market(&self, id: impl Into<MarketId>) -> GetMarket
```

Gets a market by id.

See <https://docs.polymarket.com/api-reference/markets/get-market-by-id>.

```rust
let gamma = marcasite::gamma::GammaClient::new()?;
let market = gamma.get_market("239826").include_tag(true).send().await?;
println!("{:?} closes at {:?}", market.question, market.end_date);
```

##### <a id="GammaClient.fn.get_market_by_slug"></a>`get_market_by_slug`

```rust
pub fn get_market_by_slug(&self, slug: impl Into<String>) -> GetMarket
```

Gets a market by slug.

See <https://docs.polymarket.com/api-reference/markets/get-market-by-slug>.

##### <a id="GammaClient.fn.get_market_tags"></a>`get_market_tags`

```rust
pub async fn get_market_tags(&self, id: impl Into<MarketId>) -> Result<Vec<Tag>>
```

Gets the tags attached to a market.

See <https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id>.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if `id` is not an
  integer (one or more ASCII digits), checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the market does not exist;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.get_market_description"></a>`get_market_description`

```rust
pub async fn get_market_description(&self, id: impl Into<MarketId>) -> Result<MarketDescription>
```

Gets a market's description.

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getMarketDescription` (no
published doc page).

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if `id` is not an
  integer (one or more ASCII digits), checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the market does not exist;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.list_markets_keyset"></a>`list_markets_keyset`

```rust
pub fn list_markets_keyset(&self) -> ListMarketsKeyset
```

Lists markets with cursor-based (keyset) pagination, for stable paging through large
result sets.

[`send`](gamma.md#ListMarketsKeyset.fn.send) returns one [`MarketsKeysetPage`](gamma.md#struct.MarketsKeysetPage); pass its
[`next_cursor()`](gamma.md#MarketsKeysetPage.fn.next_cursor) to
[`cursor`](gamma.md#ListMarketsKeyset.fn.cursor) for the next page, or use
[`into_stream`](gamma.md#ListMarketsKeyset.fn.into_stream) to walk every page.

The spec's response description says nested `clob_rewards` and `fee_schedule` are
populated on each market. The documented `Market` schema has no `clob_rewards`
property, so that data is not modelled (it is dropped when decoding), and it spells
the fee schedule `feeSchedule` ([`Market::fee_schedule`](gamma.md#struct.Market)).

See <https://docs.polymarket.com/api-reference/markets/list-markets-keyset-pagination>.

```rust
use futures_util::{StreamExt as _, TryStreamExt as _};

let gamma = marcasite::gamma::GammaClient::new()?;
let markets: Vec<_> = gamma
    .list_markets_keyset()
    .limit(100)
    .into_stream()
    .take(250)
    .try_collect()
    .await?;
```

##### <a id="GammaClient.fn.get_markets_information"></a>`get_markets_information`

```rust
pub fn get_markets_information(&self) -> GetMarketsInformation
```

Queries markets by information filters sent as a JSON body
(`POST /markets/information`).

[`get_abridged_markets`](gamma.md#GammaClient.fn.get_abridged_markets) takes the same filters on
`POST /markets/abridged`.

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getMarketsInformation` (no
published doc page).

##### <a id="GammaClient.fn.get_abridged_markets"></a>`get_abridged_markets`

```rust
pub fn get_abridged_markets(&self) -> GetMarketsInformation
```

Queries abridged markets by information filters sent as a JSON body
(`POST /markets/abridged`).

Takes the same filters and returns the same [`Market`](gamma.md#struct.Market) schema as
[`get_markets_information`](gamma.md#GammaClient.fn.get_markets_information) (`POST
/markets/information`).

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getAbridgedMarkets` (no published
doc page).

##### <a id="GammaClient.fn.get_public_profile"></a>`get_public_profile`

```rust
pub async fn get_public_profile(&self, address: impl Into<Address>) -> Result<PublicProfile>
```

Gets the public profile of a wallet address (proxy wallet or user address).

See <https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address>.

```rust
let gamma = marcasite::gamma::GammaClient::new()?;
let profile = gamma
    .get_public_profile("0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b")
    .await?;
println!("{:?}", profile.name);
```

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if `address` does not match the
  documented pattern `^0x[a-fA-F0-9]{40}$` (checked before sending);
- [`Error::Api`](marcasite.md#enum.Error) with status `400` (`type` `"validation error"`)
  for an address the server rejects, or `404` (`type` `"not found error"`) if no
  profile exists;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.get_profile"></a>`get_profile`

```rust
pub async fn get_profile(&self, user_address: impl Into<Address>) -> Result<Profile>
```

Gets the profile of a user address.

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getPublicProfileByUserAddress` (no
published doc page).

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if `user_address` does not match
  the documented pattern `^0x[a-fA-F0-9]{40}$` (checked before sending);
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if no profile exists;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.search"></a>`search`

```rust
pub fn search(&self, q: impl Into<String>) -> Search
```

Searches markets, events and profiles (`GET /public-search`).

`q` is the (required) search text. The endpoint is paginated with
[`page`](gamma.md#Search.fn.page) and reports `pagination.hasMore` (`hasMore` at the top level
with [`optimized`](gamma.md#Search.fn.optimized)); there is no `into_stream()` because the docs do
not say whether pages are numbered from 0 or 1.

See <https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles>.

```rust
let gamma = marcasite::gamma::GammaClient::new()?;
let results = gamma
    .search("election")
    .limit_per_type(5)
    .search_profiles(true)
    .send()
    .await?;
for event in results.events.unwrap_or_default() {
    println!("{:?}", event.title);
}
```

##### <a id="GammaClient.fn.list_series"></a>`list_series`

```rust
pub fn list_series(&self) -> ListSeries
```

Lists series (offset pagination).

See <https://docs.polymarket.com/api-reference/series/list-series>.

```rust
let gamma = marcasite::gamma::GammaClient::new()?;
let series = gamma.list_series().closed(false).limit(10).send().await?;
```

##### <a id="GammaClient.fn.get_series"></a>`get_series`

```rust
pub fn get_series(&self, id: impl Into<SeriesId>) -> GetSeries
```

Gets a series by id.

See <https://docs.polymarket.com/api-reference/series/get-series-by-id>.

##### <a id="GammaClient.fn.get_series_comment_count"></a>`get_series_comment_count`

```rust
pub async fn get_series_comment_count(&self, id: impl Into<SeriesId>) -> Result<CommentCount>
```

Gets the number of comments on a series.

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getSeriesCommentsCount` (no
published doc page).

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if `id` is not an
  integer (one or more ASCII digits), checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the series does not exist;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.get_series_summary"></a>`get_series_summary`

```rust
pub async fn get_series_summary(&self, id: impl Into<SeriesId>) -> Result<SeriesSummary>
```

Gets a series summary by series id.

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getSeriesSummaryById` (no published
doc page).

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if `id` is not an
  integer (one or more ASCII digits), checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the series does not exist;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.get_series_summary_by_slug"></a>`get_series_summary_by_slug`

```rust
pub async fn get_series_summary_by_slug(&self, slug: impl Into<String>) -> Result<SeriesSummary>
```

Gets a series summary by series slug.

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getSeriesSummaryBySlug` (no
published doc page).

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `slug`) if `slug` is
  empty, `.` or `..`, checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the series does not exist;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.list_teams"></a>`list_teams`

```rust
pub fn list_teams(&self) -> ListTeams
```

Lists sports teams (offset pagination).

See <https://docs.polymarket.com/api-reference/sports/list-teams>.

##### <a id="GammaClient.fn.get_team"></a>`get_team`

```rust
pub async fn get_team(&self, id: impl Into<TeamId>) -> Result<Team>
```

Gets a sports team by id.

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getTeam` (no published doc page).

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if `id` is not an
  integer (one or more ASCII digits), checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the team does not exist;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.get_sports_metadata"></a>`get_sports_metadata`

```rust
pub async fn get_sports_metadata(&self) -> Result<Vec<SportsMetadata>>
```

Gets the configuration of every sport: images, resolution sources and related tag
and series identifiers.

See <https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information>.

###### Errors

See [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.get_sports_market_types"></a>`get_sports_market_types`

```rust
pub async fn get_sports_market_types(&self) -> Result<SportsMarketTypes>
```

Gets the valid sports market types (the values of
[`Market::sports_market_type`](gamma.md#struct.Market)).

See <https://docs.polymarket.com/api-reference/sports/get-valid-sports-market-types>.

###### Errors

See [`Error`](marcasite.md#enum.Error).

##### <a id="GammaClient.fn.get_status"></a>`get_status`

```rust
pub async fn get_status(&self) -> Result<String>
```

Checks the health of the Gamma API (`GET /status`).

Returns the plain-text body of a successful response (the spec's example is `"OK"`).

See `docs/polymarket/specs/gamma-openapi.yaml`, operationId `getGammaStatus` (no published doc
page).

```rust
let gamma = marcasite::gamma::GammaClient::new()?;
let status = gamma.get_status().await?;
println!("Gamma API status: {status}");
```

###### Errors

The spec documents no error response for this endpoint; see
[`Error`](marcasite.md#enum.Error) for the failures every request can have.

##### <a id="GammaClient.fn.list_tags"></a>`list_tags`

```rust
pub fn list_tags(&self) -> ListTags
```

Lists tags (offset pagination).

See <https://docs.polymarket.com/api-reference/tags/list-tags>.

```rust
let gamma = marcasite::gamma::GammaClient::new()?;
let tags = gamma.list_tags().limit(20).ascending(true).send().await?;
```

##### <a id="GammaClient.fn.get_tag"></a>`get_tag`

```rust
pub fn get_tag(&self, id: impl Into<TagId>) -> GetTag
```

Gets a tag by id.

See <https://docs.polymarket.com/api-reference/tags/get-tag-by-id>.

##### <a id="GammaClient.fn.get_tag_by_slug"></a>`get_tag_by_slug`

```rust
pub fn get_tag_by_slug(&self, slug: impl Into<String>) -> GetTag
```

Gets a tag by slug.

See <https://docs.polymarket.com/api-reference/tags/get-tag-by-slug>.

##### <a id="GammaClient.fn.get_related_tag_relationships"></a>`get_related_tag_relationships`

```rust
pub fn get_related_tag_relationships(&self, id: impl Into<TagId>) -> GetRelatedTagRelationships
```

Gets the relationships between a tag (by id) and its related tags.

See <https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id>.

##### <a id="GammaClient.fn.get_related_tag_relationships_by_slug"></a>`get_related_tag_relationships_by_slug`

```rust
pub fn get_related_tag_relationships_by_slug(&self, slug: impl Into<String>) -> GetRelatedTagRelationships
```

Gets the relationships between a tag (by slug) and its related tags.

See <https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-slug>.

##### <a id="GammaClient.fn.get_related_tags"></a>`get_related_tags`

```rust
pub fn get_related_tags(&self, id: impl Into<TagId>) -> GetRelatedTags
```

Gets the tags related to a tag (by id).

See <https://docs.polymarket.com/api-reference/tags/get-tags-related-to-a-tag-id>.

##### <a id="GammaClient.fn.get_related_tags_by_slug"></a>`get_related_tags_by_slug`

```rust
pub fn get_related_tags_by_slug(&self, slug: impl Into<String>) -> GetRelatedTags
```

Gets the tags related to a tag (by slug).

See <https://docs.polymarket.com/api-reference/tags/get-tags-related-to-a-tag-slug>.

### <a id="struct.GammaClientBuilder"></a>`struct GammaClientBuilder`

```rust
#[must_use]
pub struct GammaClientBuilder { /* private fields */ }
```

Builder for [`GammaClient`](gamma.md#struct.GammaClient).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="GammaClientBuilder.fn.base_url"></a>`base_url`

```rust
pub fn base_url(self, url: impl Into<String>) -> Self
```

Overrides the base URL (default [`GammaClient::DEFAULT_BASE_URL`](gamma.md#GammaClient.constant.DEFAULT_BASE_URL)), e.g. to target a
mock server in tests. A path prefix is preserved.

##### <a id="GammaClientBuilder.fn.http_client"></a>`http_client`

```rust
pub fn http_client(self, http: HttpClient) -> Self
```

Uses an existing [`HttpClient`](marcasite.md#struct.HttpClient) (and its timeouts, user agent and retry policy).

##### <a id="GammaClientBuilder.fn.build"></a>`build`

```rust
pub fn build(self) -> Result<GammaClient>
```

Builds the client.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the base URL is invalid or the HTTP
client cannot be built.

### <a id="struct.GetCommentsById"></a>`struct GetCommentsById`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetCommentsById { /* private fields */ }
```

Request builder for [`GammaClient::get_comments_by_id`](gamma.md#GammaClient.fn.get_comments_by_id).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetCommentsById.fn.get_positions"></a>`get_positions`

```rust
pub fn get_positions(self, get_positions: bool) -> Self
```

The `get_positions` flag (documented only as a boolean).

##### <a id="GetCommentsById.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Comment>>
```

Sends the request.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if the id is not
  an integer (one or more ASCII digits), checked before sending;
- otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetEvent"></a>`struct GetEvent`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetEvent { /* private fields */ }
```

Request builder for [`GammaClient::get_event`](gamma.md#GammaClient.fn.get_event) and [`GammaClient::get_event_by_slug`](gamma.md#GammaClient.fn.get_event_by_slug).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetEvent.fn.include_chat"></a>`include_chat`

```rust
pub fn include_chat(self, include_chat: bool) -> Self
```

The `include_chat` flag (documented only as a boolean on this endpoint).

##### <a id="GetEvent.fn.include_template"></a>`include_template`

```rust
pub fn include_template(self, include_template: bool) -> Self
```

The `include_template` flag (documented only as a boolean on this endpoint).

##### <a id="GetEvent.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Event>
```

Sends the request.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error), checked before sending, if the id
  is not an integer (parameter `id`) or the slug is empty, `.` or `..` (parameter
  `slug`);
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the event does not exist
  (see [`Error::is_not_found`](marcasite.md#Error.fn.is_not_found));
- otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetMarket"></a>`struct GetMarket`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetMarket { /* private fields */ }
```

Request builder for [`GammaClient::get_market`](gamma.md#GammaClient.fn.get_market) and
[`GammaClient::get_market_by_slug`](gamma.md#GammaClient.fn.get_market_by_slug).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetMarket.fn.include_tag"></a>`include_tag`

```rust
pub fn include_tag(self, include_tag: bool) -> Self
```

The `include_tag` flag (documented only as a boolean on this endpoint).

##### <a id="GetMarket.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Market>
```

Sends the request.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error), checked before sending, if the id
  is not an integer (parameter `id`) or the slug is empty, `.` or `..` (parameter
  `slug`);
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the market does not exist
  (see [`Error::is_not_found`](marcasite.md#Error.fn.is_not_found));
- otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetMarketsInformation"></a>`struct GetMarketsInformation`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetMarketsInformation { /* private fields */ }
```

Request builder for [`GammaClient::get_markets_information`](gamma.md#GammaClient.fn.get_markets_information)
(`POST /markets/information`) and [`GammaClient::get_abridged_markets`](gamma.md#GammaClient.fn.get_abridged_markets)
(`POST /markets/abridged`).

Every setter maps to one field of the JSON body
(`components/schemas/MarketsInformationBody`); unset fields are omitted.

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetMarketsInformation.fn.ids"></a>`ids`

```rust
pub fn ids<I>(self, ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<MarketId>,
```

Filter by market ids (`id`). The spec types them as integers, so an id that is
 not one or more ASCII digits (or does not fit in an `i64`) is rejected before
 sending with [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="GetMarketsInformation.fn.slugs"></a>`slugs`

```rust
pub fn slugs<I>(self, slugs: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by slugs (`slug`).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="GetMarketsInformation.fn.closed"></a>`closed`

```rust
pub fn closed(self, closed: bool) -> Self
```

The `closed` filter (documented only as a nullable boolean).

##### <a id="GetMarketsInformation.fn.clob_token_ids"></a>`clob_token_ids`

```rust
pub fn clob_token_ids<I>(self, clob_token_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

Filter by CLOB token ids (`clobTokenIds`).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="GetMarketsInformation.fn.condition_ids"></a>`condition_ids`

```rust
pub fn condition_ids<I>(self, condition_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Filter by condition ids (`conditionIds`).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="GetMarketsInformation.fn.liquidity_num_min"></a>`liquidity_num_min`

```rust
pub fn liquidity_num_min(self, liquidity_num_min: impl Into<Decimal>) -> Self
```

Minimum liquidity (`liquidityNumMin`).

##### <a id="GetMarketsInformation.fn.liquidity_num_max"></a>`liquidity_num_max`

```rust
pub fn liquidity_num_max(self, liquidity_num_max: impl Into<Decimal>) -> Self
```

Maximum liquidity (`liquidityNumMax`).

##### <a id="GetMarketsInformation.fn.volume_num_min"></a>`volume_num_min`

```rust
pub fn volume_num_min(self, volume_num_min: impl Into<Decimal>) -> Self
```

Minimum volume (`volumeNumMin`).

##### <a id="GetMarketsInformation.fn.volume_num_max"></a>`volume_num_max`

```rust
pub fn volume_num_max(self, volume_num_max: impl Into<Decimal>) -> Self
```

Maximum volume (`volumeNumMax`).

##### <a id="GetMarketsInformation.fn.start_date_min"></a>`start_date_min`

```rust
pub fn start_date_min(self, start_date_min: DateTime<Utc>) -> Self
```

Earliest start date (`startDateMin`).

##### <a id="GetMarketsInformation.fn.start_date_max"></a>`start_date_max`

```rust
pub fn start_date_max(self, start_date_max: DateTime<Utc>) -> Self
```

Latest start date (`startDateMax`).

##### <a id="GetMarketsInformation.fn.end_date_min"></a>`end_date_min`

```rust
pub fn end_date_min(self, end_date_min: DateTime<Utc>) -> Self
```

Earliest end date (`endDateMin`).

##### <a id="GetMarketsInformation.fn.end_date_max"></a>`end_date_max`

```rust
pub fn end_date_max(self, end_date_max: DateTime<Utc>) -> Self
```

Latest end date (`endDateMax`).

##### <a id="GetMarketsInformation.fn.related_tags"></a>`related_tags`

```rust
pub fn related_tags(self, related_tags: bool) -> Self
```

The `relatedTags` flag (documented only as a nullable boolean).

##### <a id="GetMarketsInformation.fn.tag_id"></a>`tag_id`

```rust
pub fn tag_id(self, tag_id: impl Into<TagId>) -> Self
```

Filter by tag id (`tagId`). The spec types it as an integer, so an id that is not
one or more ASCII digits (or does not fit in an `i64`) is rejected before sending
with [`Error::Validation`](marcasite.md#enum.Error).

##### <a id="GetMarketsInformation.fn.cyom"></a>`cyom`

```rust
pub fn cyom(self, cyom: bool) -> Self
```

The `cyom` filter (documented only as a nullable boolean).

##### <a id="GetMarketsInformation.fn.uma_resolution_status"></a>`uma_resolution_status`

```rust
pub fn uma_resolution_status(self, uma_resolution_status: impl Into<String>) -> Self
```

Filter by UMA resolution status (`umaResolutionStatus`; the spec documents no
values).

##### <a id="GetMarketsInformation.fn.game_id"></a>`game_id`

```rust
pub fn game_id(self, game_id: impl Into<String>) -> Self
```

Filter by game id (`gameId`).

##### <a id="GetMarketsInformation.fn.sports_market_types"></a>`sports_market_types`

```rust
pub fn sports_market_types<I>(self, sports_market_types: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by sports market types (`sportsMarketTypes`).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="GetMarketsInformation.fn.rewards_min_size"></a>`rewards_min_size`

```rust
pub fn rewards_min_size(self, rewards_min_size: impl Into<Decimal>) -> Self
```

Minimum size for liquidity rewards (`rewardsMinSize`).

##### <a id="GetMarketsInformation.fn.question_ids"></a>`question_ids`

```rust
pub fn question_ids<I>(self, question_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<QuestionId>,
```

Filter by question ids (`questionIds`).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="GetMarketsInformation.fn.include_tags"></a>`include_tags`

```rust
pub fn include_tags(self, include_tags: bool) -> Self
```

The `includeTags` flag (documented only as a nullable boolean).

##### <a id="GetMarketsInformation.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Market>>
```

Sends the request.

The endpoint is read-only, so it is retried like a `GET` on transient failures.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error), checked before sending, if an
  [`ids`](gamma.md#GetMarketsInformation.fn.ids) entry (parameter `id`) or [`tag_id`](gamma.md#GetMarketsInformation.fn.tag_id) (parameter
  `tagId`) is not an integer;
- [`Error::Api`](marcasite.md#enum.Error) with status `422` for a body the server rejects
  (documented only as "Validation error");
- otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetRelatedTagRelationships"></a>`struct GetRelatedTagRelationships`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetRelatedTagRelationships { /* private fields */ }
```

Request builder for [`GammaClient::get_related_tag_relationships`](gamma.md#GammaClient.fn.get_related_tag_relationships) and
[`GammaClient::get_related_tag_relationships_by_slug`](gamma.md#GammaClient.fn.get_related_tag_relationships_by_slug).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetRelatedTagRelationships.fn.omit_empty"></a>`omit_empty`

```rust
pub fn omit_empty(self, omit_empty: bool) -> Self
```

The `omit_empty` flag (documented only as a boolean).

##### <a id="GetRelatedTagRelationships.fn.status"></a>`status`

```rust
pub fn status(self, status: RelatedTagsStatus) -> Self
```

The `status` filter.

##### <a id="GetRelatedTagRelationships.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<RelatedTag>>
```

Sends the request.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error), checked before sending, if the id
  is not an integer (parameter `id`) or the slug is empty, `.` or `..` (parameter
  `slug`);
- otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetRelatedTags"></a>`struct GetRelatedTags`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetRelatedTags { /* private fields */ }
```

Request builder for [`GammaClient::get_related_tags`](gamma.md#GammaClient.fn.get_related_tags) and
[`GammaClient::get_related_tags_by_slug`](gamma.md#GammaClient.fn.get_related_tags_by_slug).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetRelatedTags.fn.omit_empty"></a>`omit_empty`

```rust
pub fn omit_empty(self, omit_empty: bool) -> Self
```

The `omit_empty` flag (documented only as a boolean).

##### <a id="GetRelatedTags.fn.status"></a>`status`

```rust
pub fn status(self, status: RelatedTagsStatus) -> Self
```

The `status` filter.

##### <a id="GetRelatedTags.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Tag>>
```

Sends the request.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error), checked before sending, if the id
  is not an integer (parameter `id`) or the slug is empty, `.` or `..` (parameter
  `slug`);
- otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetSeries"></a>`struct GetSeries`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetSeries { /* private fields */ }
```

Request builder for [`GammaClient::get_series`](gamma.md#GammaClient.fn.get_series).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetSeries.fn.include_chat"></a>`include_chat`

```rust
pub fn include_chat(self, include_chat: bool) -> Self
```

The `include_chat` flag (documented only as a boolean on this endpoint).

##### <a id="GetSeries.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Series>
```

Sends the request.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `id`) if the id is not
  an integer (one or more ASCII digits), checked before sending;
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the series does not exist
  (see [`Error::is_not_found`](marcasite.md#Error.fn.is_not_found));
- otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetTag"></a>`struct GetTag`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetTag { /* private fields */ }
```

Request builder for [`GammaClient::get_tag`](gamma.md#GammaClient.fn.get_tag) and [`GammaClient::get_tag_by_slug`](gamma.md#GammaClient.fn.get_tag_by_slug).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetTag.fn.include_template"></a>`include_template`

```rust
pub fn include_template(self, include_template: bool) -> Self
```

The `include_template` flag (documented only as a boolean).

##### <a id="GetTag.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Tag>
```

Sends the request.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error), checked before sending, if the id
  is not an integer (parameter `id`) or the slug is empty, `.` or `..` (parameter
  `slug`);
- [`Error::Api`](marcasite.md#enum.Error) with status `404` if the tag does not exist (see
  [`Error::is_not_found`](marcasite.md#Error.fn.is_not_found));
- otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.ImageOptimization"></a>`struct ImageOptimization`

```rust
#[non_exhaustive]
pub struct ImageOptimization {
    /// Record id.
    pub id: Option<String>,
    /// URL of the source image.
    pub image_url_source: Option<String>,
    /// URL of the optimized image.
    pub image_url_optimized: Option<String>,
    /// Size of the source image in KB.
    pub image_size_kb_source: Option<Decimal>,
    /// Size of the optimized image in KB.
    pub image_size_kb_optimized: Option<Decimal>,
    /// Whether optimization has completed.
    pub image_optimized_complete: Option<bool>,
    /// When the optimized image was last updated. The spec types this as a plain string
    /// with no format.
    pub image_optimized_last_updated: Option<String>,
    /// Id of the related record (wire name `relID`).
    pub rel_id: Option<i64>,
    /// Name of the field on the related record that holds the image.
    pub field: Option<String>,
    /// Name of the related record type.
    pub relname: Option<String>,
}
```

Metadata about an optimized copy of an image (`components/schemas/ImageOptimization`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ListComments"></a>`struct ListComments`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListComments { /* private fields */ }
```

Request builder for [`GammaClient::list_comments`](gamma.md#GammaClient.fn.list_comments).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListComments.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of comments per page (`limit`; the docs give a minimum of `0` and
no maximum).

##### <a id="ListComments.fn.offset"></a>`offset`

```rust
pub fn offset(self, offset: u32) -> Self
```

Number of comments to skip (`offset`). Live rejects values above 200 (a `422`;
the listing is meant to be walked with `/comments/keyset`, which this crate does
not wrap), so larger values fail client-side with
[`Error::Validation`](marcasite.md#enum.Error).

##### <a id="ListComments.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `createdAt` or `reactionCount`); snake_case names
such as `start_date` are rejected with a `422` (`order fields are not valid`), although
the spec's keyset example uses them. See `SPEC_DEVIATIONS.md`.

##### <a id="ListComments.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="ListComments.fn.get_positions"></a>`get_positions`

```rust
pub fn get_positions(self, get_positions: bool) -> Self
```

The `get_positions` flag (documented only as a boolean).

##### <a id="ListComments.fn.holders_only"></a>`holders_only`

```rust
pub fn holders_only(self, holders_only: bool) -> Self
```

The `holders_only` filter (documented only as a boolean).

##### <a id="ListComments.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Comment>>
```

Fetches one page.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `offset`) if
  [`offset`](gamma.md#ListComments.fn.offset) exceeds 200, checked before sending;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListComments.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Comment>
```

Streams every comment from the configured offset onwards, fetching pages lazily.

The stream ends at the first empty page, or right after yielding the first error.
A page shorter than [`limit`](gamma.md#ListComments.fn.limit) does not end it, because the docs give no
maximum `limit` and the server may return fewer comments, so the last request
returns an empty page.

Live rejects offsets above 200: past that the stream yields one
[`Error::Validation`](marcasite.md#enum.Error) (parameter `offset`, without sending a
request) and ends.

### <a id="struct.ListCommentsByUser"></a>`struct ListCommentsByUser`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListCommentsByUser { /* private fields */ }
```

Request builder for [`GammaClient::list_comments_by_user`](gamma.md#GammaClient.fn.list_comments_by_user).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListCommentsByUser.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of comments per page (`limit`; the docs give a minimum of `0` and
no maximum).

##### <a id="ListCommentsByUser.fn.offset"></a>`offset`

```rust
pub fn offset(self, offset: u32) -> Self
```

Number of comments to skip (`offset`). Live rejects values above 200 (a `422`;
the listing is meant to be walked with `/comments/keyset`, which this crate does
not wrap), so larger values fail client-side with
[`Error::Validation`](marcasite.md#enum.Error).

##### <a id="ListCommentsByUser.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `createdAt` or `reactionCount`); snake_case names
such as `start_date` are rejected with a `422` (`order fields are not valid`), although
the spec's keyset example uses them. See `SPEC_DEVIATIONS.md`.

##### <a id="ListCommentsByUser.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="ListCommentsByUser.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Comment>>
```

Fetches one page.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `user_address`) if the
  address is empty, `.` or `..`, or (parameter `offset`) if [`offset`](gamma.md#ListCommentsByUser.fn.offset)
  exceeds 200, checked before sending;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListCommentsByUser.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Comment>
```

Streams every comment from the configured offset onwards, fetching pages lazily.

The stream ends at the first empty page, or right after yielding the first error.
A page shorter than [`limit`](gamma.md#ListCommentsByUser.fn.limit) does not end it, because the docs give no
maximum `limit` and the server may return fewer comments, so the last request
returns an empty page.

Live rejects offsets above 200: past that the stream yields one
[`Error::Validation`](marcasite.md#enum.Error) (parameter `offset`, without sending a
request) and ends.

### <a id="struct.ListEventCreators"></a>`struct ListEventCreators`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListEventCreators { /* private fields */ }
```

Request builder for [`GammaClient::list_event_creators`](gamma.md#GammaClient.fn.list_event_creators).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListEventCreators.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of creators per page (`limit`; the docs give a minimum of `0` and
no maximum).

##### <a id="ListEventCreators.fn.offset"></a>`offset`

```rust
pub fn offset(self, offset: u32) -> Self
```

Number of creators to skip (`offset`).

##### <a id="ListEventCreators.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `volume`, `volume24hr`, `liquidity`, `startDate`,
`endDate`, `createdAt` or `id`); snake_case names such as `start_date` are rejected with
a `422` (`order fields are not valid`), although the spec's keyset example uses them.
See `SPEC_DEVIATIONS.md`.

##### <a id="ListEventCreators.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="ListEventCreators.fn.creator_name"></a>`creator_name`

```rust
pub fn creator_name(self, creator_name: impl Into<String>) -> Self
```

Filter by creator name (`creator_name`).

##### <a id="ListEventCreators.fn.creator_handle"></a>`creator_handle`

```rust
pub fn creator_handle(self, creator_handle: impl Into<String>) -> Self
```

Filter by creator handle (`creator_handle`).

##### <a id="ListEventCreators.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<EventCreator>>
```

Fetches one page.

###### Errors

See [`Error`](marcasite.md#enum.Error).

##### <a id="ListEventCreators.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<EventCreator>
```

Streams every creator from the configured offset onwards, fetching pages lazily.

The stream ends at the first empty page, or right after yielding the first error.
A page shorter than [`limit`](gamma.md#ListEventCreators.fn.limit) does not end it, because the docs give no
maximum `limit` and the server may return fewer creators, so the last request
returns an empty page.

### <a id="struct.ListEvents"></a>`struct ListEvents`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListEvents { /* private fields */ }
```

Request builder for [`GammaClient::list_events`](gamma.md#GammaClient.fn.list_events).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListEvents.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of events per page (`limit`; the docs give a minimum of `0` and
no maximum).

##### <a id="ListEvents.fn.offset"></a>`offset`

```rust
pub fn offset(self, offset: u32) -> Self
```

Number of events to skip (`offset`). Live rejects values above 2000 (a `422`
pointing at the keyset listing, not in the spec), so larger values fail
client-side with [`Error::Validation`](marcasite.md#enum.Error); use
[`GammaClient::list_events_keyset`](gamma.md#GammaClient.fn.list_events_keyset) to page deeper.

##### <a id="ListEvents.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `volume`, `volume24hr`, `liquidity`, `startDate`,
`endDate`, `createdAt` or `id`); snake_case names such as `start_date` are rejected with
a `422` (`order fields are not valid`), although the spec's keyset example uses them.
See `SPEC_DEVIATIONS.md`.

##### <a id="ListEvents.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="ListEvents.fn.ids"></a>`ids`

```rust
pub fn ids<I>(self, ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<EventId>,
```

Filter by event ids (`id`, repeated). The spec types them as integers, so an id
 that is not one or more ASCII digits is rejected before sending with
 [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListEvents.fn.tag_id"></a>`tag_id`

```rust
pub fn tag_id(self, tag_id: impl Into<TagId>) -> Self
```

Filter by tag id (`tag_id`). The spec types it as an integer, so an id that is not
one or more ASCII digits is rejected before sending with
[`Error::Validation`](marcasite.md#enum.Error).

##### <a id="ListEvents.fn.exclude_tag_ids"></a>`exclude_tag_ids`

```rust
pub fn exclude_tag_ids<I>(self, exclude_tag_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TagId>,
```

Tag ids to exclude (`exclude_tag_id`, repeated). The spec types them as
 integers, so an id that is not one or more ASCII digits is rejected before
 sending with [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListEvents.fn.slugs"></a>`slugs`

```rust
pub fn slugs<I>(self, slugs: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by slugs (`slug`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListEvents.fn.tag_slug"></a>`tag_slug`

```rust
pub fn tag_slug(self, tag_slug: impl Into<String>) -> Self
```

Filter by tag slug (`tag_slug`).

##### <a id="ListEvents.fn.related_tags"></a>`related_tags`

```rust
pub fn related_tags(self, related_tags: bool) -> Self
```

The `related_tags` flag (documented only as a boolean).

##### <a id="ListEvents.fn.active"></a>`active`

```rust
pub fn active(self, active: bool) -> Self
```

The `active` filter (documented only as a boolean).

##### <a id="ListEvents.fn.archived"></a>`archived`

```rust
pub fn archived(self, archived: bool) -> Self
```

The `archived` filter (documented only as a boolean).

##### <a id="ListEvents.fn.featured"></a>`featured`

```rust
pub fn featured(self, featured: bool) -> Self
```

The `featured` filter (documented only as a boolean).

##### <a id="ListEvents.fn.cyom"></a>`cyom`

```rust
pub fn cyom(self, cyom: bool) -> Self
```

The `cyom` filter (documented only as a boolean).

##### <a id="ListEvents.fn.include_chat"></a>`include_chat`

```rust
pub fn include_chat(self, include_chat: bool) -> Self
```

The `include_chat` flag (documented only as a boolean on this endpoint).

##### <a id="ListEvents.fn.include_template"></a>`include_template`

```rust
pub fn include_template(self, include_template: bool) -> Self
```

The `include_template` flag (documented only as a boolean on this endpoint).

##### <a id="ListEvents.fn.recurrence"></a>`recurrence`

```rust
pub fn recurrence(self, recurrence: impl Into<String>) -> Self
```

The `recurrence` filter (a string; the spec documents no values).

##### <a id="ListEvents.fn.closed"></a>`closed`

```rust
pub fn closed(self, closed: bool) -> Self
```

The `closed` filter (documented only as a boolean).

##### <a id="ListEvents.fn.liquidity_min"></a>`liquidity_min`

```rust
pub fn liquidity_min(self, liquidity_min: impl Into<Decimal>) -> Self
```

Minimum liquidity (`liquidity_min`).

##### <a id="ListEvents.fn.liquidity_max"></a>`liquidity_max`

```rust
pub fn liquidity_max(self, liquidity_max: impl Into<Decimal>) -> Self
```

Maximum liquidity (`liquidity_max`).

##### <a id="ListEvents.fn.volume_min"></a>`volume_min`

```rust
pub fn volume_min(self, volume_min: impl Into<Decimal>) -> Self
```

Minimum volume (`volume_min`).

##### <a id="ListEvents.fn.volume_max"></a>`volume_max`

```rust
pub fn volume_max(self, volume_max: impl Into<Decimal>) -> Self
```

Maximum volume (`volume_max`).

##### <a id="ListEvents.fn.start_date_min"></a>`start_date_min`

```rust
pub fn start_date_min(self, start_date_min: DateTime<Utc>) -> Self
```

Earliest start date (`start_date_min`).

##### <a id="ListEvents.fn.start_date_max"></a>`start_date_max`

```rust
pub fn start_date_max(self, start_date_max: DateTime<Utc>) -> Self
```

Latest start date (`start_date_max`).

##### <a id="ListEvents.fn.end_date_min"></a>`end_date_min`

```rust
pub fn end_date_min(self, end_date_min: DateTime<Utc>) -> Self
```

Earliest end date (`end_date_min`).

##### <a id="ListEvents.fn.end_date_max"></a>`end_date_max`

```rust
pub fn end_date_max(self, end_date_max: DateTime<Utc>) -> Self
```

Latest end date (`end_date_max`).

##### <a id="ListEvents.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Event>>
```

Fetches one page.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if an [`ids`](gamma.md#ListEvents.fn.ids) or
  [`exclude_tag_ids`](gamma.md#ListEvents.fn.exclude_tag_ids) entry or [`tag_id`](gamma.md#ListEvents.fn.tag_id) is
  not an integer, or [`offset`](gamma.md#ListEvents.fn.offset) exceeds 2000 (live rejects larger
  offsets; page deeper with [`GammaClient::list_events_keyset`](gamma.md#GammaClient.fn.list_events_keyset)), checked before
  sending;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListEvents.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Event>
```

Streams every event from the configured offset onwards, fetching pages lazily.

The stream ends at the first empty page, or right after yielding the first error
(the errors of [`send`](gamma.md#ListEvents.fn.send)). A page shorter than [`limit`](gamma.md#ListEvents.fn.limit)
does not end it, because the docs give no maximum `limit` and the server may return
fewer events, so the last request returns an empty page.

Live rejects offsets above 2000, so a listing longer than that cannot be walked: the
stream yields every event up to the page that starts at the last accepted offset, then
one [`Error::Validation`](marcasite.md#enum.Error) (parameter `offset`, without
sending a request) and ends. Use [`GammaClient::list_events_keyset`](gamma.md#GammaClient.fn.list_events_keyset) to walk the whole
listing.

### <a id="struct.ListEventsKeyset"></a>`struct ListEventsKeyset`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListEventsKeyset { /* private fields */ }
```

Request builder for [`GammaClient::list_events_keyset`](gamma.md#GammaClient.fn.list_events_keyset).

The endpoint rejects `offset`, so there is no setter for it; page with
[`cursor`](gamma.md#ListEventsKeyset.fn.cursor) instead.

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListEventsKeyset.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of events per page (`limit`), between 1 and 100 (server default
20). Values outside that range are rejected client-side with
[`Error::Validation`](marcasite.md#enum.Error).

##### <a id="ListEventsKeyset.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `volume`, `volume24hr`, `liquidity`, `startDate`,
`endDate`, `createdAt` or `id`); snake_case names such as `start_date` are rejected with
a `422` (`order fields are not valid`), although the spec's keyset example uses them.
See `SPEC_DEVIATIONS.md`.

##### <a id="ListEventsKeyset.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort direction (`ascending`, server default `true`). Only used when
[`order`](gamma.md#ListEventsKeyset.fn.order) is set.

##### <a id="ListEventsKeyset.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Opaque cursor from a previous page's
[`next_cursor()`](gamma.md#EventsKeysetPage.fn.next_cursor), sent as `after_cursor`.

##### <a id="ListEventsKeyset.fn.ids"></a>`ids`

```rust
pub fn ids<I>(self, ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<EventId>,
```

Filter by event ids (`id`, repeated). The spec types them as integers, so an id
 that is not one or more ASCII digits is rejected before sending with
 [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListEventsKeyset.fn.slugs"></a>`slugs`

```rust
pub fn slugs<I>(self, slugs: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by slugs (`slug`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListEventsKeyset.fn.closed"></a>`closed`

```rust
pub fn closed(self, closed: bool) -> Self
```

The `closed` filter (documented only as a boolean).

##### <a id="ListEventsKeyset.fn.live"></a>`live`

```rust
pub fn live(self, live: bool) -> Self
```

The `live` filter (documented only as a boolean).

##### <a id="ListEventsKeyset.fn.featured"></a>`featured`

```rust
pub fn featured(self, featured: bool) -> Self
```

The `featured` filter (documented only as a boolean).

##### <a id="ListEventsKeyset.fn.cyom"></a>`cyom`

```rust
pub fn cyom(self, cyom: bool) -> Self
```

The `cyom` filter (documented only as a boolean).

##### <a id="ListEventsKeyset.fn.title_search"></a>`title_search`

```rust
pub fn title_search(self, title_search: impl Into<String>) -> Self
```

The `title_search` filter (a string; the docs do not describe the matching).

##### <a id="ListEventsKeyset.fn.liquidity_min"></a>`liquidity_min`

```rust
pub fn liquidity_min(self, liquidity_min: impl Into<Decimal>) -> Self
```

Minimum liquidity (`liquidity_min`).

##### <a id="ListEventsKeyset.fn.liquidity_max"></a>`liquidity_max`

```rust
pub fn liquidity_max(self, liquidity_max: impl Into<Decimal>) -> Self
```

Maximum liquidity (`liquidity_max`).

##### <a id="ListEventsKeyset.fn.volume_min"></a>`volume_min`

```rust
pub fn volume_min(self, volume_min: impl Into<Decimal>) -> Self
```

Minimum volume (`volume_min`).

##### <a id="ListEventsKeyset.fn.volume_max"></a>`volume_max`

```rust
pub fn volume_max(self, volume_max: impl Into<Decimal>) -> Self
```

Maximum volume (`volume_max`).

##### <a id="ListEventsKeyset.fn.start_date_min"></a>`start_date_min`

```rust
pub fn start_date_min(self, start_date_min: DateTime<Utc>) -> Self
```

Earliest start date (`start_date_min`).

##### <a id="ListEventsKeyset.fn.start_date_max"></a>`start_date_max`

```rust
pub fn start_date_max(self, start_date_max: DateTime<Utc>) -> Self
```

Latest start date (`start_date_max`).

##### <a id="ListEventsKeyset.fn.end_date_min"></a>`end_date_min`

```rust
pub fn end_date_min(self, end_date_min: DateTime<Utc>) -> Self
```

Earliest end date (`end_date_min`).

##### <a id="ListEventsKeyset.fn.end_date_max"></a>`end_date_max`

```rust
pub fn end_date_max(self, end_date_max: DateTime<Utc>) -> Self
```

Latest end date (`end_date_max`).

##### <a id="ListEventsKeyset.fn.start_time_min"></a>`start_time_min`

```rust
pub fn start_time_min(self, start_time_min: DateTime<Utc>) -> Self
```

Earliest start time (`start_time_min`).

##### <a id="ListEventsKeyset.fn.start_time_max"></a>`start_time_max`

```rust
pub fn start_time_max(self, start_time_max: DateTime<Utc>) -> Self
```

Latest start time (`start_time_max`).

##### <a id="ListEventsKeyset.fn.tag_ids"></a>`tag_ids`

```rust
pub fn tag_ids<I>(self, tag_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TagId>,
```

Filter by tag ids (`tag_id`, repeated). The spec types them as integers, so an id
 that is not one or more ASCII digits is rejected before sending with
 [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListEventsKeyset.fn.tag_slug"></a>`tag_slug`

```rust
pub fn tag_slug(self, tag_slug: impl Into<String>) -> Self
```

Filter by tag slug (`tag_slug`).

##### <a id="ListEventsKeyset.fn.exclude_tag_ids"></a>`exclude_tag_ids`

```rust
pub fn exclude_tag_ids<I>(self, exclude_tag_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TagId>,
```

Tag ids to exclude (`exclude_tag_id`, repeated). Documented as "Cannot overlap
 with tag_id": an overlap with [`tag_ids`](gamma.md#ListEventsKeyset.fn.tag_ids) (compared as integers),
 or an id that is not one or more ASCII digits, is rejected before sending with
 [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListEventsKeyset.fn.related_tags"></a>`related_tags`

```rust
pub fn related_tags(self, related_tags: bool) -> Self
```

The `related_tags` flag (documented only as a boolean).

##### <a id="ListEventsKeyset.fn.tag_match"></a>`tag_match`

```rust
pub fn tag_match(self, tag_match: impl Into<String>) -> Self
```

The `tag_match` parameter (a string; the spec documents no values).

##### <a id="ListEventsKeyset.fn.series_ids"></a>`series_ids`

```rust
pub fn series_ids<I>(self, series_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<SeriesId>,
```

Filter by series ids (`series_id`, repeated). The spec types them as integers, so
 an id that is not one or more ASCII digits is rejected before sending with
 [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListEventsKeyset.fn.game_ids"></a>`game_ids`

```rust
pub fn game_ids<I>(self, game_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<i64>,
```

Filter by game ids (`game_id`, repeated; integers on this endpoint).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListEventsKeyset.fn.event_date"></a>`event_date`

```rust
pub fn event_date(self, event_date: DateTime<Utc>) -> Self
```

The `event_date` filter.

##### <a id="ListEventsKeyset.fn.event_week"></a>`event_week`

```rust
pub fn event_week(self, event_week: i64) -> Self
```

The `event_week` filter (an integer).

##### <a id="ListEventsKeyset.fn.featured_order"></a>`featured_order`

```rust
pub fn featured_order(self, featured_order: bool) -> Self
```

The `featured_order` flag (documented only as a boolean).

##### <a id="ListEventsKeyset.fn.recurrence"></a>`recurrence`

```rust
pub fn recurrence(self, recurrence: impl Into<String>) -> Self
```

The `recurrence` filter (a string; the spec documents no values, and the server
answers `422` for an invalid one).

##### <a id="ListEventsKeyset.fn.created_by"></a>`created_by`

```rust
pub fn created_by<I>(self, created_by: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

The `created_by` filter (repeated strings).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListEventsKeyset.fn.parent_event_id"></a>`parent_event_id`

```rust
pub fn parent_event_id(self, parent_event_id: impl Into<EventId>) -> Self
```

The `parent_event_id` filter. The spec types it as an integer, so an id that is
not one or more ASCII digits is rejected before sending with
[`Error::Validation`](marcasite.md#enum.Error).

##### <a id="ListEventsKeyset.fn.include_children"></a>`include_children`

```rust
pub fn include_children(self, include_children: bool) -> Self
```

The `include_children` flag (documented only as a boolean).

##### <a id="ListEventsKeyset.fn.partner_slug"></a>`partner_slug`

```rust
pub fn partner_slug(self, partner_slug: impl Into<String>) -> Self
```

The `partner_slug` parameter, documented as: "When set, external_partners are
attached to matching events". The documented `Event` schema has no
`external_partners` property, so that data is not modelled and is dropped when
decoding.

##### <a id="ListEventsKeyset.fn.include_chat"></a>`include_chat`

```rust
pub fn include_chat(self, include_chat: bool) -> Self
```

When `true`, includes the `Chats` and `Series.Chats` relations (`include_chat`),
modelled as [`Event::chats`](gamma.md#struct.Event) and [`Series::chats`](gamma.md#struct.Series).

##### <a id="ListEventsKeyset.fn.include_template"></a>`include_template`

```rust
pub fn include_template(self, include_template: bool) -> Self
```

When `true`, includes the `Templates` relation (`include_template`), modelled as
[`Event::templates`](gamma.md#struct.Event).

##### <a id="ListEventsKeyset.fn.include_best_lines"></a>`include_best_lines`

```rust
pub fn include_best_lines(self, include_best_lines: bool) -> Self
```

When `true`, includes the `BestLines` relation (`include_best_lines`). The
documented `Event` schema has no property for it, so that data is not modelled
and is dropped when decoding.

##### <a id="ListEventsKeyset.fn.locale"></a>`locale`

```rust
pub fn locale(self, locale: impl Into<String>) -> Self
```

The `locale` parameter (a string; the spec documents no values).

##### <a id="ListEventsKeyset.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<EventsKeysetPage>
```

Fetches one page, starting at [`cursor`](gamma.md#ListEventsKeyset.fn.cursor) if set.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error), checked before sending, if
  [`limit`](gamma.md#ListEventsKeyset.fn.limit) is out of range, an integer-typed id is not one or more
  ASCII digits, or [`exclude_tag_ids`](gamma.md#ListEventsKeyset.fn.exclude_tag_ids) overlaps
  [`tag_ids`](gamma.md#ListEventsKeyset.fn.tag_ids);
- [`Error::Api`](marcasite.md#enum.Error) with status `422` (whose
  [`error_type`](marcasite.md#ApiError.fn.error_type) is `"validation error"`) for an
  invalid cursor, order field, recurrence or filter;
- [`Error::Api`](marcasite.md#enum.Error) with status `500` (`"internal error"`) for a
  server-side failure;
- [`Error::Api`](marcasite.md#enum.Error) with status `503` and
  [`error_type`](marcasite.md#ApiError.fn.error_type) `"service unavailable"`, which the
  spec documents as "keyset pagination is not configured". That is a server
  configuration state rather than a transient failure, so retrying is unlikely to
  help; [`GammaClient::list_events`](gamma.md#GammaClient.fn.list_events) and [`GammaClient::list_events_paginated`](gamma.md#GammaClient.fn.list_events_paginated) are
  the offset-paginated alternatives;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListEventsKeyset.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Event>
```

Streams every event from [`cursor`](gamma.md#ListEventsKeyset.fn.cursor) (or the beginning) onwards,
fetching pages lazily until a page has no (or an empty) `next_cursor`.

The stream ends right after yielding the first error (the errors of
[`send`](gamma.md#ListEventsKeyset.fn.send)).

### <a id="struct.ListEventsPaginated"></a>`struct ListEventsPaginated`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListEventsPaginated { /* private fields */ }
```

Request builder for [`GammaClient::list_events_paginated`](gamma.md#GammaClient.fn.list_events_paginated).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListEventsPaginated.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of events per page (`limit`; the docs give a minimum of `0` and
no maximum).

##### <a id="ListEventsPaginated.fn.offset"></a>`offset`

```rust
pub fn offset(self, offset: u32) -> Self
```

Number of events to skip (`offset`). Live rejects values above 2000 (a `422`
pointing at the keyset listing, not in the spec), so larger values fail
client-side with [`Error::Validation`](marcasite.md#enum.Error); use
[`GammaClient::list_events_keyset`](gamma.md#GammaClient.fn.list_events_keyset) to page deeper.

##### <a id="ListEventsPaginated.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `volume`, `volume24hr`, `liquidity`, `startDate`,
`endDate`, `createdAt` or `id`); snake_case names such as `start_date` are rejected with
a `422` (`order fields are not valid`), although the spec's keyset example uses them.
See `SPEC_DEVIATIONS.md`.

##### <a id="ListEventsPaginated.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="ListEventsPaginated.fn.include_chat"></a>`include_chat`

```rust
pub fn include_chat(self, include_chat: bool) -> Self
```

The `include_chat` flag (documented only as a boolean on this endpoint).

##### <a id="ListEventsPaginated.fn.include_template"></a>`include_template`

```rust
pub fn include_template(self, include_template: bool) -> Self
```

The `include_template` flag (documented only as a boolean on this endpoint).

##### <a id="ListEventsPaginated.fn.recurrence"></a>`recurrence`

```rust
pub fn recurrence(self, recurrence: impl Into<String>) -> Self
```

The `recurrence` filter (a string; the spec documents no values).

##### <a id="ListEventsPaginated.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<EventsPage>
```

Fetches one page, including its pagination metadata.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if [`offset`](gamma.md#ListEventsPaginated.fn.offset) exceeds
  2000 (live rejects larger offsets; page deeper with
  [`GammaClient::list_events_keyset`](gamma.md#GammaClient.fn.list_events_keyset)), checked before sending;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListEventsPaginated.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Event>
```

Streams every event from the configured offset onwards, fetching pages lazily.

The stream ends after a page whose `pagination.hasMore` is `false`, at the first
empty page, or right after yielding the first error. A page shorter than
[`limit`](gamma.md#ListEventsPaginated.fn.limit) does not end it while `hasMore` is not `false`, because the
server may return fewer events than requested.

Live rejects offsets above 2000: past that the stream yields one
[`Error::Validation`](marcasite.md#enum.Error) (parameter `offset`, without sending a
request) and ends. Use [`GammaClient::list_events_keyset`](gamma.md#GammaClient.fn.list_events_keyset) to walk longer listings.

### <a id="struct.ListMarkets"></a>`struct ListMarkets`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListMarkets { /* private fields */ }
```

Request builder for [`GammaClient::list_markets`](gamma.md#GammaClient.fn.list_markets).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListMarkets.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of markets per page (`limit`; the docs give a minimum of `0` and
no maximum).

##### <a id="ListMarkets.fn.offset"></a>`offset`

```rust
pub fn offset(self, offset: u32) -> Self
```

Number of markets to skip (`offset`). Live rejects values above 2000 (a `422`
pointing at the keyset listing, not in the spec), so larger values fail
client-side with [`Error::Validation`](marcasite.md#enum.Error); use
[`GammaClient::list_markets_keyset`](gamma.md#GammaClient.fn.list_markets_keyset) to page deeper.

##### <a id="ListMarkets.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `volumeNum`, `liquidityNum`, `volume24hr`,
`startDate`, `endDate`, `createdAt` or `id`); snake_case names such as `start_date` are
rejected with a `422` (`order fields are not valid`), although the spec's keyset example
uses them. See `SPEC_DEVIATIONS.md`.

##### <a id="ListMarkets.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="ListMarkets.fn.ids"></a>`ids`

```rust
pub fn ids<I>(self, ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<MarketId>,
```

Filter by market ids (`id`, repeated). The spec types them as integers, so an id
 that is not one or more ASCII digits is rejected before sending with
 [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarkets.fn.slugs"></a>`slugs`

```rust
pub fn slugs<I>(self, slugs: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by slugs (`slug`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarkets.fn.clob_token_ids"></a>`clob_token_ids`

```rust
pub fn clob_token_ids<I>(self, clob_token_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

Filter by CLOB token ids (`clob_token_ids`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarkets.fn.condition_ids"></a>`condition_ids`

```rust
pub fn condition_ids<I>(self, condition_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Filter by condition ids (`condition_ids`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarkets.fn.liquidity_num_min"></a>`liquidity_num_min`

```rust
pub fn liquidity_num_min(self, liquidity_num_min: impl Into<Decimal>) -> Self
```

Minimum liquidity (`liquidity_num_min`).

##### <a id="ListMarkets.fn.liquidity_num_max"></a>`liquidity_num_max`

```rust
pub fn liquidity_num_max(self, liquidity_num_max: impl Into<Decimal>) -> Self
```

Maximum liquidity (`liquidity_num_max`).

##### <a id="ListMarkets.fn.volume_num_min"></a>`volume_num_min`

```rust
pub fn volume_num_min(self, volume_num_min: impl Into<Decimal>) -> Self
```

Minimum volume (`volume_num_min`).

##### <a id="ListMarkets.fn.volume_num_max"></a>`volume_num_max`

```rust
pub fn volume_num_max(self, volume_num_max: impl Into<Decimal>) -> Self
```

Maximum volume (`volume_num_max`).

##### <a id="ListMarkets.fn.start_date_min"></a>`start_date_min`

```rust
pub fn start_date_min(self, start_date_min: DateTime<Utc>) -> Self
```

Earliest start date (`start_date_min`).

##### <a id="ListMarkets.fn.start_date_max"></a>`start_date_max`

```rust
pub fn start_date_max(self, start_date_max: DateTime<Utc>) -> Self
```

Latest start date (`start_date_max`).

##### <a id="ListMarkets.fn.end_date_min"></a>`end_date_min`

```rust
pub fn end_date_min(self, end_date_min: DateTime<Utc>) -> Self
```

Earliest end date (`end_date_min`).

##### <a id="ListMarkets.fn.end_date_max"></a>`end_date_max`

```rust
pub fn end_date_max(self, end_date_max: DateTime<Utc>) -> Self
```

Latest end date (`end_date_max`).

##### <a id="ListMarkets.fn.tag_id"></a>`tag_id`

```rust
pub fn tag_id(self, tag_id: impl Into<TagId>) -> Self
```

Filter by tag id (`tag_id`). The spec types it as an integer, so an id that is not
one or more ASCII digits is rejected before sending with
[`Error::Validation`](marcasite.md#enum.Error).

##### <a id="ListMarkets.fn.related_tags"></a>`related_tags`

```rust
pub fn related_tags(self, related_tags: bool) -> Self
```

The `related_tags` flag (documented only as a boolean).

##### <a id="ListMarkets.fn.cyom"></a>`cyom`

```rust
pub fn cyom(self, cyom: bool) -> Self
```

The `cyom` filter (documented only as a boolean).

##### <a id="ListMarkets.fn.uma_resolution_status"></a>`uma_resolution_status`

```rust
pub fn uma_resolution_status(self, uma_resolution_status: impl Into<String>) -> Self
```

Filter by UMA resolution status (`uma_resolution_status`; the spec documents no
values).

##### <a id="ListMarkets.fn.game_id"></a>`game_id`

```rust
pub fn game_id(self, game_id: impl Into<String>) -> Self
```

Filter by game id (`game_id`, a string on this endpoint).

##### <a id="ListMarkets.fn.sports_market_types"></a>`sports_market_types`

```rust
pub fn sports_market_types<I>(self, sports_market_types: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by sports market types (`sports_market_types`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarkets.fn.rewards_min_size"></a>`rewards_min_size`

```rust
pub fn rewards_min_size(self, rewards_min_size: impl Into<Decimal>) -> Self
```

Minimum size for liquidity rewards (`rewards_min_size`).

##### <a id="ListMarkets.fn.question_ids"></a>`question_ids`

```rust
pub fn question_ids<I>(self, question_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<QuestionId>,
```

Filter by question ids (`question_ids`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarkets.fn.include_tag"></a>`include_tag`

```rust
pub fn include_tag(self, include_tag: bool) -> Self
```

The `include_tag` flag (documented only as a boolean on this endpoint).

##### <a id="ListMarkets.fn.closed"></a>`closed`

```rust
pub fn closed(self, closed: bool) -> Self
```

The `closed` filter (documented only as a boolean, defaulting to `false`).

##### <a id="ListMarkets.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Market>>
```

Fetches one page.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if an
  [`ids`](gamma.md#ListMarkets.fn.ids) entry or [`tag_id`](gamma.md#ListMarkets.fn.tag_id) is not an integer, or
  [`offset`](gamma.md#ListMarkets.fn.offset) exceeds 2000 (live rejects larger offsets; page deeper with
  [`GammaClient::list_markets_keyset`](gamma.md#GammaClient.fn.list_markets_keyset)), checked before sending;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListMarkets.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Market>
```

Streams every market from the configured offset onwards, fetching pages lazily.

The stream ends at the first empty page, or right after yielding the first error
(the errors of [`send`](gamma.md#ListMarkets.fn.send)). A page shorter than [`limit`](gamma.md#ListMarkets.fn.limit)
does not end it, because the docs give no maximum `limit` and the server may return
fewer markets, so the last request returns an empty page.

Live rejects offsets above 2000, so a listing longer than that cannot be walked: the
stream yields every market up to the page that starts at the last accepted offset,
then one [`Error::Validation`](marcasite.md#enum.Error) (parameter `offset`,
without sending a request) and ends. Use [`GammaClient::list_markets_keyset`](gamma.md#GammaClient.fn.list_markets_keyset) to walk
the whole listing.

### <a id="struct.ListMarketsKeyset"></a>`struct ListMarketsKeyset`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListMarketsKeyset { /* private fields */ }
```

Request builder for [`GammaClient::list_markets_keyset`](gamma.md#GammaClient.fn.list_markets_keyset).

The endpoint rejects `offset`, so there is no setter for it; page with
[`cursor`](gamma.md#ListMarketsKeyset.fn.cursor) instead.

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListMarketsKeyset.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of markets per page (`limit`), between 1 and 100 (server default
20). Values outside that range are rejected client-side with
[`Error::Validation`](marcasite.md#enum.Error).

##### <a id="ListMarketsKeyset.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `volumeNum`, `liquidityNum`, `volume24hr`,
`startDate`, `endDate`, `createdAt` or `id`); snake_case names such as `start_date` are
rejected with a `422` (`order fields are not valid`), although the spec's keyset example
uses them. See `SPEC_DEVIATIONS.md`.

##### <a id="ListMarketsKeyset.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort direction (`ascending`, server default `true`). Only used when
[`order`](gamma.md#ListMarketsKeyset.fn.order) is set.

##### <a id="ListMarketsKeyset.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Opaque cursor from a previous page's
[`next_cursor()`](gamma.md#MarketsKeysetPage.fn.next_cursor), sent as `after_cursor`.

##### <a id="ListMarketsKeyset.fn.ids"></a>`ids`

```rust
pub fn ids<I>(self, ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<MarketId>,
```

Filter by market ids (`id`, repeated). The spec types them as integers, so an id
 that is not one or more ASCII digits is rejected before sending with
 [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarketsKeyset.fn.slugs"></a>`slugs`

```rust
pub fn slugs<I>(self, slugs: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by slugs (`slug`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarketsKeyset.fn.closed"></a>`closed`

```rust
pub fn closed(self, closed: bool) -> Self
```

The `closed` filter (documented only as a boolean, defaulting to `false`).

##### <a id="ListMarketsKeyset.fn.decimalized"></a>`decimalized`

```rust
pub fn decimalized(self, decimalized: bool) -> Self
```

The `decimalized` flag (documented only as a boolean).

##### <a id="ListMarketsKeyset.fn.clob_token_ids"></a>`clob_token_ids`

```rust
pub fn clob_token_ids<I>(self, clob_token_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

Filter by CLOB token ids (`clob_token_ids`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarketsKeyset.fn.condition_ids"></a>`condition_ids`

```rust
pub fn condition_ids<I>(self, condition_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Filter by condition ids (`condition_ids`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarketsKeyset.fn.question_ids"></a>`question_ids`

```rust
pub fn question_ids<I>(self, question_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<QuestionId>,
```

Filter by question ids (`question_ids`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarketsKeyset.fn.liquidity_num_min"></a>`liquidity_num_min`

```rust
pub fn liquidity_num_min(self, liquidity_num_min: impl Into<Decimal>) -> Self
```

Minimum liquidity (`liquidity_num_min`).

##### <a id="ListMarketsKeyset.fn.liquidity_num_max"></a>`liquidity_num_max`

```rust
pub fn liquidity_num_max(self, liquidity_num_max: impl Into<Decimal>) -> Self
```

Maximum liquidity (`liquidity_num_max`).

##### <a id="ListMarketsKeyset.fn.volume_num_min"></a>`volume_num_min`

```rust
pub fn volume_num_min(self, volume_num_min: impl Into<Decimal>) -> Self
```

Minimum volume (`volume_num_min`).

##### <a id="ListMarketsKeyset.fn.volume_num_max"></a>`volume_num_max`

```rust
pub fn volume_num_max(self, volume_num_max: impl Into<Decimal>) -> Self
```

Maximum volume (`volume_num_max`).

##### <a id="ListMarketsKeyset.fn.start_date_min"></a>`start_date_min`

```rust
pub fn start_date_min(self, start_date_min: DateTime<Utc>) -> Self
```

Earliest start date (`start_date_min`).

##### <a id="ListMarketsKeyset.fn.start_date_max"></a>`start_date_max`

```rust
pub fn start_date_max(self, start_date_max: DateTime<Utc>) -> Self
```

Latest start date (`start_date_max`).

##### <a id="ListMarketsKeyset.fn.end_date_min"></a>`end_date_min`

```rust
pub fn end_date_min(self, end_date_min: DateTime<Utc>) -> Self
```

Earliest end date (`end_date_min`).

##### <a id="ListMarketsKeyset.fn.end_date_max"></a>`end_date_max`

```rust
pub fn end_date_max(self, end_date_max: DateTime<Utc>) -> Self
```

Latest end date (`end_date_max`).

##### <a id="ListMarketsKeyset.fn.tag_ids"></a>`tag_ids`

```rust
pub fn tag_ids<I>(self, tag_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TagId>,
```

Filter by tag ids (`tag_id`, repeated). The spec types them as integers, so an id
 that is not one or more ASCII digits is rejected before sending with
 [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarketsKeyset.fn.related_tags"></a>`related_tags`

```rust
pub fn related_tags(self, related_tags: bool) -> Self
```

The `related_tags` flag (documented only as a boolean).

##### <a id="ListMarketsKeyset.fn.tag_match"></a>`tag_match`

```rust
pub fn tag_match(self, tag_match: impl Into<String>) -> Self
```

The `tag_match` parameter (a string; the spec documents no values).

##### <a id="ListMarketsKeyset.fn.cyom"></a>`cyom`

```rust
pub fn cyom(self, cyom: bool) -> Self
```

The `cyom` filter (documented only as a boolean).

##### <a id="ListMarketsKeyset.fn.rfq_enabled"></a>`rfq_enabled`

```rust
pub fn rfq_enabled(self, rfq_enabled: bool) -> Self
```

The `rfq_enabled` filter (documented only as a boolean).

##### <a id="ListMarketsKeyset.fn.uma_resolution_status"></a>`uma_resolution_status`

```rust
pub fn uma_resolution_status(self, uma_resolution_status: impl Into<String>) -> Self
```

Filter by UMA resolution status (`uma_resolution_status`; the spec documents no
values).

##### <a id="ListMarketsKeyset.fn.game_id"></a>`game_id`

```rust
pub fn game_id(self, game_id: impl Into<String>) -> Self
```

Filter by game id (`game_id`, a string on this endpoint).

##### <a id="ListMarketsKeyset.fn.sports_market_types"></a>`sports_market_types`

```rust
pub fn sports_market_types<I>(self, sports_market_types: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by sports market types (`sports_market_types`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListMarketsKeyset.fn.include_tag"></a>`include_tag`

```rust
pub fn include_tag(self, include_tag: bool) -> Self
```

When `true`, includes the `Tags` relation on each market (`include_tag`).

##### <a id="ListMarketsKeyset.fn.locale"></a>`locale`

```rust
pub fn locale(self, locale: impl Into<String>) -> Self
```

The `locale` parameter (a string; the spec documents no values).

##### <a id="ListMarketsKeyset.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<MarketsKeysetPage>
```

Fetches one page, starting at [`cursor`](gamma.md#ListMarketsKeyset.fn.cursor) if set.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error), checked before sending, if
  [`limit`](gamma.md#ListMarketsKeyset.fn.limit) is out of range or an [`ids`](gamma.md#ListMarketsKeyset.fn.ids) or
  [`tag_ids`](gamma.md#ListMarketsKeyset.fn.tag_ids) entry is not an integer;
- [`Error::Api`](marcasite.md#enum.Error) with status `422` (whose
  [`error_type`](marcasite.md#ApiError.fn.error_type) is `"validation error"`) for an
  invalid cursor, order field or filter;
- [`Error::Api`](marcasite.md#enum.Error) with status `500` (`"internal error"`) for a
  server-side failure;
- [`Error::Api`](marcasite.md#enum.Error) with status `503` and
  [`error_type`](marcasite.md#ApiError.fn.error_type) `"service unavailable"`, which the
  spec documents as "keyset pagination is not configured". That is a server
  configuration state rather than a transient failure, so retrying is unlikely to
  help; [`GammaClient::list_markets`](gamma.md#GammaClient.fn.list_markets) is the offset-paginated alternative;
- otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListMarketsKeyset.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Market>
```

Streams every market from [`cursor`](gamma.md#ListMarketsKeyset.fn.cursor) (or the beginning) onwards,
fetching pages lazily until a page has no (or an empty) `next_cursor`.

The stream ends right after yielding the first error (the errors of
[`send`](gamma.md#ListMarketsKeyset.fn.send)).

### <a id="struct.ListSeries"></a>`struct ListSeries`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListSeries { /* private fields */ }
```

Request builder for [`GammaClient::list_series`](gamma.md#GammaClient.fn.list_series).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListSeries.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of series per page (`limit`; the docs give a minimum of `0` and
no maximum).

##### <a id="ListSeries.fn.offset"></a>`offset`

```rust
pub fn offset(self, offset: u32) -> Self
```

Number of series to skip (`offset`).

##### <a id="ListSeries.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase
JSON field names of the response type (e.g. `volume`, `startDate`, `createdAt` or
`id`); snake_case names such as `start_date` are rejected with a `422` (`order fields
are not valid`), although the spec's keyset example uses them. See
`SPEC_DEVIATIONS.md`.

Live answers `volume24hr` or `liquidity` combined with `ascending(false)` and a
large enough [`limit`](gamma.md#ListSeries.fn.limit) with a `500` (observed from `limit` 20 for
`volume24hr` and 50 for `liquidity`; the threshold shifts with the data). That is a
server bug; use a smaller `limit` or another order field (`volume` works). See
`SPEC_DEVIATIONS.md`.

##### <a id="ListSeries.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="ListSeries.fn.slugs"></a>`slugs`

```rust
pub fn slugs<I>(self, slugs: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by slugs (`slug`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListSeries.fn.categories_ids"></a>`categories_ids`

```rust
pub fn categories_ids<I>(self, categories_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<i64>,
```

Filter by category ids (`categories_ids`, repeated integers).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListSeries.fn.categories_labels"></a>`categories_labels`

```rust
pub fn categories_labels<I>(self, categories_labels: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by category labels (`categories_labels`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListSeries.fn.closed"></a>`closed`

```rust
pub fn closed(self, closed: bool) -> Self
```

The `closed` filter (documented only as a boolean).

##### <a id="ListSeries.fn.include_chat"></a>`include_chat`

```rust
pub fn include_chat(self, include_chat: bool) -> Self
```

The `include_chat` flag (documented only as a boolean on this endpoint).

##### <a id="ListSeries.fn.recurrence"></a>`recurrence`

```rust
pub fn recurrence(self, recurrence: impl Into<String>) -> Self
```

The `recurrence` filter (a string; the spec documents no values).

##### <a id="ListSeries.fn.exclude_events"></a>`exclude_events`

```rust
pub fn exclude_events(self, exclude_events: bool) -> Self
```

The `exclude_events` flag (documented only as a boolean).

##### <a id="ListSeries.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Series>>
```

Fetches one page.

###### Errors

See [`Error`](marcasite.md#enum.Error).

##### <a id="ListSeries.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Series>
```

Streams every series from the configured offset onwards, fetching pages lazily.

The stream ends at the first empty page, or right after yielding the first error.
A page shorter than [`limit`](gamma.md#ListSeries.fn.limit) does not end it, because the docs give no
maximum `limit` and the server may return fewer series, so the last request returns
an empty page.

### <a id="struct.ListSportEventResults"></a>`struct ListSportEventResults`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListSportEventResults { /* private fields */ }
```

Request builder for [`GammaClient::list_sport_event_results`](gamma.md#GammaClient.fn.list_sport_event_results).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListSportEventResults.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of events per page (`limit`; the docs give a minimum of `0` and
no maximum).

##### <a id="ListSportEventResults.fn.offset"></a>`offset`

```rust
pub fn offset(self, offset: u32) -> Self
```

Number of events to skip (`offset`).

##### <a id="ListSportEventResults.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `volume`, `volume24hr`, `liquidity`, `startDate`,
`endDate`, `createdAt` or `id`); snake_case names such as `start_date` are rejected with
a `422` (`order fields are not valid`), although the spec's keyset example uses them.
See `SPEC_DEVIATIONS.md`.

##### <a id="ListSportEventResults.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="ListSportEventResults.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Event>>
```

Fetches one page.

###### Errors

See [`Error`](marcasite.md#enum.Error).

##### <a id="ListSportEventResults.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Event>
```

Streams every event from the configured offset onwards, fetching pages lazily.

The stream ends at the first empty page, or right after yielding the first error.
A page shorter than [`limit`](gamma.md#ListSportEventResults.fn.limit) does not end it, because the docs give no
maximum `limit` and the server may return fewer events, so the last request returns
an empty page.

### <a id="struct.ListTags"></a>`struct ListTags`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListTags { /* private fields */ }
```

Request builder for [`GammaClient::list_tags`](gamma.md#GammaClient.fn.list_tags).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListTags.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of tags per page (`limit`; the docs give a minimum of `0` and no
maximum).

##### <a id="ListTags.fn.offset"></a>`offset`

```rust
pub fn offset(self, offset: u32) -> Self
```

Number of tags to skip (`offset`).

##### <a id="ListTags.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `label`, `createdAt` or `id`); snake_case names
such as `start_date` are rejected with a `422` (`order fields are not valid`), although
the spec's keyset example uses them. See `SPEC_DEVIATIONS.md`.

##### <a id="ListTags.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="ListTags.fn.include_template"></a>`include_template`

```rust
pub fn include_template(self, include_template: bool) -> Self
```

The `include_template` flag (documented only as a boolean).

##### <a id="ListTags.fn.is_carousel"></a>`is_carousel`

```rust
pub fn is_carousel(self, is_carousel: bool) -> Self
```

The `is_carousel` filter (documented only as a boolean).

##### <a id="ListTags.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Tag>>
```

Fetches one page.

###### Errors

See [`Error`](marcasite.md#enum.Error).

##### <a id="ListTags.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Tag>
```

Streams every tag from the configured offset onwards, fetching pages lazily.

The stream ends at the first empty page, or right after yielding the first error.
A page shorter than [`limit`](gamma.md#ListTags.fn.limit) does not end it, because the docs give no
maximum `limit` and the server may return fewer tags, so the last request returns an
empty page.

### <a id="struct.ListTeams"></a>`struct ListTeams`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListTeams { /* private fields */ }
```

Request builder for [`GammaClient::list_teams`](gamma.md#GammaClient.fn.list_teams).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListTeams.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of teams per page (`limit`; the docs give a minimum of `0` and no
maximum).

##### <a id="ListTeams.fn.offset"></a>`offset`

```rust
pub fn offset(self, offset: u32) -> Self
```

Number of teams to skip (`offset`).

##### <a id="ListTeams.fn.order"></a>`order`

```rust
pub fn order(self, order: impl Into<String>) -> Self
```

Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
field names of the response type (e.g. `name`, `createdAt` or `id`); snake_case names
such as `start_date` are rejected with a `422` (`order fields are not valid`), although
the spec's keyset example uses them. See `SPEC_DEVIATIONS.md`.

##### <a id="ListTeams.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="ListTeams.fn.leagues"></a>`leagues`

```rust
pub fn leagues<I>(self, leagues: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by leagues (`league`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListTeams.fn.names"></a>`names`

```rust
pub fn names<I>(self, names: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by team names (`name`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListTeams.fn.abbreviations"></a>`abbreviations`

```rust
pub fn abbreviations<I>(self, abbreviations: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

Filter by abbreviations (`abbreviation`, repeated).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="ListTeams.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<Team>>
```

Fetches one page.

###### Errors

See [`Error`](marcasite.md#enum.Error).

##### <a id="ListTeams.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Team>
```

Streams every team from the configured offset onwards, fetching pages lazily.

The stream ends at the first empty page, or right after yielding the first error.
A page shorter than [`limit`](gamma.md#ListTeams.fn.limit) does not end it, because the docs give no
maximum `limit` and the server may return fewer teams, so the last request returns
an empty page.

### <a id="struct.Market"></a>`struct Market`

```rust
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
    pub end_date: Option<DateTime<Utc>>,
    /// Category.
    pub category: Option<String>,
    /// Liquidity. The spec types this as a string (`liquidity`); it is parsed as a decimal, and
    /// an empty string becomes `None`.
    pub liquidity: Option<Decimal>,
    /// Sponsor name.
    pub sponsor_name: Option<String>,
    /// Sponsor image URL.
    pub sponsor_image: Option<String>,
    /// Start date.
    pub start_date: Option<DateTime<Utc>>,
    /// X-axis value.
    pub x_axis_value: Option<String>,
    /// Y-axis value.
    pub y_axis_value: Option<String>,
    /// Denomination token.
    pub denomination_token: Option<String>,
    /// Fee. The spec types this as a string (`fee`); it is parsed as a decimal, and
    /// an empty string becomes `None`.
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
    /// Outcome labels, e.g. `["Yes", "No"]`.
    ///
    /// The spec types this as a plain string and documents no encoding. Live sends a
    /// JSON-encoded string (`"[\"Yes\", \"No\"]"`) on every route except
    /// `GET /public-search?optimized=true`, which sends a real JSON array. Both decode to
    /// the list; it serializes back to the string form. See `SPEC_DEVIATIONS.md`.
    pub outcomes: Option<Vec<String>>,
    /// Outcome prices, index-aligned with [`outcomes`](gamma.md#struct.Market).
    ///
    /// The spec types this as a plain string and documents no encoding. Live sends a
    /// JSON-encoded string of decimal strings (`"[\"0.1\", \"0.9\"]"`), or a real JSON
    /// array on `GET /public-search?optimized=true` (see [`outcomes`](gamma.md#struct.Market)).
    /// Both decode to the list; it serializes back to the string form.
    pub outcome_prices: Option<Vec<Decimal>>,
    /// Volume. The spec types this as a string (`volume`); it is parsed as a decimal, and
    /// an empty string becomes `None`.
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
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
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
    pub question_id: Option<QuestionId>,
    /// UMA end date. The spec types this as a plain string with no format.
    pub uma_end_date: Option<String>,
    /// Whether the order book is enabled.
    pub enable_order_book: Option<bool>,
    /// Minimum price tick size.
    pub order_price_min_tick_size: Option<Decimal>,
    /// Minimum order size.
    pub order_min_size: Option<Decimal>,
    /// UMA resolution status.
    pub uma_resolution_status: Option<String>,
    /// Curation order.
    pub curation_order: Option<i64>,
    /// Volume as a number.
    pub volume_num: Option<Decimal>,
    /// Liquidity as a number.
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
    pub volume_24hr: Option<Decimal>,
    /// 1-week volume.
    pub volume_1wk: Option<Decimal>,
    /// 1-month volume.
    pub volume_1mo: Option<Decimal>,
    /// 1-year volume.
    pub volume_1yr: Option<Decimal>,
    /// Game start time. The spec types this as a plain string with no format.
    pub game_start_time: Option<String>,
    /// Seconds delay.
    pub seconds_delay: Option<i64>,
    /// CLOB token ids, index-aligned with [`outcomes`](gamma.md#struct.Market).
    ///
    /// The spec types this as a plain string and documents no encoding. Live sends a
    /// JSON-encoded string (`"[\"5385...\", \"5268...\"]"`); the optimized search does not
    /// send the field. Both a JSON-encoded string and a real array decode to the list; it
    /// serializes back to the string form.
    pub clob_token_ids: Option<Vec<TokenId>>,
    /// Disqus thread.
    pub disqus_thread: Option<String>,
    /// Short outcomes. The spec types this as a plain string.
    pub short_outcomes: Option<String>,
    /// Team A id (wire name `teamAID`). The spec types this as a string here (a team's
    /// own `id` is an integer); it is kept exactly as sent.
    pub team_a_id: Option<TeamId>,
    /// Team B id (wire name `teamBID`). The spec types this as a string here (a team's
    /// own `id` is an integer); it is kept exactly as sent.
    pub team_b_id: Option<TeamId>,
    /// UMA bond. The spec types this as a string (`umaBond`); it is parsed as a decimal, and
    /// an empty string becomes `None`.
    pub uma_bond: Option<Decimal>,
    /// UMA reward. The spec types this as a string (`umaReward`); it is parsed as a decimal, and
    /// an empty string becomes `None`.
    pub uma_reward: Option<Decimal>,
    /// 24-hour CLOB volume.
    pub volume_24hr_clob: Option<Decimal>,
    /// 1-week CLOB volume.
    pub volume_1wk_clob: Option<Decimal>,
    /// 1-month CLOB volume.
    pub volume_1mo_clob: Option<Decimal>,
    /// 1-year CLOB volume.
    pub volume_1yr_clob: Option<Decimal>,
    /// CLOB volume.
    pub volume_clob: Option<Decimal>,
    /// CLOB liquidity.
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
    pub ready_timestamp: Option<DateTime<Utc>>,
    /// When the market was funded.
    pub funded_timestamp: Option<DateTime<Utc>>,
    /// When the market started accepting orders.
    pub accepting_orders_timestamp: Option<DateTime<Utc>>,
    /// Competitiveness score.
    pub competitive: Option<Decimal>,
    /// Minimum size for liquidity rewards.
    pub rewards_min_size: Option<Decimal>,
    /// Maximum spread for liquidity rewards.
    pub rewards_max_spread: Option<Decimal>,
    /// Current spread.
    pub spread: Option<Decimal>,
    /// Whether the market resolves automatically.
    pub automatically_resolved: Option<bool>,
    /// Price change over one day.
    pub one_day_price_change: Option<Decimal>,
    /// Price change over one hour.
    pub one_hour_price_change: Option<Decimal>,
    /// Price change over one week.
    pub one_week_price_change: Option<Decimal>,
    /// Price change over one month.
    pub one_month_price_change: Option<Decimal>,
    /// Price change over one year.
    pub one_year_price_change: Option<Decimal>,
    /// Last trade price.
    pub last_trade_price: Option<Decimal>,
    /// Best bid.
    pub best_bid: Option<Decimal>,
    /// Best ask.
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
    /// [`GammaClient::get_sports_market_types`](gamma.md#GammaClient.fn.get_sports_market_types)).
    pub sports_market_type: Option<String>,
    /// Line.
    pub line: Option<Decimal>,
    /// UMA resolution statuses (e.g. `["proposed", "resolved"]`, `[]` if none).
    ///
    /// The spec types this as a plain string and documents no encoding. Live sends a
    /// JSON-encoded string of strings; both that and a real array decode to the list, which
    /// serializes back to the string form.
    pub uma_resolution_statuses: Option<Vec<String>>,
    /// Whether deployment is pending.
    pub pending_deployment: Option<bool>,
    /// Whether the market is being deployed.
    pub deploying: Option<bool>,
    /// When deployment started.
    pub deploying_timestamp: Option<DateTime<Utc>>,
    /// When deployment is scheduled.
    pub scheduled_deployment_timestamp: Option<DateTime<Utc>>,
    /// Whether RFQ is enabled.
    pub rfq_enabled: Option<bool>,
    /// Event start time.
    pub event_start_time: Option<DateTime<Utc>>,
    /// Whether fees are enabled.
    pub fees_enabled: Option<bool>,
    /// Fee schedule (wire name `feeSchedule`). The keyset listings' response description
    /// spells it `fee_schedule`; this field reads only the schema's `feeSchedule`.
    pub fee_schedule: Option<FeeSchedule>,
    /// Whether the market is part of a negative-risk group (`negRisk`; undocumented, observed
    /// live). Absent on some markets embedded in events.
    pub neg_risk: Option<bool>,
    /// Id of the negative-risk market group, a 32-byte hex string (wire name
    /// `negRiskMarketID`; undocumented, observed live).
    pub neg_risk_market_id: Option<String>,
    /// Negative-risk request id, a 32-byte hex string (wire name `negRiskRequestID`;
    /// undocumented, observed live).
    pub neg_risk_request_id: Option<String>,
    /// Whether the market is approved (undocumented; observed live).
    pub approved: Option<bool>,
    /// Combo status, e.g. `enabled`, `disabled` or `pending` (undocumented; observed live,
    /// kept as sent).
    pub combo_status: Option<String>,
    /// The `cyom` flag (undocumented; observed live; the `cyom` request filter is documented).
    pub cyom: Option<bool>,
    /// Name of the fee schedule applied to the market, e.g. `politics_fees` (undocumented;
    /// observed live; `null` on some markets).
    pub fee_type: Option<String>,
    /// Whether holding rewards are enabled (undocumented; observed live).
    pub holding_rewards_enabled: Option<bool>,
    /// Whether PagerDuty notifications are enabled (undocumented; observed live).
    pub pager_duty_notification_enabled: Option<bool>,
    /// Position ids (undocumented; observed live as a real JSON array of numeric strings).
    pub position_ids: Option<Vec<String>>,
    /// Address that submitted the market (wire name `submitted_by`; undocumented, observed
    /// live; mixed-case checksum form).
    pub submitted_by: Option<Address>,
    /// Data version, e.g. `v1` (undocumented; observed live).
    pub version: Option<String>,
    /// Provider-specific metadata, a free-form object (undocumented; observed live with
    /// sports-data keys such as `opticOddsFixtureId`; the set of keys varies).
    pub market_metadata: Option<Value>,
    /// Address of the (legacy AMM) market maker (undocumented; observed live).
    pub market_maker_address: Option<Address>,
    /// AMM liquidity (undocumented; observed live as a JSON number).
    pub liquidity_amm: Option<Decimal>,
    /// AMM volume (undocumented; observed live as a JSON number).
    pub volume_amm: Option<Decimal>,
    /// 1-month AMM volume (undocumented; observed live as a JSON number).
    pub volume_1mo_amm: Option<Decimal>,
    /// 1-week AMM volume (undocumented; observed live as a JSON number).
    pub volume_1wk_amm: Option<Decimal>,
    /// 1-year AMM volume (undocumented; observed live as a JSON number).
    pub volume_1yr_amm: Option<Decimal>,
    /// 24-hour AMM volume (undocumented; observed live as a JSON number).
    pub volume_24hr_amm: Option<Decimal>,
    /// Whether the legacy AMM is live (undocumented; observed live on old markets).
    pub fpmm_live: Option<bool>,
    /// Mailchimp category tag, a numeric string or the text `null` (undocumented; observed
    /// live on old markets).
    pub category_mailchimp_tag: Option<String>,
    /// Whether the market was announced on Discord (undocumented; observed live on old
    /// markets).
    pub sent_discord: Option<bool>,
    /// Twitter card image location (undocumented; observed live on old markets).
    pub twitter_card_location: Option<String>,
    /// When the Twitter card was last refreshed: Unix milliseconds as a string
    /// (undocumented; observed live on old markets, kept as sent).
    pub twitter_card_last_refreshed: Option<String>,
    /// When the Twitter card was last validated: Unix seconds with a fraction, as a string
    /// (undocumented; observed live on old markets, kept as sent).
    pub twitter_card_last_validated: Option<String>,
    /// Liquidity-reward programs (undocumented; observed live; absent if the market has
    /// none).
    pub clob_rewards: Option<Vec<ClobReward>>,
    /// Subcategory (undocumented; observed live on old markets).
    pub subcategory: Option<String>,
}
```

A market (`components/schemas/Market`).

Every field is optional because the spec marks none as required. Fields the spec types
as `number` are `Decimal`s and serialize back as JSON numbers. The amounts the spec
types as `string` (`liquidity`, `volume`, `fee`, `umaBond`, `umaReward`) are parsed
into `Decimal`s too: an empty string or `null` becomes `None`, any other non-numeric
text fails decoding, and they serialize back as JSON strings. Other string fields are
kept exactly as sent.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.MarketDescription"></a>`struct MarketDescription`

```rust
#[non_exhaustive]
pub struct MarketDescription {
    /// The description.
    pub description: Option<String>,
}
```

A market's description (`components/schemas/MarketDescription`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.MarketsKeysetPage"></a>`struct MarketsKeysetPage`

```rust
#[non_exhaustive]
pub struct MarketsKeysetPage {
    /// The markets on this page (documented as an empty array if none were found).
    pub markets: Option<Vec<Market>>,
    /// Cursor for the next page, passed to [`cursor`](gamma.md#ListMarketsKeyset.fn.cursor). The spec
    /// documents it as present only when the number of returned markets equals the
    /// effective limit, and omitted on the last page.
    pub next_cursor: Option<String>,
}
```

One page of [`GammaClient::list_markets_keyset`](gamma.md#GammaClient.fn.list_markets_keyset) (`components/schemas/KeysetMarketsResponse`).

The fields keep their wire names; [`items`](gamma.md#MarketsKeysetPage.fn.items),
[`into_items`](gamma.md#MarketsKeysetPage.fn.into_items) and [`next_cursor()`](gamma.md#MarketsKeysetPage.fn.next_cursor) give the same
view as every other page type.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="MarketsKeysetPage.fn.items"></a>`items`

```rust
#[must_use]
pub fn items(&self) -> &[Market]
```

The markets on this page (empty if the page carries none).

##### <a id="MarketsKeysetPage.fn.into_items"></a>`into_items`

```rust
#[must_use]
pub fn into_items(self) -> Vec<Market>
```

Consumes the page and returns its markets.

##### <a id="MarketsKeysetPage.fn.next_cursor"></a>`next_cursor`

```rust
#[must_use]
pub fn next_cursor(&self) -> Option<&str>
```

The cursor for the next page, or `None` on the last page (an absent or empty
`next_cursor`).

### <a id="struct.Pagination"></a>`struct Pagination`

```rust
#[non_exhaustive]
pub struct Pagination {
    /// Whether more results are available.
    pub has_more: Option<bool>,
    /// Total number of results.
    pub total_results: Option<i64>,
}
```

Pagination metadata (`components/schemas/Pagination`), returned by
[`GammaClient::list_events_paginated`](gamma.md#GammaClient.fn.list_events_paginated) and
[`GammaClient::search`](gamma.md#GammaClient.fn.search).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Profile"></a>`struct Profile`

```rust
#[non_exhaustive]
pub struct Profile {
    /// Profile id.
    pub id: Option<String>,
    /// Display name.
    pub name: Option<String>,
    /// User id.
    pub user: Option<i64>,
    /// Referral.
    pub referral: Option<String>,
    /// Id of the user who created the profile.
    pub created_by: Option<i64>,
    /// Id of the user who last updated the profile.
    pub updated_by: Option<i64>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    pub updated_at: Option<DateTime<Utc>>,
    /// UTM source.
    pub utm_source: Option<String>,
    /// UTM medium.
    pub utm_medium: Option<String>,
    /// UTM campaign.
    pub utm_campaign: Option<String>,
    /// UTM content.
    pub utm_content: Option<String>,
    /// UTM term.
    pub utm_term: Option<String>,
    /// Whether the wallet is activated.
    pub wallet_activated: Option<bool>,
    /// Pseudonym.
    pub pseudonym: Option<String>,
    /// Whether the username is displayed publicly.
    pub display_username_public: Option<bool>,
    /// Profile image URL.
    pub profile_image: Option<String>,
    /// Bio.
    pub bio: Option<String>,
    /// Proxy wallet address.
    pub proxy_wallet: Option<Address>,
    /// Optimized profile image metadata.
    pub profile_image_optimized: Option<ImageOptimization>,
    /// Whether the account is close-only.
    pub is_close_only: Option<bool>,
    /// Whether a certification is required.
    pub is_cert_req: Option<bool>,
    /// Certification request date.
    pub cert_req_date: Option<DateTime<Utc>>,
    /// Taker fee tier number, `0` upwards (undocumented; observed live).
    pub taker_tier: Option<i64>,
    /// Taker fee tier name, e.g. `Tier 0`, `Silver`, `Obsidian` (undocumented; observed live,
    /// kept as sent).
    pub taker_tier_name: Option<String>,
    /// Weighted volume used for the taker tier (undocumented; observed live as a JSON
    /// number).
    pub weighted_volume: Option<Decimal>,
}
```

A user profile (`components/schemas/Profile`), returned by [`GammaClient::get_profile`](gamma.md#GammaClient.fn.get_profile)
and [`GammaClient::search`](gamma.md#GammaClient.fn.search).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.PublicProfile"></a>`struct PublicProfile`

```rust
#[non_exhaustive]
pub struct PublicProfile {
    /// When the profile was created.
    pub created_at: Option<DateTime<Utc>>,
    /// The proxy wallet address.
    pub proxy_wallet: Option<Address>,
    /// URL of the profile image.
    pub profile_image: Option<String>,
    /// Whether the username is displayed publicly.
    pub display_username_public: Option<bool>,
    /// Profile bio.
    pub bio: Option<String>,
    /// Auto-generated pseudonym.
    pub pseudonym: Option<String>,
    /// User-chosen display name.
    pub name: Option<String>,
    /// Associated users.
    pub users: Option<Vec<PublicProfileUser>>,
    /// X (Twitter) username.
    pub x_username: Option<String>,
    /// Whether the profile has a verified badge.
    pub verified_badge: Option<bool>,
    /// Taker fee tier number, `0` upwards (undocumented; observed live).
    pub taker_tier: Option<i64>,
    /// Taker fee tier name, e.g. `Tier 0`, `Silver`, `Obsidian` (undocumented; observed live,
    /// kept as sent).
    pub taker_tier_name: Option<String>,
    /// Weighted volume used for the taker tier (undocumented; observed live as a JSON
    /// number).
    pub weighted_volume: Option<Decimal>,
}
```

A public profile (`components/schemas/PublicProfileResponse`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.PublicProfileUser"></a>`struct PublicProfileUser`

```rust
#[non_exhaustive]
pub struct PublicProfileUser {
    /// User id.
    pub id: Option<String>,
    /// Whether the user is a creator.
    pub creator: Option<bool>,
    /// Whether the user is a moderator (wire name `mod`).
    pub is_mod: Option<bool>,
    /// Whether the user is a community moderator (undocumented; observed live).
    pub community_mod: Option<bool>,
}
```

A user associated with a public profile (`components/schemas/PublicProfileUser`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Reaction"></a>`struct Reaction`

```rust
#[non_exhaustive]
pub struct Reaction {
    /// Reaction id.
    pub id: Option<String>,
    /// Id of the comment reacted to (wire name `commentID`, an integer).
    pub comment_id: Option<CommentId>,
    /// Reaction type.
    pub reaction_type: Option<String>,
    /// Icon.
    pub icon: Option<String>,
    /// Address of the user who reacted.
    pub user_address: Option<Address>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Profile of the user who reacted.
    pub profile: Option<CommentProfile>,
}
```

A reaction to a comment (`components/schemas/Reaction`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.RelatedTag"></a>`struct RelatedTag`

```rust
#[non_exhaustive]
pub struct RelatedTag {
    /// Relationship id.
    pub id: Option<String>,
    /// The tag (wire name `tagID`, an integer).
    pub tag_id: Option<TagId>,
    /// The related tag (wire name `relatedTagID`, an integer).
    pub related_tag_id: Option<TagId>,
    /// Rank of the relationship.
    pub rank: Option<i64>,
}
```

A relationship between two tags (`components/schemas/RelatedTag`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Search"></a>`struct Search`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct Search { /* private fields */ }
```

Request builder for [`GammaClient::search`](gamma.md#GammaClient.fn.search).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="Search.fn.cache"></a>`cache`

```rust
pub fn cache(self, cache: bool) -> Self
```

The `cache` flag (documented only as a boolean).

##### <a id="Search.fn.events_status"></a>`events_status`

```rust
pub fn events_status(self, events_status: impl Into<String>) -> Self
```

The `events_status` filter (a string; the spec documents no values).

##### <a id="Search.fn.limit_per_type"></a>`limit_per_type`

```rust
pub fn limit_per_type(self, limit_per_type: u32) -> Self
```

The `limit_per_type` parameter (an integer).

##### <a id="Search.fn.page"></a>`page`

```rust
pub fn page(self, page: u32) -> Self
```

The `page` parameter (an integer; the docs do not say whether pages are numbered
from 0 or 1).

##### <a id="Search.fn.events_tags"></a>`events_tags`

```rust
pub fn events_tags<I>(self, events_tags: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<String>,
```

The `events_tag` filter (repeated strings).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="Search.fn.keep_closed_markets"></a>`keep_closed_markets`

```rust
pub fn keep_closed_markets(self, keep_closed_markets: i64) -> Self
```

The `keep_closed_markets` parameter (an integer; the spec documents no values).

##### <a id="Search.fn.sort"></a>`sort`

```rust
pub fn sort(self, sort: impl Into<String>) -> Self
```

The `sort` parameter (a string; the spec documents no values).

##### <a id="Search.fn.ascending"></a>`ascending`

```rust
pub fn ascending(self, ascending: bool) -> Self
```

Sort ascending (`true`) or descending (`false`) (`ascending`).

##### <a id="Search.fn.search_tags"></a>`search_tags`

```rust
pub fn search_tags(self, search_tags: bool) -> Self
```

The `search_tags` flag (documented only as a boolean).

##### <a id="Search.fn.search_profiles"></a>`search_profiles`

```rust
pub fn search_profiles(self, search_profiles: bool) -> Self
```

The `search_profiles` flag (documented only as a boolean).

##### <a id="Search.fn.recurrence"></a>`recurrence`

```rust
pub fn recurrence(self, recurrence: impl Into<String>) -> Self
```

The `recurrence` filter (a string; the spec documents no values).

##### <a id="Search.fn.exclude_tag_ids"></a>`exclude_tag_ids`

```rust
pub fn exclude_tag_ids<I>(self, exclude_tag_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TagId>,
```

Tag ids to exclude (`exclude_tag_id`, repeated). The spec types them as
 integers, so an id that is not one or more ASCII digits is rejected before
 sending with [`Error::Validation`](marcasite.md#enum.Error).

Replaces any previously set values; an empty iterator removes the filter.

##### <a id="Search.fn.optimized"></a>`optimized`

```rust
pub fn optimized(self, optimized: bool) -> Self
```

The `optimized` flag (documented only as a boolean).

With `true` live returns a slimmer result: the events and their markets carry only
a few fields (so most [`Event`](gamma.md#struct.Event) and [`Market`](gamma.md#struct.Market) fields are `None`),
the markets' `outcomes` and `outcomePrices` are real JSON arrays instead of
JSON-encoded strings (both decode to the same lists), and the response has
[`has_more`](gamma.md#struct.SearchResults) instead of
[`pagination`](gamma.md#struct.SearchResults). See `SPEC_DEVIATIONS.md`.

##### <a id="Search.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<SearchResults>
```

Sends the request.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) (parameter `exclude_tag_id`) if an
  [`exclude_tag_ids`](gamma.md#Search.fn.exclude_tag_ids) entry is not an integer, checked before
  sending;
- otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.SearchResults"></a>`struct SearchResults`

```rust
#[non_exhaustive]
pub struct SearchResults {
    /// Matching events.
    pub events: Option<Vec<Event>>,
    /// Matching tags.
    pub tags: Option<Vec<SearchTag>>,
    /// Matching profiles.
    pub profiles: Option<Vec<Profile>>,
    /// Pagination metadata. Sent by the regular search; the
    /// [`optimized`](gamma.md#Search.fn.optimized) search sends [`has_more`](gamma.md#struct.SearchResults) instead.
    pub pagination: Option<Pagination>,
    /// Whether more results are available (wire name `hasMore`; undocumented, observed live
    /// only with `optimized=true`, where it replaces [`pagination`](gamma.md#struct.SearchResults)).
    pub has_more: Option<bool>,
}
```

Results of [`GammaClient::search`](gamma.md#GammaClient.fn.search) (`components/schemas/Search`).

The spec calls this schema `Search`; in this crate [`Search`](gamma.md#struct.Search) is the request builder.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.SearchTag"></a>`struct SearchTag`

```rust
#[non_exhaustive]
pub struct SearchTag {
    /// Tag id.
    pub id: Option<TagId>,
    /// Display label.
    pub label: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Number of events with the tag.
    pub event_count: Option<i64>,
}
```

A tag matched by [`GammaClient::search`](gamma.md#GammaClient.fn.search) (`components/schemas/SearchTag`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Series"></a>`struct Series`

```rust
#[non_exhaustive]
pub struct Series {
    /// Series id.
    pub id: Option<SeriesId>,
    /// Ticker.
    pub ticker: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Subtitle.
    pub subtitle: Option<String>,
    /// Series type.
    pub series_type: Option<String>,
    /// Recurrence.
    pub recurrence: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Image URL.
    pub image: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Layout.
    pub layout: Option<String>,
    /// Whether the series is active.
    pub active: Option<bool>,
    /// Whether the series is closed.
    pub closed: Option<bool>,
    /// Whether the series is archived.
    pub archived: Option<bool>,
    /// Whether the series is flagged as new.
    pub new: Option<bool>,
    /// Whether the series is featured.
    pub featured: Option<bool>,
    /// Whether the series is restricted.
    pub restricted: Option<bool>,
    /// Whether the series is a template.
    pub is_template: Option<bool>,
    /// Template variables. The spec types this as a boolean on series (a string on events
    /// and collections).
    pub template_variables: Option<bool>,
    /// Publication time. The spec types this as a plain string with no format.
    pub published_at: Option<String>,
    /// Creator.
    pub created_by: Option<String>,
    /// Last updater.
    pub updated_by: Option<String>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    pub updated_at: Option<DateTime<Utc>>,
    /// Whether comments are enabled.
    pub comments_enabled: Option<bool>,
    /// Competitiveness. The spec types this as a string on series (a number on events and
    /// markets), so it is kept exactly as sent.
    pub competitive: Option<String>,
    /// 24-hour volume.
    pub volume_24hr: Option<Decimal>,
    /// Volume.
    pub volume: Option<Decimal>,
    /// Liquidity.
    pub liquidity: Option<Decimal>,
    /// Start date.
    pub start_date: Option<DateTime<Utc>>,
    /// Pyth token id (wire name `pythTokenID`).
    pub pyth_token_id: Option<String>,
    /// CoinGecko asset name.
    pub cg_asset_name: Option<String>,
    /// Score.
    pub score: Option<i64>,
    /// The series' events.
    pub events: Option<Vec<Event>>,
    /// Collections.
    pub collections: Option<Vec<Collection>>,
    /// Categories.
    pub categories: Option<Vec<Category>>,
    /// Tags.
    pub tags: Option<Vec<Tag>>,
    /// Number of comments.
    pub comment_count: Option<i64>,
    /// Chats. The keyset event listing documents a series' chats as included only with
    /// `include_chat=true`.
    pub chats: Option<Vec<Chat>>,
}
```

A series: a recurring set of events (`components/schemas/Series`).

Every field is optional because the spec marks none as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.SeriesId"></a>`struct SeriesId`

```rust
pub struct SeriesId(/* private fields */);
```

A Gamma series id (sent as a string in responses; an integer in paths and filters).

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&SeriesId>`, `From<&String>`, `From<&str>`, `From<SeriesId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="SeriesId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="SeriesId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="SeriesId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.SeriesSummary"></a>`struct SeriesSummary`

```rust
#[non_exhaustive]
pub struct SeriesSummary {
    /// Series id.
    pub id: Option<SeriesId>,
    /// Title.
    pub title: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Event dates. The spec types these as plain strings with no format.
    pub event_dates: Option<Vec<String>>,
    /// Event weeks.
    pub event_weeks: Option<Vec<i64>>,
    /// Earliest week with an open event (wire name `earliest_open_week`).
    pub earliest_open_week: Option<i64>,
    /// Earliest date with an open event (wire name `earliest_open_date`). The spec types
    /// this as a plain string with no format.
    pub earliest_open_date: Option<String>,
    /// Total volume of the series (undocumented; observed live as a JSON number).
    pub volume: Option<Decimal>,
    /// 24-hour volume of the series (undocumented; observed live as a JSON number).
    pub volume_24hr: Option<Decimal>,
}
```

A summary of a series' event dates (`components/schemas/SeriesSummary`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.SportsMarketTypes"></a>`struct SportsMarketTypes`

```rust
#[non_exhaustive]
pub struct SportsMarketTypes {
    /// Every valid sports market type.
    pub market_types: Option<Vec<String>>,
}
```

The valid sports market types (`components/schemas/SportsMarketTypesResponse`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.SportsMetadata"></a>`struct SportsMetadata`

```rust
#[non_exhaustive]
pub struct SportsMetadata {
    /// Sport id (undocumented; observed live as an integer).
    pub id: Option<i64>,
    /// Display name, e.g. `NFL` (undocumented; observed live).
    pub name: Option<String>,
    /// Creation time (undocumented; observed live).
    pub created_at: Option<DateTime<Utc>>,
    /// Id of the sport's primary tag (undocumented; observed live as an integer).
    pub primary_tag_id: Option<i64>,
    /// The sport identifier or abbreviation.
    pub sport: Option<String>,
    /// URL of the sport's logo or image.
    pub image: Option<String>,
    /// URL of the official resolution source for the sport (e.g. the league website).
    pub resolution: Option<String>,
    /// Preferred ordering for display, typically `"home"` or `"away"`.
    pub ordering: Option<String>,
    /// Comma-separated list of tag ids associated with the sport; see
    /// [`tag_ids`](gamma.md#SportsMetadata.fn.tag_ids).
    pub tags: Option<String>,
    /// Series identifier linking the sport to a tournament or season series (a string on
    /// the wire).
    pub series: Option<SeriesId>,
}
```

Configuration of one sport (`components/schemas/SportsMetadata`), also embedded in
events as [`Event::sport`](gamma.md#struct.Event).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="SportsMetadata.fn.tag_ids"></a>`tag_ids`

```rust
pub fn tag_ids(&self) -> impl Iterator<Item = TagId> + '_
```

The tag ids in [`tags`](gamma.md#struct.SportsMetadata), split on commas (empty entries are skipped).

### <a id="struct.Tag"></a>`struct Tag`

```rust
#[non_exhaustive]
pub struct Tag {
    /// Tag id.
    pub id: Option<TagId>,
    /// Display label.
    pub label: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Whether the tag is always shown.
    pub force_show: Option<bool>,
    /// Publication time. The spec types this as a plain string with no format.
    pub published_at: Option<String>,
    /// Id of the user who created the tag.
    pub created_by: Option<i64>,
    /// Id of the user who last updated the tag.
    pub updated_by: Option<i64>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    pub updated_at: Option<DateTime<Utc>>,
    /// Whether the tag is always hidden.
    pub force_hide: Option<bool>,
    /// Whether the tag is shown in the carousel.
    pub is_carousel: Option<bool>,
    /// Number of active events with the tag (undocumented; observed live on the tags
    /// returned by [`GammaClient::get_related_tags`](gamma.md#GammaClient.fn.get_related_tags)).
    pub active_events_count: Option<i64>,
}
```

A tag used to categorise events, markets and series.

Every field is optional because the spec (`components/schemas/Tag`) marks none as
required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.TagId"></a>`struct TagId`

```rust
pub struct TagId(/* private fields */);
```

A Gamma tag id (sent as a string in responses, e.g. `"100381"`).

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&String>`, `From<&TagId>`, `From<&str>`, `From<String>`, `From<TagId>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="TagId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="TagId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="TagId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.Team"></a>`struct Team`

```rust
#[non_exhaustive]
pub struct Team {
    /// Team id (an integer on the wire).
    pub id: Option<TeamId>,
    /// Team name.
    pub name: Option<String>,
    /// League.
    pub league: Option<String>,
    /// Record.
    pub record: Option<String>,
    /// Logo URL.
    pub logo: Option<String>,
    /// Abbreviation.
    pub abbreviation: Option<String>,
    /// Alias.
    pub alias: Option<String>,
    /// Team colour as a CSS hex string, e.g. `#E0A000` (undocumented; observed live; absent
    /// on some teams).
    pub color: Option<String>,
    /// Id of the team at the sports-data provider (wire name `providerId`; undocumented,
    /// observed live as an integer; absent on some teams).
    pub provider_id: Option<i64>,
    /// `home` or `away` (undocumented; observed live on the teams embedded in an event's
    /// `teams`).
    pub ordering: Option<String>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    pub updated_at: Option<DateTime<Utc>>,
}
```

A sports team (`components/schemas/Team`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.TeamId"></a>`struct TeamId`

```rust
pub struct TeamId(/* private fields */);
```

A Gamma team id (an integer on the wire).

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&String>`, `From<&TeamId>`, `From<&str>`, `From<String>`, `From<TeamId>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="TeamId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="TeamId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="TeamId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.Template"></a>`struct Template`

```rust
#[non_exhaustive]
pub struct Template {
    /// Template id.
    pub id: Option<String>,
    /// Event title.
    pub event_title: Option<String>,
    /// Event slug.
    pub event_slug: Option<String>,
    /// Event image URL.
    pub event_image: Option<String>,
    /// Market title.
    pub market_title: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Resolution source.
    pub resolution_source: Option<String>,
    /// Whether the event uses negative risk.
    pub neg_risk: Option<bool>,
    /// Sort order.
    pub sort_by: Option<String>,
    /// Whether market images are shown.
    pub show_market_images: Option<bool>,
    /// Series slug.
    pub series_slug: Option<String>,
    /// Outcomes. The spec types this as a plain string.
    pub outcomes: Option<String>,
}
```

An event template (`components/schemas/Template`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

## Enums

### <a id="enum.CommentParentEntityType"></a>`enum CommentParentEntityType`

```rust
#[non_exhaustive]
pub enum CommentParentEntityType {
    /// An event (`parent_entity_id` is an event id).
    Event,
    /// A series (`parent_entity_id` is a series id).
    Series,
    /// A perpetuals asset (undocumented; accepted live).
    PerpsAsset,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

The type of entity a comment is attached to (the `parent_entity_type` of
[`GammaClient::list_comments`](gamma.md#GammaClient.fn.list_comments) and of [`Comment::parent_entity_type`](gamma.md#struct.Comment)).

The spec lists `Event`, `Series` and `market`; live accepts `Event`, `Series` and
`PerpsAsset` and rejects `market` with a `422` (see `SPEC_DEVIATIONS.md`), so there is
no `Market` variant.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="CommentParentEntityType.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="CommentParentEntityType.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.RelatedTagsStatus"></a>`enum RelatedTagsStatus`

```rust
#[non_exhaustive]
pub enum RelatedTagsStatus {
    /// The value `active`.
    Active,
    /// The value `closed`.
    Closed,
    /// The value `all`.
    All,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

The `status` filter of the related-tags endpoints (the spec documents the values,
not their meaning).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="RelatedTagsStatus.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="RelatedTagsStatus.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.
