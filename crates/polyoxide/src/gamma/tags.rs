//! Tags: `/tags`, `/tags/{id}`, `/tags/slug/{slug}` and the related-tags endpoints.

use crate::Paginated;
use chrono::{DateTime, Utc};
use polyoxide_core::{Query, Result, pagination::offset_stream, serde_util};
use serde::{Deserialize, Serialize};

use super::{
    GammaClient,
    util::{Lookup, setters},
};

polyoxide_core::string_id! {
    /// A Gamma tag id (sent as a string in responses, e.g. `"100381"`).
    pub struct TagId;
}

polyoxide_core::string_enum! {
    /// The `status` filter of the related-tags endpoints (the spec documents the values,
    /// not their meaning).
    pub enum RelatedTagsStatus {
        /// The value `active`.
        Active => "active",
        /// The value `closed`.
        Closed => "closed",
        /// The value `all`.
        All => "all",
    }
}

/// A tag used to categorise events, markets and series.
///
/// Every field is optional because the spec (`components/schemas/Tag`) marks none as
/// required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub updated_at: Option<DateTime<Utc>>,
    /// Whether the tag is always hidden.
    pub force_hide: Option<bool>,
    /// Whether the tag is shown in the carousel.
    pub is_carousel: Option<bool>,
}

/// A relationship between two tags (`components/schemas/RelatedTag`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RelatedTag {
    /// Relationship id.
    pub id: Option<String>,
    /// The tag (wire name `tagID`, an integer).
    #[serde(rename = "tagID", default, with = "serde_util::integer_id_option")]
    pub tag_id: Option<TagId>,
    /// The related tag (wire name `relatedTagID`, an integer).
    #[serde(
        rename = "relatedTagID",
        default,
        with = "serde_util::integer_id_option"
    )]
    pub related_tag_id: Option<TagId>,
    /// Rank of the relationship.
    pub rank: Option<i64>,
}

impl GammaClient {
    /// Lists tags (offset pagination).
    ///
    /// See <https://docs.polymarket.com/api-reference/tags/list-tags>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let tags = gamma.list_tags().limit(20).ascending(true).send().await?;
    /// # let _ = tags;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_tags(&self) -> ListTags {
        ListTags {
            client: self.clone(),
            params: ListTagsParams::default(),
        }
    }

    /// Gets a tag by id.
    ///
    /// See <https://docs.polymarket.com/api-reference/tags/get-tag-by-id>.
    pub fn get_tag(&self, id: impl Into<TagId>) -> GetTag {
        GetTag {
            client: self.clone(),
            lookup: Lookup::Id(id.into()),
            params: GetTagParams::default(),
        }
    }

    /// Gets a tag by slug.
    ///
    /// See <https://docs.polymarket.com/api-reference/tags/get-tag-by-slug>.
    pub fn get_tag_by_slug(&self, slug: impl Into<String>) -> GetTag {
        GetTag {
            client: self.clone(),
            lookup: Lookup::Slug(slug.into()),
            params: GetTagParams::default(),
        }
    }

    /// Gets the relationships between a tag (by id) and its related tags.
    ///
    /// See <https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id>.
    pub fn get_related_tag_relationships(
        &self,
        id: impl Into<TagId>,
    ) -> GetRelatedTagRelationships {
        GetRelatedTagRelationships {
            client: self.clone(),
            lookup: Lookup::Id(id.into()),
            params: RelatedTagsParams::default(),
        }
    }

    /// Gets the relationships between a tag (by slug) and its related tags.
    ///
    /// See <https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-slug>.
    pub fn get_related_tag_relationships_by_slug(
        &self,
        slug: impl Into<String>,
    ) -> GetRelatedTagRelationships {
        GetRelatedTagRelationships {
            client: self.clone(),
            lookup: Lookup::Slug(slug.into()),
            params: RelatedTagsParams::default(),
        }
    }

    /// Gets the tags related to a tag (by id).
    ///
    /// See <https://docs.polymarket.com/api-reference/tags/get-tags-related-to-a-tag-id>.
    pub fn get_related_tags(&self, id: impl Into<TagId>) -> GetRelatedTags {
        GetRelatedTags {
            client: self.clone(),
            lookup: Lookup::Id(id.into()),
            params: RelatedTagsParams::default(),
        }
    }

    /// Gets the tags related to a tag (by slug).
    ///
    /// See <https://docs.polymarket.com/api-reference/tags/get-tags-related-to-a-tag-slug>.
    pub fn get_related_tags_by_slug(&self, slug: impl Into<String>) -> GetRelatedTags {
        GetRelatedTags {
            client: self.clone(),
            lookup: Lookup::Slug(slug.into()),
            params: RelatedTagsParams::default(),
        }
    }
}

