//! Models shared by several Gamma topics (`components/schemas` in
//! `docs/specs/gamma-openapi.yaml`).
//!
//! The Gamma spec marks no field as required, so every field is optional.

use chrono::{DateTime, Utc};
use polyoxide_core::serde_util;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Metadata about an optimized copy of an image (`components/schemas/ImageOptimization`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImageOptimization {
    /// Record id.
    pub id: Option<String>,
    /// URL of the source image.
    pub image_url_source: Option<String>,
    /// URL of the optimized image.
    pub image_url_optimized: Option<String>,
    /// Size of the source image in KB.
    #[serde(default, with = "crate::gamma::util::number_option")]
    pub image_size_kb_source: Option<Decimal>,
    /// Size of the optimized image in KB.
    #[serde(default, with = "crate::gamma::util::number_option")]
    pub image_size_kb_optimized: Option<Decimal>,
    /// Whether optimization has completed.
    pub image_optimized_complete: Option<bool>,
    /// When the optimized image was last updated. The spec types this as a plain string
    /// with no format.
    pub image_optimized_last_updated: Option<String>,
    /// Id of the related record (wire name `relID`).
    #[serde(rename = "relID")]
    pub rel_id: Option<i64>,
    /// Name of the field on the related record that holds the image.
    pub field: Option<String>,
    /// Name of the related record type.
    pub relname: Option<String>,
}

/// A category (`components/schemas/Category`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// A collection of events (`components/schemas/Collection`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
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

/// A chat channel attached to an event or series (`components/schemas/Chat`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default, with = "serde_util::datetime_option")]
    pub start_time: Option<DateTime<Utc>>,
    /// End time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub end_time: Option<DateTime<Utc>>,
}

/// An event template (`components/schemas/Template`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Pagination metadata (`components/schemas/Pagination`), returned by
/// [`GammaClient::list_events_paginated`](super::GammaClient::list_events_paginated) and
/// [`GammaClient::search`](super::GammaClient::search).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Pagination {
    /// Whether more results are available.
    pub has_more: Option<bool>,
    /// Total number of results.
    pub total_results: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names and types from `components/schemas/ImageOptimization` in
    /// `docs/specs/gamma-openapi.yaml`.
    #[test]
    fn image_optimization_roundtrips_wire_names() {
        let json = serde_json::json!({
            "id": "1",
            "imageUrlSource": "https://example.com/a.png",
            "imageUrlOptimized": null,
            "imageSizeKbSource": 12.5,
            "imageSizeKbOptimized": 3,
            "imageOptimizedComplete": true,
            "imageOptimizedLastUpdated": "2024-01-01",
            "relID": 42,
            "field": "image",
            "relname": "markets"
        });
        let image: ImageOptimization = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(image.rel_id, Some(42));
        assert_eq!(image.image_size_kb_source, Some(Decimal::new(125, 1)));
        assert_eq!(image.image_size_kb_optimized, Some(Decimal::from(3)));
        assert_eq!(serde_json::to_value(&image).unwrap(), json);
    }

    #[test]
    fn number_fields_accept_numeric_strings() {
        let image: ImageOptimization =
            serde_json::from_str(r#"{"imageSizeKbSource":"0.1"}"#).unwrap();
        assert_eq!(image.image_size_kb_source, Some(Decimal::new(1, 1)));
        assert_eq!(image.id, None);
    }

    #[test]
    fn pagination_decodes() {
        let page: Pagination =
            serde_json::from_str(r#"{"hasMore":true,"totalResults":120}"#).unwrap();
        assert_eq!(page.has_more, Some(true));
        assert_eq!(page.total_results, Some(120));
    }
}
