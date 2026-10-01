//! Tags: `/tags`, `/tags/{id}`.

use chrono::{DateTime, Utc};
use futures_core::Stream;
use polyoxide_core::{Query, Result, pagination::offset_stream, serde_util};
use serde::{Deserialize, Serialize};

use super::GammaClient;

polyoxide_core::string_id! {
    /// A Gamma tag id (sent as a string in responses, e.g. `"100381"`).
    pub struct TagId;
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
            limit: None,
            offset: None,
            order: None,
            ascending: None,
            include_template: None,
            is_carousel: None,
        }
    }

    /// Gets a tag by id.
    ///
    /// Fails with [`Error::Api`](crate::Error::Api) (status `404`) if the tag does not exist;
    /// see [`Error::is_not_found`](crate::Error::is_not_found).
    ///
    /// See <https://docs.polymarket.com/api-reference/tags/get-tag-by-id>.
    pub fn get_tag(&self, id: impl Into<TagId>) -> GetTag {
        GetTag {
            client: self.clone(),
            id: id.into(),
            include_template: None,
        }
    }
}

/// Request builder for [`GammaClient::list_tags`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListTags {
    client: GammaClient,
    limit: Option<u64>,
    offset: Option<u64>,
    order: Option<String>,
    ascending: Option<bool>,
    include_template: Option<bool>,
    is_carousel: Option<bool>,
}

impl ListTags {
    /// Maximum number of tags per page.
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Number of tags to skip.
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Comma-separated list of fields to order by.
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Sort ascending (`true`) or descending (`false`).
    pub fn ascending(mut self, ascending: bool) -> Self {
        self.ascending = Some(ascending);
        self
    }

    /// Include tag templates.
    pub fn include_template(mut self, include_template: bool) -> Self {
        self.include_template = Some(include_template);
        self
    }

    /// Only carousel tags (`true`) or only non-carousel tags (`false`).
    pub fn is_carousel(mut self, is_carousel: bool) -> Self {
        self.is_carousel = Some(is_carousel);
        self
    }

    fn query(&self, offset: Option<u64>) -> Query {
        let mut q = Query::new();
        q.push_opt("limit", self.limit)
            .push_opt("offset", offset)
            .push_opt("order", self.order.as_deref())
            .push_opt("ascending", self.ascending)
            .push_opt("include_template", self.include_template)
            .push_opt("is_carousel", self.is_carousel);
        q
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Tag>> {
        let query = self.query(self.offset);
        self.client
            .transport
            .get(&["tags"])
            .query(query)
            .send()
            .await
    }

    /// Streams every tag from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page or the first page shorter than
    /// [`limit`](Self::limit) (when set).
    pub fn into_stream(self) -> impl Stream<Item = Result<Tag>> + Send + 'static {
        let start = self.offset.unwrap_or(0);
        let page_size = self.limit;
        offset_stream(start, page_size, move |offset| {
            let request = self.clone();
            async move {
                let query = request.query(Some(offset));
                request
                    .client
                    .transport
                    .get(&["tags"])
                    .query(query)
                    .send()
                    .await
            }
        })
    }
}

/// Request builder for [`GammaClient::get_tag`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetTag {
    client: GammaClient,
    id: TagId,
    include_template: Option<bool>,
}

impl GetTag {
    /// Include the tag template.
    pub fn include_template(mut self, include_template: bool) -> Self {
        self.include_template = Some(include_template);
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing tag is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn send(self) -> Result<Tag> {
        let mut query = Query::new();
        query.push_opt("include_template", self.include_template);
        self.client
            .transport
            .get(&["tags", self.id.as_str()])
            .query(query)
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
}