/// Request builder for [`GammaClient::list_tags`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListTags {
    client: GammaClient,
    params: ListTagsParams,
}

#[derive(Debug, Clone, Default)]
struct ListTagsParams {
    limit: Option<u32>,
    offset: Option<u32>,
    order: Option<String>,
    ascending: Option<bool>,
    include_template: Option<bool>,
    is_carousel: Option<bool>,
}

impl ListTags {
    setters! {
        /// Maximum number of tags per page (`limit`; the docs give a minimum of `0` and no
        /// maximum).
        limit: u32;
        /// Number of tags to skip (`offset`).
        offset: u32;
        /// Comma-separated list of fields to order by (`order`).
        order: into String;
        /// Sort ascending (`true`) or descending (`false`) (`ascending`).
        ascending: bool;
        /// The `include_template` flag (documented only as a boolean).
        include_template: bool;
        /// The `is_carousel` filter (documented only as a boolean).
        is_carousel: bool;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Tag>> {
        let p = &self.params;
        let mut query = Query::new();
        query
            .push_opt("limit", p.limit)
            .push_opt("offset", offset)
            .push_opt("order", p.order.as_deref())
            .push_opt("ascending", p.ascending)
            .push_opt("include_template", p.include_template)
            .push_opt("is_carousel", p.is_carousel);
        self.client
            .transport
            .get(&["tags"])
            .query(query)
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Tag>> {
        self.fetch(self.params.offset.map(u64::from)).await
    }

    /// Streams every tag from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page, or right after yielding the first error.
    /// A page shorter than [`limit`](Self::limit) does not end it, because the docs give no
    /// maximum `limit` and the server may return fewer tags, so the last request returns an
    /// empty page.
    pub fn into_stream(self) -> Paginated<Tag> {
        let start = self.params.offset.map_or(0, u64::from);
        offset_stream(start, move |offset| {
            let request = self.clone();
            async move { request.fetch(Some(offset)).await }
        })
    }
}

/// Request builder for [`GammaClient::get_tag`] and [`GammaClient::get_tag_by_slug`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetTag {
    client: GammaClient,
    lookup: Lookup<TagId>,
    params: GetTagParams,
}

#[derive(Debug, Clone, Default)]
struct GetTagParams {
    include_template: Option<bool>,
}

impl GetTag {
    setters! {
        /// The `include_template` flag (documented only as a boolean).
        include_template: bool;
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation), checked before sending, if the id
    ///   is not an integer (parameter `id`) or the slug is empty, `.` or `..` (parameter
    ///   `slug`);
    /// - [`Error::Api`](crate::Error::Api) with status `404` if the tag does not exist (see
    ///   [`Error::is_not_found`](crate::Error::is_not_found));
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Tag> {
        let segments = self.lookup.path("tags", &[])?;
        let mut query = Query::new();
        query.push_opt("include_template", self.params.include_template);
        self.client
            .transport
            .get(&segments)
            .query(query)
            .send()
            .await
    }
}

/// Parameters shared by the four related-tags endpoints.
#[derive(Debug, Clone, Default)]
struct RelatedTagsParams {
    omit_empty: Option<bool>,
    status: Option<RelatedTagsStatus>,
}

impl RelatedTagsParams {
    fn query(&self) -> Query {
        let mut q = Query::new();
        q.push_opt("omit_empty", self.omit_empty)
            .push_opt("status", self.status.as_ref());
        q
    }
}

/// Request builder for [`GammaClient::get_related_tag_relationships`] and
/// [`GammaClient::get_related_tag_relationships_by_slug`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetRelatedTagRelationships {
    client: GammaClient,
    lookup: Lookup<TagId>,
    params: RelatedTagsParams,
}

