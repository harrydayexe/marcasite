//! Every documented field of every Gamma response schema is modelled under its exact wire
//! name and type: each spec-generated fixture deserializes and serializes back unchanged.

use polyoxide::gamma::{
    Comment, CommentCount, Event, EventCreator, EventTweetCount, EventsKeysetPage, EventsPage,
    Market, MarketDescription, MarketsKeysetPage, Profile, PublicProfile, RelatedTag,
    SearchResults, Series, SeriesSummary, SportsMarketTypes, SportsMetadata, Tag, Team,
};
use serde::{Serialize, de::DeserializeOwned};

use super::{fixture, strip_nulls};

fn assert_roundtrip<T: DeserializeOwned + Serialize>(schema: &str) {
    let wire = fixture(schema);
    let parsed: T = serde_json::from_value(wire.clone())
        .unwrap_or_else(|e| panic!("{schema} does not deserialize: {e}"));
    let back = strip_nulls(serde_json::to_value(&parsed).unwrap());
    assert_eq!(back, wire, "{schema} does not round-trip");
}

#[test]
fn market() {
    assert_roundtrip::<Market>("Market");
}

#[test]
fn event() {
    assert_roundtrip::<Event>("Event");
}

#[test]
fn series() {
    assert_roundtrip::<Series>("Series");
}

#[test]
fn series_summary() {
    assert_roundtrip::<SeriesSummary>("SeriesSummary");
}

#[test]
fn team() {
    assert_roundtrip::<Team>("Team");
}

#[test]
fn tag() {
    assert_roundtrip::<Tag>("Tag");
}

#[test]
fn related_tag() {
    assert_roundtrip::<RelatedTag>("RelatedTag");
}

#[test]
fn event_creator() {
    assert_roundtrip::<EventCreator>("EventCreator");
}

#[test]
fn comment() {
    assert_roundtrip::<Comment>("Comment");
}

#[test]
fn profile() {
    assert_roundtrip::<Profile>("Profile");
}

#[test]
fn public_profile() {
    assert_roundtrip::<PublicProfile>("PublicProfileResponse");
}

#[test]
fn sports_metadata() {
    assert_roundtrip::<SportsMetadata>("SportsMetadata");
}

#[test]
fn sports_market_types() {
    assert_roundtrip::<SportsMarketTypes>("SportsMarketTypesResponse");
}

#[test]
fn search_results() {
    assert_roundtrip::<SearchResults>("Search");
}

#[test]
fn events_page() {
    assert_roundtrip::<EventsPage>("EventsPagination");
}

#[test]
fn markets_keyset_page() {
    assert_roundtrip::<MarketsKeysetPage>("KeysetMarketsResponse");
}

#[test]
fn events_keyset_page() {
    assert_roundtrip::<EventsKeysetPage>("KeysetEventsResponse");
}

#[test]
fn market_description() {
    assert_roundtrip::<MarketDescription>("MarketDescription");
}

#[test]
fn comment_count() {
    assert_roundtrip::<CommentCount>("Count");
}

#[test]
fn event_tweet_count() {
    assert_roundtrip::<EventTweetCount>("EventTweetCount");
}

#[test]
fn every_field_is_optional() {
    let _: Market = serde_json::from_str("{}").unwrap();
    let _: Event = serde_json::from_str("{}").unwrap();
    let _: Series = serde_json::from_str("{}").unwrap();
    let _: Comment = serde_json::from_str("{}").unwrap();
    let _: Profile = serde_json::from_str("{}").unwrap();
    let _: PublicProfile = serde_json::from_str("{}").unwrap();
    let _: SearchResults = serde_json::from_str("{}").unwrap();
    let _: Team = serde_json::from_str("{}").unwrap();
    let _: RelatedTag = serde_json::from_str("{}").unwrap();
}

#[test]
fn explicit_nulls_are_accepted() {
    let market: Market = serde_json::from_str(
        r#"{"id":null,"endDate":null,"liquidity":null,"volumeNum":null,"events":null,"feeSchedule":null}"#,
    )
    .unwrap();
    assert_eq!(market.id, None);
    assert_eq!(market.end_date, None);
    assert_eq!(market.liquidity, None);
    assert_eq!(market.volume_num, None);
}
