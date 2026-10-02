//! Responses captured from the live API (`fixtures/live/*.json`).
//!
//! Each fixture file wraps the captured body: `_captured` records the route and capture date,
//! `body` is the response (trimmed: arrays shortened, long text cut). They pin the formats the
//! docs do not show: JSON-encoded lists inside strings, the `optimized` search shape,
//! space-separated timestamps, and every undocumented key (`SPEC_DEVIATIONS.md`).

use std::collections::BTreeSet;

use marcasite::{
    Decimal,
    gamma::{
        Comment, CommentId, CommentParentEntityType, Event, Market, Profile, PublicProfile,
        SearchResults, SeriesSummary, SportsMetadata, Tag, Team,
    },
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use super::live_fixture;

/// Collects the paths of non-empty keys present in `raw` but missing from `back` (the
/// re-serialized model), the offline twin of the live tests' drift report.
fn unmodelled(raw: &Value, back: &Value, path: &str, out: &mut BTreeSet<String>) {
    match (raw, back) {
        (Value::Object(raw), Value::Object(back)) => {
            for (key, raw_value) in raw {
                let empty = match raw_value {
                    Value::Null => true,
                    Value::String(s) => s.is_empty(),
                    Value::Array(a) => a.is_empty(),
                    Value::Object(o) => o.is_empty(),
                    _ => false,
                };
                if empty || key == "$schema" {
                    continue;
                }
                let child = format!("{path}.{key}");
                match back.get(key) {
                    Some(back_value) => unmodelled(raw_value, back_value, &child, out),
                    None => {
                        out.insert(child);
                    }
                }
            }
        }
        (Value::Array(raw), Value::Array(back)) => {
            for (raw_item, back_item) in raw.iter().zip(back) {
                unmodelled(raw_item, back_item, &format!("{path}[]"), out);
            }
        }
        _ => {}
    }
}

/// Decodes `name`, checks that no non-empty key of the capture was dropped and that decoding
/// the re-serialized model gives the same model, and returns it.
fn decode<T: DeserializeOwned + Serialize + PartialEq + std::fmt::Debug>(name: &str) -> T {
    let raw = live_fixture(name);
    let model: T = serde_json::from_value(raw.clone())
        .unwrap_or_else(|e| panic!("live fixture {name} does not decode: {e}"));
    let back = serde_json::to_value(&model).unwrap();
    let mut dropped = BTreeSet::new();
    unmodelled(&raw, &back, "$", &mut dropped);
    assert!(
        dropped.is_empty(),
        "live fixture {name}: keys the model drops: {dropped:?}"
    );
    let again: T = serde_json::from_value(back).unwrap();
    assert_eq!(again, model, "live fixture {name} does not round-trip");
    model
}

fn strings(items: &[&str]) -> Option<Vec<String>> {
    Some(items.iter().map(|s| (*s).to_owned()).collect())
}

#[test]
fn market_decodes_json_encoded_lists_and_undocumented_keys() {
    let market: Market = decode("Market");
    // JSON-encoded strings on this route decode to typed lists.
    assert_eq!(market.outcomes, strings(&["Yes", "No"]));
    assert_eq!(
        market.outcome_prices,
        Some(vec![Decimal::new(745, 3), Decimal::new(255, 3)])
    );
    let tokens = market.clob_token_ids.as_ref().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(tokens[0].as_str().starts_with("1110619025"));
    assert_eq!(market.uma_resolution_statuses, Some(Vec::new()));
    // Undocumented keys.
    assert_eq!(market.neg_risk, Some(true));
    assert!(
        market
            .neg_risk_market_id
            .as_deref()
            .unwrap()
            .starts_with("0x")
    );
    assert_eq!(market.fee_type.as_deref(), Some("economics_fees"));
    assert_eq!(market.combo_status.as_deref(), Some("disabled"));
    assert_eq!(market.approved, Some(true));
    assert_eq!(market.version.as_deref(), Some("v1"));
    let reward = &market.clob_rewards.as_ref().unwrap()[0];
    assert_eq!(reward.rewards_daily_rate, Some(Decimal::from(1000)));
    assert_eq!(reward.rewards_amount, Some(Decimal::ZERO));
    assert_eq!(reward.end_date.as_deref(), Some("2500-12-31"));

    // The lists serialize back to the string form the regular routes use.
    let value = serde_json::to_value(&market).unwrap();
    assert_eq!(value["outcomes"], serde_json::json!("[\"Yes\",\"No\"]"));
    assert_eq!(
        value["outcomePrices"],
        serde_json::json!("[\"0.745\",\"0.255\"]")
    );
}

#[test]
fn legacy_amm_market_decodes_amm_keys_and_space_separated_close_time() {
    let market: Market = decode("MarketOld");
    assert_eq!(market.liquidity_amm, Some(Decimal::ZERO));
    assert_eq!(market.volume_24hr_amm, Some(Decimal::ZERO));
    assert_eq!(market.volume_1yr_amm, Some(Decimal::ZERO));
    assert_eq!(market.fpmm_live, Some(true));
    // `Market.closedTime` is kept as sent: `YYYY-MM-DD HH:MM:SS+00`, not RFC 3339.
    assert_eq!(
        market.closed_time.as_deref(),
        Some("2024-02-25 07:01:50+00")
    );
    assert_eq!(
        market.outcome_prices,
        Some(vec![Decimal::ONE, Decimal::ZERO])
    );
}

#[test]
fn closed_sports_market_keeps_space_separated_timestamps_as_sent() {
    let market: Market = decode("MarketClosedSports");
    assert_eq!(
        market.closed_time.as_deref(),
        Some("2026-09-09 14:25:04+00")
    );
    assert_eq!(
        market.game_start_time.as_deref(),
        Some("2026-09-09 10:30:00+00")
    );
    assert_eq!(market.uma_end_date.as_deref(), Some("2026-09-09T14:25:04Z"));
    assert_eq!(
        market.uma_resolution_statuses,
        strings(&["proposed", "proposed"])
    );
    assert_eq!(market.position_ids.as_ref().unwrap().len(), 2);
    assert_eq!(market.fee_type.as_deref(), Some("sports_fees_v3"));
    assert_eq!(market.neg_risk, Some(false));
}

#[test]
fn sports_event_decodes_sport_teams_and_metadata() {
    let event: Event = decode("EventSports");
    assert_eq!(event.game_id, Some(68_046_766));
    assert_eq!(event.parent_event_id, Some(1_118_243));
    assert_eq!(event.version.as_deref(), Some("v1"));
    assert_eq!(event.neg_risk_augmented, Some(false));
    let metadata = event.event_metadata.as_ref().unwrap();
    assert_eq!(metadata["opticOddsFixtureId"], "202610091BC7C6D1");
    let sport = event.sport.as_ref().unwrap();
    assert_eq!(sport.sport.as_deref(), Some("kor2"));
    assert_eq!(sport.name.as_deref(), Some("K League 2"));
    assert_eq!(sport.primary_tag_id, Some(105_258));
    assert!(sport.created_at.is_some());
    assert_eq!(sport.tag_ids().count(), 4);
    let teams = event.teams.as_ref().unwrap();
    assert_eq!(teams.len(), 2);
    assert_eq!(teams[0].color.as_deref(), Some("#053F7D"));
    assert_eq!(teams[0].provider_id, Some(41_261));
    assert_eq!(teams[0].ordering.as_deref(), Some("home"));
    let market = &event.markets.as_ref().unwrap()[0];
    assert!(market.market_metadata.as_ref().unwrap().is_object());
}

#[test]
fn election_event_decodes_country_and_election_type() {
    let event: Event = decode("EventElection");
    assert_eq!(event.country_name.as_deref(), Some("Brazil"));
    assert_eq!(event.election_type.as_deref(), Some("Presidential"));
    assert_eq!(event.cumulative_markets, Some(false));
    assert_eq!(event.neg_risk_augmented, Some(true));
}

#[test]
fn optimized_search_decodes_arrays_and_has_more() {
    let results: SearchResults = decode("SearchOptimized");
    assert_eq!(results.has_more, Some(true));
    assert_eq!(results.pagination, None);
    let event = &results.events.as_ref().unwrap()[0];
    assert_eq!(event.ended, Some(false));
    let market = &event.markets.as_ref().unwrap()[0];
    // Real JSON arrays here, JSON-encoded strings everywhere else; same typed lists.
    assert_eq!(market.outcomes, strings(&["Yes", "No"]));
    assert_eq!(
        market.outcome_prices,
        Some(vec![Decimal::new(105, 3), Decimal::new(895, 3)])
    );
    assert_eq!(market.clob_token_ids, None);
    assert_eq!(market.best_ask, Some(Decimal::new(12, 2)));
}

#[test]
fn regular_search_decodes_pagination_and_string_lists() {
    let results: SearchResults = decode("Search");
    assert_eq!(results.has_more, None);
    assert_eq!(results.pagination.as_ref().unwrap().has_more, Some(true));
    let market = &results.events.as_ref().unwrap()[0]
        .markets
        .as_ref()
        .unwrap()[0];
    assert_eq!(market.outcomes, strings(&["Yes", "No"]));
    assert_eq!(market.clob_token_ids.as_ref().unwrap().len(), 2);
}

#[test]
fn comment_decodes_media_and_parent_entity_type() {
    let comment: Comment = decode("Comment");
    assert_eq!(
        comment.parent_entity_type,
        Some(CommentParentEntityType::Event)
    );
    let media = &comment.media.as_ref().unwrap()[0];
    assert_eq!(media.provider.as_deref(), Some("giphy"));
    assert_eq!(media.media_type.as_deref(), Some("gif"));
    assert_eq!(media.comment_id, Some(CommentId::from("3279002")));
    assert!(media.created_at.is_some());
    // `commentID` is an integer on the wire and stays one.
    let value = serde_json::to_value(media).unwrap();
    assert_eq!(value["commentID"], serde_json::json!(3_279_002));

    let perps: Comment = decode("CommentPerpsAsset");
    let kind = perps.parent_entity_type.unwrap();
    assert_eq!(kind, CommentParentEntityType::PerpsAsset);
    assert!(!kind.is_unknown());
}

#[test]
fn profiles_decode_taker_tier_and_weighted_volume() {
    let public: PublicProfile = decode("PublicProfile");
    assert_eq!(public.taker_tier, Some(6));
    assert_eq!(public.taker_tier_name.as_deref(), Some("Obsidian"));
    assert_eq!(
        public.weighted_volume,
        Some(Decimal::new(4_927_874_644_158, 6))
    );
    assert_eq!(public.users.as_ref().unwrap()[0].community_mod, Some(false));
    let profile: Profile = decode("Profile");
    assert_eq!(profile.taker_tier, Some(6));
    assert_eq!(profile.weighted_volume, public.weighted_volume);
}

#[test]
fn series_summary_decodes_volumes() {
    let summary: SeriesSummary = decode("SeriesSummary");
    assert_eq!(summary.volume, Some(Decimal::new(1_627_744_943_554, 6)));
    assert!(summary.volume_24hr.is_some());
}

#[test]
fn sports_metadata_team_and_related_tag_decode_extra_keys() {
    let sport: SportsMetadata = decode("SportsMetadata");
    assert_eq!(sport.id, Some(630));
    assert_eq!(sport.name.as_deref(), Some("UFL"));
    assert_eq!(sport.primary_tag_id, Some(105_925));
    let team: Team = decode("Team");
    assert_eq!(team.color.as_deref(), Some("#CF9275"));
    assert_eq!(team.provider_id, Some(140_005_569));
    let tag: Tag = decode("RelatedTagWithCount");
    assert_eq!(tag.active_events_count, Some(330));
}