impl GetRelatedTagRelationships {
    setters! {
        /// The `omit_empty` flag (documented only as a boolean).
        omit_empty: bool;
        /// The `status` filter.
        status: RelatedTagsStatus;
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation), checked before sending, if the id
    ///   is not an integer (parameter `id`) or the slug is empty, `.` or `..` (parameter
    ///   `slug`);
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<RelatedTag>> {
        let segments = self.lookup.path("tags", &["related-tags"])?;
        self.client
            .transport
            .get(&segments)
            .query(self.params.query())
            .send()
            .await
    }
}

/// Request builder for [`GammaClient::get_related_tags`] and
/// [`GammaClient::get_related_tags_by_slug`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetRelatedTags {
    client: GammaClient,
    lookup: Lookup<TagId>,
    params: RelatedTagsParams,
}

impl GetRelatedTags {
    setters! {
        /// The `omit_empty` flag (documented only as a boolean).
        omit_empty: bool;
        /// The `status` filter.
        status: RelatedTagsStatus;
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation), checked before sending, if the id
    ///   is not an integer (parameter `id`) or the slug is empty, `.` or `..` (parameter
    ///   `slug`);
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Tag>> {
        let segments = self.lookup.path("tags", &["related-tags", "tags"])?;
        self.client
            .transport
            .get(&segments)
            .query(self.params.query())
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names and types from `components/schemas/Tag` in `docs/specs/gamma-openapi.yaml`.
    #[test]
    fn deserializes_tag() {
        let json = r#"{
            "id": "100381",
            "label": "Politics",
            "slug": "politics",
            "forceShow": false,
            "publishedAt": "2023-10-25 20:13:51.871+00",
            "createdBy": 1,
            "updatedBy": null,
            "createdAt": "2023-10-25T20:13:51.886Z",
            "updatedAt": "2024-01-01T00:00:00Z",
            "forceHide": null,
            "isCarousel": true
        }"#;
        let tag: Tag = serde_json::from_str(json).unwrap();
        assert_eq!(tag.id, Some(TagId::from("100381")));
        assert_eq!(tag.label.as_deref(), Some("Politics"));
        assert_eq!(tag.created_by, Some(1));
        assert_eq!(tag.updated_by, None);
        assert_eq!(
            tag.created_at.map(|d| d.timestamp_millis()),
            Some(1_698_264_831_886)
        );
        assert_eq!(tag.is_carousel, Some(true));
    }

    #[test]
    fn deserializes_empty_tag() {
        let tag: Tag = serde_json::from_str("{}").unwrap();
        assert_eq!(tag.id, None);
    }

    /// Field names and types from `components/schemas/RelatedTag` in
    /// `docs/specs/gamma-openapi.yaml`.
    #[test]
    fn related_tag_ids_are_integers_on_the_wire() {
        let json = serde_json::json!({"id": "7", "tagID": 100381, "relatedTagID": 2, "rank": 1});
        let related: RelatedTag = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(related.tag_id, Some(TagId::from("100381")));
        assert_eq!(related.related_tag_id, Some(TagId::from("2")));
        assert_eq!(related.rank, Some(1));
        assert_eq!(serde_json::to_value(&related).unwrap(), json);
    }

    #[test]
    fn related_tags_status_spelling() {
        assert_eq!(RelatedTagsStatus::Active.as_str(), "active");
        assert_eq!(RelatedTagsStatus::All.to_string(), "all");
    }
}
