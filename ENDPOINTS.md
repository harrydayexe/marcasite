# Endpoint checklist

Implementation status of every Polymarket Predictions API endpoint in polyoxide. **Keep this file
up to date** whenever an endpoint is added or changed. It is the single place to check coverage.

- Scope (current): **unauthenticated** endpoints only. Endpoints requiring CLOB L1/L2 auth, Builder
  API keys or Relayer API keys are listed under "Out of scope" at the bottom.
- Columns: *Docs* links the local verbatim copy in `docs/` and the online page (pages that exist only
  in the spec link to the spec file + `operationId`); *Rust* is the client method; *Status* is
  `[x]` implemented and tested, `[ ]` not yet implemented.
- Base URLs are defaults; every client accepts an override (see `*ClientBuilder::base_url`).


## Gamma API

Base URL `https://gamma-api.polymarket.com`, spec [`docs/specs/gamma-openapi.yaml`](docs/specs/gamma-openapi.yaml), feature `gamma`, module `polyoxide::gamma`.

| Status | Method | Path | Docs | Rust |
|---|---|---|---|---|
| [x] | `GET` | `/status` | spec only (`getGammaStatus`) | `GammaClient::get_status` |
| [x] | `GET` | `/teams` | [local](docs/api-reference/sports/list-teams.md) / [online](https://docs.polymarket.com/api-reference/sports/list-teams) | `GammaClient::list_teams` |
| [x] | `GET` | `/teams/{id}` | spec only (`getTeam`) | `GammaClient::get_team` |
| [x] | `GET` | `/tags` | [local](docs/api-reference/tags/list-tags.md) / [online](https://docs.polymarket.com/api-reference/tags/list-tags) | `GammaClient::list_tags` |
| [x] | `GET` | `/tags/{id}` | [local](docs/api-reference/tags/get-tag-by-id.md) / [online](https://docs.polymarket.com/api-reference/tags/get-tag-by-id) | `GammaClient::get_tag` |
| [x] | `GET` | `/tags/slug/{slug}` | [local](docs/api-reference/tags/get-tag-by-slug.md) / [online](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug) | `GammaClient::get_tag_by_slug` |
| [x] | `GET` | `/tags/{id}/related-tags` | [local](docs/api-reference/tags/get-related-tags-relationships-by-tag-id.md) / [online](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id) | `GammaClient::get_related_tag_relationships` |
| [x] | `GET` | `/tags/slug/{slug}/related-tags` | [local](docs/api-reference/tags/get-related-tags-relationships-by-tag-slug.md) / [online](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-slug) | `GammaClient::get_related_tag_relationships_by_slug` |
| [x] | `GET` | `/tags/{id}/related-tags/tags` | [local](docs/api-reference/tags/get-tags-related-to-a-tag-id.md) / [online](https://docs.polymarket.com/api-reference/tags/get-tags-related-to-a-tag-id) | `GammaClient::get_related_tags` |
| [x] | `GET` | `/tags/slug/{slug}/related-tags/tags` | [local](docs/api-reference/tags/get-tags-related-to-a-tag-slug.md) / [online](https://docs.polymarket.com/api-reference/tags/get-tags-related-to-a-tag-slug) | `GammaClient::get_related_tags_by_slug` |
| [x] | `GET` | `/events` | [local](docs/api-reference/events/list-events.md) / [online](https://docs.polymarket.com/api-reference/events/list-events) | `GammaClient::list_events` |
| [x] | `GET` | `/events/pagination` | spec only (`listEventsPagination`) | `GammaClient::list_events_paginated` |
| [x] | `GET` | `/events/results` | spec only (`listSportEventsResults`) | `GammaClient::list_sport_event_results` |
| [x] | `GET` | `/events/{id}` | [local](docs/api-reference/events/get-event-by-id.md) / [online](https://docs.polymarket.com/api-reference/events/get-event-by-id) | `GammaClient::get_event` |
| [x] | `GET` | `/events/{id}/tweet-count` | spec only (`getEventTweetCount`) | `GammaClient::get_event_tweet_count` |
| [x] | `GET` | `/events/{id}/comments/count` | spec only (`getEventCommentsCount`) | `GammaClient::get_event_comment_count` |
| [x] | `GET` | `/events/{id}/tags` | [local](docs/api-reference/events/get-event-tags.md) / [online](https://docs.polymarket.com/api-reference/events/get-event-tags) | `GammaClient::get_event_tags` |
| [x] | `GET` | `/events/slug/{slug}` | [local](docs/api-reference/events/get-event-by-slug.md) / [online](https://docs.polymarket.com/api-reference/events/get-event-by-slug) | `GammaClient::get_event_by_slug` |
| [x] | `GET` | `/events/creators` | spec only (`listEventCreators`) | `GammaClient::list_event_creators` |
| [x] | `GET` | `/events/creators/{id}` | spec only (`getEventCreator`) | `GammaClient::get_event_creator` |
| [x] | `GET` | `/markets` | [local](docs/api-reference/markets/list-markets.md) / [online](https://docs.polymarket.com/api-reference/markets/list-markets) | `GammaClient::list_markets` |
| [x] | `GET` | `/markets/{id}` | [local](docs/api-reference/markets/get-market-by-id.md) / [online](https://docs.polymarket.com/api-reference/markets/get-market-by-id) | `GammaClient::get_market` |
| [x] | `GET` | `/markets/{id}/description` | spec only (`getMarketDescription`) | `GammaClient::get_market_description` |
| [x] | `GET` | `/markets/{id}/tags` | [local](docs/api-reference/markets/get-market-tags-by-id.md) / [online](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id) | `GammaClient::get_market_tags` |
| [x] | `GET` | `/markets/slug/{slug}` | [local](docs/api-reference/markets/get-market-by-slug.md) / [online](https://docs.polymarket.com/api-reference/markets/get-market-by-slug) | `GammaClient::get_market_by_slug` |
| [x] | `POST` | `/markets/information` | spec only (`getMarketsInformation`) | `GammaClient::get_markets_information` |
| [x] | `POST` | `/markets/abridged` | spec only (`getAbridgedMarkets`) | `GammaClient::get_abridged_markets` |
| [x] | `GET` | `/markets/keyset` | [local](docs/api-reference/markets/list-markets-keyset-pagination.md) / [online](https://docs.polymarket.com/api-reference/markets/list-markets-keyset-pagination) | `GammaClient::list_markets_keyset` |
| [x] | `GET` | `/events/keyset` | [local](docs/api-reference/events/list-events-keyset-pagination.md) / [online](https://docs.polymarket.com/api-reference/events/list-events-keyset-pagination) | `GammaClient::list_events_keyset` |
| [x] | `GET` | `/series` | [local](docs/api-reference/series/list-series.md) / [online](https://docs.polymarket.com/api-reference/series/list-series) | `GammaClient::list_series` |
| [x] | `GET` | `/series/{id}` | [local](docs/api-reference/series/get-series-by-id.md) / [online](https://docs.polymarket.com/api-reference/series/get-series-by-id) | `GammaClient::get_series` |
| [x] | `GET` | `/series/{id}/comments/count` | spec only (`getSeriesCommentsCount`) | `GammaClient::get_series_comment_count` |
| [x] | `GET` | `/series-summary/{id}` | spec only (`getSeriesSummaryById`) | `GammaClient::get_series_summary` |
| [x] | `GET` | `/series-summary/slug/{slug}` | spec only (`getSeriesSummaryBySlug`) | `GammaClient::get_series_summary_by_slug` |
| [x] | `GET` | `/comments` | [local](docs/api-reference/comments/list-comments.md) / [online](https://docs.polymarket.com/api-reference/comments/list-comments) | `GammaClient::list_comments` |
| [x] | `GET` | `/comments/{id}` | [local](docs/api-reference/comments/get-comments-by-comment-id.md) / [online](https://docs.polymarket.com/api-reference/comments/get-comments-by-comment-id) | `GammaClient::get_comments_by_id` |
| [x] | `GET` | `/comments/user_address/{user_address}` | [local](docs/api-reference/comments/get-comments-by-user-address.md) / [online](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address) | `GammaClient::list_comments_by_user` |
| [x] | `GET` | `/public-profile` | [local](docs/api-reference/profiles/get-public-profile-by-wallet-address.md) / [online](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address) | `GammaClient::get_public_profile` |
| [x] | `GET` | `/profiles/user_address/{user_address}` | spec only (`getPublicProfileByUserAddress`) | `GammaClient::get_profile` |
| [x] | `GET` | `/sports` | [local](docs/api-reference/sports/get-sports-metadata-information.md) / [online](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information) | `GammaClient::get_sports_metadata` |
| [x] | `GET` | `/sports/market-types` | [local](docs/api-reference/sports/get-valid-sports-market-types.md) / [online](https://docs.polymarket.com/api-reference/sports/get-valid-sports-market-types) | `GammaClient::get_sports_market_types` |
| [x] | `GET` | `/public-search` | [local](docs/api-reference/search/search-markets-events-and-profiles.md) / [online](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles) | `GammaClient::search` |

## CLOB API (public market data)

Base URL `https://clob.polymarket.com`, spec [`docs/specs/clob-openapi.yaml`](docs/specs/clob-openapi.yaml), feature `clob`, module `polyoxide::clob`.

| Status | Method | Path | Docs | Rust |
|---|---|---|---|---|
| [x] | `GET` | `/time` | [local](docs/api-reference/data/get-server-time.md) / [online](https://docs.polymarket.com/api-reference/data/get-server-time) | `ClobClient::get_server_time` |
| [x] | `GET` | `/midpoint` | [local](docs/api-reference/data/get-midpoint-price.md) / [online](https://docs.polymarket.com/api-reference/data/get-midpoint-price) | `ClobClient::get_midpoint` |
| [ ] | `GET` | `/midpoints` | [local](docs/api-reference/market-data/get-midpoint-prices-query-parameters.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-midpoint-prices-query-parameters) | none: not implemented, answers `400 Invalid payload` live (see [`SPEC_DEVIATIONS.md`](SPEC_DEVIATIONS.md)); use the `POST` form |
| [x] | `POST` | `/midpoints` | [local](docs/api-reference/market-data/get-midpoint-prices-request-body.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-midpoint-prices-request-body) | `ClobClient::get_midpoints` |
| [x] | `GET` | `/spread` | [local](docs/api-reference/market-data/get-spread.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-spread) | `ClobClient::get_spread` |
| [x] | `POST` | `/spreads` | [local](docs/api-reference/market-data/get-spreads.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-spreads) | `ClobClient::get_spreads` |
| [x] | `GET` | `/last-trade-price` | [local](docs/api-reference/market-data/get-last-trade-price.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-last-trade-price) | `ClobClient::get_last_trade_price` |
| [ ] | `GET` | `/last-trades-prices` | [local](docs/api-reference/market-data/get-last-trade-prices-query-parameters.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-last-trade-prices-query-parameters) | none: not implemented, answers `400 Invalid payload` live (see [`SPEC_DEVIATIONS.md`](SPEC_DEVIATIONS.md)); use the `POST` form |
| [x] | `POST` | `/last-trades-prices` | [local](docs/api-reference/market-data/get-last-trade-prices-request-body.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-last-trade-prices-request-body) | `ClobClient::get_last_trade_prices` |
| [x] | `GET` | `/fee-rate` | [local](docs/api-reference/market-data/get-fee-rate.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-fee-rate) | `ClobClient::get_fee_rate` |
| [x] | `GET` | `/fee-rate/{token_id}` | [local](docs/api-reference/market-data/get-fee-rate-by-path-parameter.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-fee-rate-by-path-parameter) | `ClobClient::get_fee_rate_by_path` |
| [x] | `GET` | `/tick-size` | [local](docs/api-reference/market-data/get-tick-size.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-tick-size) | `ClobClient::get_tick_size` |
| [x] | `GET` | `/tick-size/{token_id}` | [local](docs/api-reference/market-data/get-tick-size-by-path-parameter.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-tick-size-by-path-parameter) | `ClobClient::get_tick_size_by_path` |
| [x] | `GET` | `/neg-risk` | spec only (`getNegRisk`) | `ClobClient::get_neg_risk` |
| [x] | `GET` | `/neg-risk/{token_id}` | spec only (`getNegRiskByPath`) | `ClobClient::get_neg_risk_by_path` |
| [x] | `GET` | `/price` | [local](docs/api-reference/market-data/get-market-price.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-market-price) | `ClobClient::get_price` |
| [ ] | `GET` | `/prices` | [local](docs/api-reference/market-data/get-market-prices-query-parameters.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-market-prices-query-parameters) | none: not implemented, answers `400 Invalid payload` live (see [`SPEC_DEVIATIONS.md`](SPEC_DEVIATIONS.md)); use the `POST` form |
| [x] | `POST` | `/prices` | [local](docs/api-reference/market-data/get-market-prices-request-body.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-market-prices-request-body) | `ClobClient::get_prices` |
| [x] | `GET` | `/book` | [local](docs/api-reference/market-data/get-order-book.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-order-book) | `ClobClient::get_order_book` |
| [ ] | `GET` | `/books` | spec only (`getBooksGet`) | none: not implemented, answers `400 Invalid payload` live (see [`SPEC_DEVIATIONS.md`](SPEC_DEVIATIONS.md)); use the `POST` form |
| [x] | `POST` | `/books` | [local](docs/api-reference/market-data/get-order-books-request-body.md) / [online](https://docs.polymarket.com/api-reference/market-data/get-order-books-request-body) | `ClobClient::get_order_books` |
| [x] | `GET` | `/simplified-markets` | [local](docs/api-reference/markets/get-simplified-markets.md) / [online](https://docs.polymarket.com/api-reference/markets/get-simplified-markets) | `ClobClient::list_simplified_markets` |
| [x] | `GET` | `/sampling-markets` | [local](docs/api-reference/markets/get-sampling-markets.md) / [online](https://docs.polymarket.com/api-reference/markets/get-sampling-markets) | `ClobClient::list_sampling_markets` |
| [x] | `GET` | `/sampling-simplified-markets` | [local](docs/api-reference/markets/get-sampling-simplified-markets.md) / [online](https://docs.polymarket.com/api-reference/markets/get-sampling-simplified-markets) | `ClobClient::list_sampling_simplified_markets` |
| [x] | `GET` | `/clob-markets/{condition_id}` | [local](docs/api-reference/markets/get-clob-market-info.md) / [online](https://docs.polymarket.com/api-reference/markets/get-clob-market-info) | `ClobClient::get_clob_market_info` |
| [x] | `GET` | `/markets-by-token/{token_id}` | [local](docs/api-reference/markets/get-market-by-token.md) / [online](https://docs.polymarket.com/api-reference/markets/get-market-by-token) | `ClobClient::get_market_by_token` |
| [x] | `POST` | `/markets/live-activity` | spec only (`getMarketsLiveActivity`) | `ClobClient::get_markets_live_activity` |
| [x] | `GET` | `/markets/live-activity/{condition_id}` | spec only (`getMarketLiveActivity`) | `ClobClient::get_market_live_activity` |
| [x] | `GET` | `/prices-history` | [local](docs/api-reference/markets/get-prices-history.md) / [online](https://docs.polymarket.com/api-reference/markets/get-prices-history) | `ClobClient::get_prices_history` |
| [x] | `POST` | `/batch-prices-history` | [local](docs/api-reference/markets/get-batch-prices-history.md) / [online](https://docs.polymarket.com/api-reference/markets/get-batch-prices-history) | `ClobClient::get_batch_prices_history` |
| [x] | `GET` | `/rewards/markets/current` | [local](docs/api-reference/rewards/get-current-active-rewards-configurations.md) / [online](https://docs.polymarket.com/api-reference/rewards/get-current-active-rewards-configurations) | `ClobClient::list_current_rewards` |
| [x] | `GET` | `/rewards/markets/{condition_id}` | [local](docs/api-reference/rewards/get-raw-rewards-for-a-specific-market.md) / [online](https://docs.polymarket.com/api-reference/rewards/get-raw-rewards-for-a-specific-market) | `ClobClient::list_raw_rewards_for_market` |
| [x] | `GET` | `/rewards/markets/multi` | [local](docs/api-reference/rewards/get-multiple-markets-with-rewards.md) / [online](https://docs.polymarket.com/api-reference/rewards/get-multiple-markets-with-rewards) | `ClobClient::list_markets_with_rewards` |
| [x] | `GET` | `/rebates/current` | [local](docs/api-reference/rebates/get-current-rebated-fees-for-a-maker.md) / [online](https://docs.polymarket.com/api-reference/rebates/get-current-rebated-fees-for-a-maker) | `ClobClient::get_current_rebated_fees` |
| [x] | `GET` | `/builder/trades` | [local](docs/api-reference/trade/get-builder-trades.md) / [online](https://docs.polymarket.com/api-reference/trade/get-builder-trades) | `ClobClient::list_builder_trades` |

## Data API v2

Base URL `https://data-api.polymarket.com`, spec [`docs/specs/data-v2-openapi.json`](docs/specs/data-v2-openapi.json), feature `data`, module `polyoxide::data`.

| Status | Method | Path | Docs | Rust |
|---|---|---|---|---|
| [x] | `GET` | `/v2/activity` | [local](docs/api-reference/feeds/list-account-activity.md) / [online](https://docs.polymarket.com/api-reference/feeds/list-account-activity) | `DataClient::list_activity` |
| [x] | `GET` | `/v2/activity/combos` | [local](docs/api-reference/feeds/list-combo-activity.md) / [online](https://docs.polymarket.com/api-reference/feeds/list-combo-activity) | `DataClient::list_combo_activity` |
| [x] | `GET` | `/v2/approvals` | [local](docs/api-reference/wallet/get-wallet-approvals.md) / [online](https://docs.polymarket.com/api-reference/wallet/get-wallet-approvals) | `DataClient::get_approvals` |
| [x] | `GET` | `/v2/biggest-winners` | [local](docs/api-reference/boards/list-the-biggest-wins.md) / [online](https://docs.polymarket.com/api-reference/boards/list-the-biggest-wins) | `DataClient::list_biggest_winners` |
| [x] | `GET` | `/v2/builders/leaderboard` | [local](docs/api-reference/boards/get-the-builders-leaderboard.md) / [online](https://docs.polymarket.com/api-reference/boards/get-the-builders-leaderboard) | `DataClient::list_builders_leaderboard` |
| [x] | `GET` | `/v2/builders/volume` | [local](docs/api-reference/boards/get-builder-volume-over-time.md) / [online](https://docs.polymarket.com/api-reference/boards/get-builder-volume-over-time) | `DataClient::get_builders_volume` |
| [x] | `GET` | `/v2/holders` | [local](docs/api-reference/markets/list-a-markets-top-holders.md) / [online](https://docs.polymarket.com/api-reference/markets/list-a-markets-top-holders) | `DataClient::list_holders` |
| [x] | `GET` | `/v2/leaderboard` | [local](docs/api-reference/boards/get-the-trader-leaderboard.md) / [online](https://docs.polymarket.com/api-reference/boards/get-the-trader-leaderboard) | `DataClient::list_leaderboard`, `DataClient::get_leaderboard_standing` (`user=`) |
| [x] | `GET` | `/v2/live-volume` | [local](docs/api-reference/markets/get-live-volume-for-an-event.md) / [online](https://docs.polymarket.com/api-reference/markets/get-live-volume-for-an-event) | `DataClient::get_live_volume` |
| [x] | `GET` | `/v2/oi` | [local](docs/api-reference/markets/get-open-interest.md) / [online](https://docs.polymarket.com/api-reference/markets/get-open-interest) | `DataClient::get_open_interest` |
| [x] | `GET` | `/v2/positions` | [local](docs/api-reference/wallet/list-positions-for-a-user-or-market.md) / [online](https://docs.polymarket.com/api-reference/wallet/list-positions-for-a-user-or-market) | `DataClient::list_positions` |
| [x] | `GET` | `/v2/positions/combos` | [local](docs/api-reference/wallet/list-combo-positions.md) / [online](https://docs.polymarket.com/api-reference/wallet/list-combo-positions) | `DataClient::list_combo_positions` |
| [x] | `GET` | `/v2/prices-history` | [local](docs/api-reference/markets/get-a-tokens-price-history.md) / [online](https://docs.polymarket.com/api-reference/markets/get-a-tokens-price-history) | `DataClient::list_prices_history` |
| [x] | `GET` | `/v2/resolutions` | [local](docs/api-reference/markets/get-resolution-state.md) / [online](https://docs.polymarket.com/api-reference/markets/get-resolution-state) | `DataClient::get_resolutions` |
| [x] | `GET` | `/v2/status` | [local](docs/api-reference/service/get-data-freshness.md) / [online](https://docs.polymarket.com/api-reference/service/get-data-freshness) | `DataClient::get_status` |
| [x] | `GET` | `/v2/trades` | [local](docs/api-reference/feeds/list-trades.md) / [online](https://docs.polymarket.com/api-reference/feeds/list-trades) | `DataClient::list_trades` |
| [x] | `GET` | `/v2/user-pnl` | [local](docs/api-reference/wallet/get-a-users-pnl-series.md) / [online](https://docs.polymarket.com/api-reference/wallet/get-a-users-pnl-series) | `DataClient::get_user_pnl` |
| [x] | `GET` | `/v2/user-stats` | [local](docs/api-reference/wallet/get-a-users-profile-stats.md) / [online](https://docs.polymarket.com/api-reference/wallet/get-a-users-profile-stats) | `DataClient::get_user_stats` |
| [x] | `GET` | `/v2/user-volume` | [local](docs/api-reference/wallet/get-a-users-trading-volume.md) / [online](https://docs.polymarket.com/api-reference/wallet/get-a-users-trading-volume) | `DataClient::get_user_volume` |
| [x] | `GET` | `/v2/value` | [local](docs/api-reference/wallet/get-portfolio-value.md) / [online](https://docs.polymarket.com/api-reference/wallet/get-portfolio-value) | `DataClient::get_portfolio_value` |

## Relayer API (public endpoints)

Base URL `https://relayer-v2.polymarket.com`, spec [`docs/specs/relayer-openapi.yaml`](docs/specs/relayer-openapi.yaml), feature `relayer`, module `polyoxide::relayer`.

| Status | Method | Path | Docs | Rust |
|---|---|---|---|---|
| [x] | `GET` | `/transaction` | [local](docs/api-reference/relayer/get-a-transaction-by-id.md) / [online](https://docs.polymarket.com/api-reference/relayer/get-a-transaction-by-id) | `RelayerClient::get_transaction` |
| [x] | `GET` | `/nonce` | [local](docs/api-reference/relayer/get-current-nonce-for-a-user.md) / [online](https://docs.polymarket.com/api-reference/relayer/get-current-nonce-for-a-user) | `RelayerClient::get_nonce` |
| [x] | `GET` | `/relay-payload` | [local](docs/api-reference/relayer/get-relayer-address-and-nonce.md) / [online](https://docs.polymarket.com/api-reference/relayer/get-relayer-address-and-nonce) | `RelayerClient::get_relay_payload` |
| [x] | `GET` | `/deployed` | [local](docs/api-reference/relayer/check-if-a-wallet-is-deployed.md) / [online](https://docs.polymarket.com/api-reference/relayer/check-if-a-wallet-is-deployed) | `RelayerClient::check_deployed` |

## Bridge API

Base URL `https://bridge.polymarket.com`, spec [`docs/specs/bridge-openapi.yaml`](docs/specs/bridge-openapi.yaml), feature `bridge`, module `polyoxide::bridge`.

| Status | Method | Path | Docs | Rust |
|---|---|---|---|---|
| [x] | `GET` | `/supported-assets` | [local](docs/api-reference/bridge/get-supported-assets.md) / [online](https://docs.polymarket.com/api-reference/bridge/get-supported-assets) | `BridgeClient::get_supported_assets` |
| [x] | `POST` | `/quote` | [local](docs/api-reference/bridge/get-a-quote.md) / [online](https://docs.polymarket.com/api-reference/bridge/get-a-quote) | `BridgeClient::get_quote` |
| [x] | `POST` | `/deposit` | [local](docs/api-reference/bridge/create-bridge-addresses.md) / [online](https://docs.polymarket.com/api-reference/bridge/create-bridge-addresses) | `BridgeClient::create_deposit_addresses` |
| [x] | `POST` | `/withdraw` | [local](docs/api-reference/bridge/create-withdrawal-addresses.md) / [online](https://docs.polymarket.com/api-reference/bridge/create-withdrawal-addresses) | `BridgeClient::create_withdrawal_addresses` |
| [x] | `GET` | `/status/{address}` | [local](docs/api-reference/bridge/get-transaction-status.md) / [online](https://docs.polymarket.com/api-reference/bridge/get-transaction-status) | `BridgeClient::list_transactions` |

## Combos / RFQ REST (public endpoints)

Base URL `https://combos-rfq-api.polymarket.com`, spec [`docs/specs/combos-rfq-openapi.yaml`](docs/specs/combos-rfq-openapi.yaml), feature `combos`, module `polyoxide::combos`.

| Status | Method | Path | Docs | Rust |
|---|---|---|---|---|
| [x] | `GET` | `/v1/rfq/combo-markets` | [local](docs/api-reference/combo-markets/get-combo-markets.md) / [online](https://docs.polymarket.com/api-reference/combo-markets/get-combo-markets) | `CombosClient::list_combo_markets` |

## WebSocket channels (public)

Feature `ws`, module `polyoxide::ws`. Each channel type is a `Stream` of typed events; URLs are
defaults (override with `*ChannelBuilder::url`).

| Status | Channel | URL | Docs | Rust |
|---|---|---|---|---|
| [x] | Market channel | `wss://ws-subscriptions-clob.polymarket.com/ws/market` | [local](docs/api-reference/wss/market.md) / [online](https://docs.polymarket.com/api-reference/wss/market), spec `docs/specs/asyncapi.json` | `MarketChannel::connect(MarketSubscription)` → `Stream<Item = Result<MarketEvent>>`; `subscribe` / `unsubscribe` / `update_subscription(MarketSubscriptionUpdate)`, also on the cloneable `MarketChannelHandle` from `handle()` |
| [x] | Sports channel | `wss://sports-api.polymarket.com/ws` | [local](docs/api-reference/wss/sports.md) / [online](https://docs.polymarket.com/api-reference/wss/sports), spec `docs/specs/asyncapi-sports.json` | `SportsChannel::connect()` → `Stream<Item = Result<SportsEvent>>` |
| [x] | PolyBolt `price.polymarket` (public channel only) | `wss://ws-live-v2.polymarket.com/ws` | [local](docs/api-reference/wss/polybolt.md) / [online](https://docs.polymarket.com/api-reference/wss/polybolt), spec `docs/specs/polybolt-asyncapi.json` | `PolyBoltChannel::connect()` → `Stream<Item = Result<PolyBoltEvent>>`; `subscribe` / `unsubscribe(PolyBoltSubscription::price_polymarket(..))`, `ping`, also on the cloneable `PolyBoltChannelHandle` from `handle()` |

## Out of scope (requires authentication)

Not implemented yet. Listed so coverage gaps are explicit.

| Service | Method | Path | Docs |
|---|---|---|---|
| CLOB API | `POST` | `/order` | [local](docs/api-reference/trade/post-a-new-order.md) / [online](https://docs.polymarket.com/api-reference/trade/post-a-new-order) |
| CLOB API | `DELETE` | `/order` | [local](docs/api-reference/trade/cancel-single-order.md) / [online](https://docs.polymarket.com/api-reference/trade/cancel-single-order) |
| CLOB API | `POST` | `/orders` | [local](docs/api-reference/trade/post-multiple-orders.md) / [online](https://docs.polymarket.com/api-reference/trade/post-multiple-orders) |
| CLOB API | `DELETE` | `/orders` | [local](docs/api-reference/trade/cancel-multiple-orders.md) / [online](https://docs.polymarket.com/api-reference/trade/cancel-multiple-orders) |
| CLOB API | `GET` | `/data/orders` | [local](docs/api-reference/trade/get-user-orders.md) / [online](https://docs.polymarket.com/api-reference/trade/get-user-orders) |
| CLOB API | `GET` | `/data/order/{orderID}` | [local](docs/api-reference/trade/get-single-order-by-id.md) / [online](https://docs.polymarket.com/api-reference/trade/get-single-order-by-id) |
| CLOB API | `DELETE` | `/cancel-all` | [local](docs/api-reference/trade/cancel-all-orders.md) / [online](https://docs.polymarket.com/api-reference/trade/cancel-all-orders) |
| CLOB API | `DELETE` | `/cancel-market-orders` | [local](docs/api-reference/trade/cancel-orders-for-a-market.md) / [online](https://docs.polymarket.com/api-reference/trade/cancel-orders-for-a-market) |
| CLOB API | `POST` | `/auth/api-key` | spec only (`createApiKey`) |
| CLOB API | `DELETE` | `/auth/api-key` | spec only (`deleteApiKey`) |
| CLOB API | `GET` | `/auth/api-keys` | spec only (`getApiKeys`) |
| CLOB API | `GET` | `/auth/derive-api-key` | spec only (`deriveApiKey`) |
| CLOB API | `GET` | `/balance-allowance` | spec only (`getBalanceAllowance`) |
| CLOB API | `PUT` | `/balance-allowance` | spec only (`updateBalanceAllowance`) |
| CLOB API | `GET` | `/balance-allowance/update` | spec only (`getUpdateBalanceAllowance`) |
| CLOB API | `GET` | `/auth/ban-status/closed-only` | spec only (`getClosedOnlyMode`) |
| CLOB API | `GET` | `/auth/builder-api-key` | spec only (`getBuilderApiKeys`) |
| CLOB API | `POST` | `/auth/builder-api-key` | spec only (`createBuilderApiKey`) |
| CLOB API | `DELETE` | `/auth/builder-api-key` | spec only (`revokeBuilderApiKey`) |
| CLOB API | `GET` | `/notifications` | spec only (`getNotifications`) |
| CLOB API | `DELETE` | `/notifications` | spec only (`dropNotifications`) |
| CLOB API | `GET` | `/rewards/user` | [local](docs/api-reference/rewards/get-earnings-for-user-by-date.md) / [online](https://docs.polymarket.com/api-reference/rewards/get-earnings-for-user-by-date) |
| CLOB API | `GET` | `/rewards/user/total` | [local](docs/api-reference/rewards/get-total-earnings-for-user-by-date.md) / [online](https://docs.polymarket.com/api-reference/rewards/get-total-earnings-for-user-by-date) |
| CLOB API | `GET` | `/rewards/user/percentages` | [local](docs/api-reference/rewards/get-reward-percentages-for-user.md) / [online](https://docs.polymarket.com/api-reference/rewards/get-reward-percentages-for-user) |
| CLOB API | `GET` | `/rewards/user/markets` | [local](docs/api-reference/rewards/get-user-earnings-and-markets-configuration.md) / [online](https://docs.polymarket.com/api-reference/rewards/get-user-earnings-and-markets-configuration) |
| CLOB API | `POST` | `/heartbeats` | [local](docs/api-reference/trade/send-heartbeat.md) / [online](https://docs.polymarket.com/api-reference/trade/send-heartbeat) |
| CLOB API | `POST` | `/v1/heartbeats` | spec only (`sendHeartbeatV1`) |
| CLOB API | `GET` | `/order-scoring` | [local](docs/api-reference/trade/get-order-scoring-status.md) / [online](https://docs.polymarket.com/api-reference/trade/get-order-scoring-status) |
| CLOB API | `GET` | `/orders-scoring` | spec only (`getOrdersScoring`) |
| CLOB API | `POST` | `/orders-scoring` | spec only (`postOrdersScoring`) |
| CLOB API | `GET` | `/data/trades` | [local](docs/api-reference/trade/get-trades.md) / [online](https://docs.polymarket.com/api-reference/trade/get-trades) |
| Relayer API | `POST` | `/submit` | [local](docs/api-reference/relayer/submit-a-transaction.md) / [online](https://docs.polymarket.com/api-reference/relayer/submit-a-transaction) |
| Relayer API | `GET` | `/transactions` | [local](docs/api-reference/relayer/get-recent-transactions-for-a-user.md) / [online](https://docs.polymarket.com/api-reference/relayer/get-recent-transactions-for-a-user) |
| Relayer API | `GET` | `/relayer/api/keys` | [local](docs/api-reference/relayer-api-keys/get-all-relayer-api-keys.md) / [online](https://docs.polymarket.com/api-reference/relayer-api-keys/get-all-relayer-api-keys) |
| Combos / RFQ REST | `POST` | `/v1/maker/quotes` | [local](docs/api-reference/maker/submit-a-quote.md) / [online](https://docs.polymarket.com/api-reference/maker/submit-a-quote) |
| Combos / RFQ REST | `POST` | `/v1/maker/quotes/cancel` | [local](docs/api-reference/maker/cancel-a-quote.md) / [online](https://docs.polymarket.com/api-reference/maker/cancel-a-quote) |
| Combos / RFQ REST | `POST` | `/v1/maker/confirmations` | [local](docs/api-reference/maker/confirm-or-decline-last-look.md) / [online](https://docs.polymarket.com/api-reference/maker/confirm-or-decline-last-look) |
| WebSocket | - | User channel `wss://ws-subscriptions-clob.polymarket.com/ws/user` | [local](docs/api-reference/wss/user.md) |
| WebSocket | - | RFQ Quoter Gateway `wss://combos-rfq-gateway-quoter.polymarket.com/ws/rfq` | [local](docs/api-reference/wss/rfq.md) |
| WebSocket | - | PolyBolt gated channels (`price.crypto`, `price.equity`, `price.crypto.twap`, `price.equity.twap`) | [local](docs/api-reference/wss/polybolt.md) |
