//! Comments: `/comments`, `/comments/{id}` and `/comments/user_address/{user_address}`.

use chrono::{DateTime, Utc};
use futures_core::Stream;
use polyoxide_core::{
    Query, Result,
    pagination::offset_stream,
    serde_util,
    types::{Address, TokenId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    GammaClient, ImageOptimization,
    util::{PageParams, setters},
};

polyoxide_core::string_id! {
    /// A Gamma comment id (sent as a string in responses; an integer in paths).
    pub struct CommentId;
}

polyoxide_core::string_enum! {
    /// The type of entity a comment is attached to (the `parent_entity_type` filter of
    /// [`GammaClient::list_comments`]).
    pub enum CommentParentEntityType {
        /// An event.
        Event => "Event",
        /// A series.
        Series => "Series",
        /// A market.
        Market => "market",
    }
}

/// A comment (`components/schemas/Comment`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Comment {
    /// Comment id.
    pub id: Option<CommentId>,
    /// Comment text.
    pub body: Option<String>,
    /// Type of the entity the comment is attached to. The spec types this as a plain
    /// string (the `parent_entity_type` filter documents `Event`, `Series` and `market`).
    pub parent_entity_type: Option<String>,
    /// Id of the entity the comment is attached to (wire name `parentEntityID`).
    #[serde(rename = "parentEntityID")]
    pub parent_entity_id: Option<i64>,
    /// Id of the comment this one replies to (wire name `parentCommentID`).
    #[serde(rename = "parentCommentID")]
    pub parent_comment_id: Option<CommentId>,
    /// Author's address.
    pub user_address: Option<Address>,
    /// Address being replied to.
    pub reply_address: Option<Address>,
    /// Creation time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub updated_at: Option<DateTime<Utc>>,
    /// Author's profile.
    pub profile: Option<CommentProfile>,
    /// Reactions to the comment.
    pub reactions: Option<Vec<Reaction>>,
    /// Number of reports.
    pub report_count: Option<i64>,
    /// Number of reactions.
    pub reaction_count: Option<i64>,
}

/// The profile of a comment's author (`components/schemas/CommentProfile`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    /// The author's positions (included with `get_positions=true`).
    pub positions: Option<Vec<CommentPosition>>,
}

/// A position held by a comment's author (`components/schemas/CommentPosition`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CommentPosition {
    /// Outcome token id.
    pub token_id: Option<TokenId>,
    /// Position size (a string on the wire).
    pub position_size: Option<Decimal>,
}

/// A reaction to a comment (`components/schemas/Reaction`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Reaction {
    /// Reaction id.
    pub id: Option<String>,
    /// Id of the comment reacted to (wire name `commentID`, an integer).
    #[serde(
        rename = "commentID",
        default,
        with = "crate::gamma::util::integer_id_option"
    )]
    pub comment_id: Option<CommentId>,
    /// Reaction type.
    pub reaction_type: Option<String>,
    /// Icon.
    pub icon: Option<String>,
    /// Address of the user who reacted.
    pub user_address: Option<Address>,
    /// Creation time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Profile of the user who reacted.
    pub profile: Option<CommentProfile>,
}

/// A comment count (`components/schemas/Count`), returned by
/// [`GammaClient::get_event_comment_count`] and [`GammaClient::get_series_comment_count`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CommentCount {
    /// Number of comments.
    pub count: Option<i64>,
}

impl GammaClient {
    /// Lists comments (offset pagination), typically on one event, series or market.
    ///
    /// See <https://docs.polymarket.com/api-reference/comments/list-comments>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use polyoxide::gamma::CommentParentEntityType;
    ///
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let comments = gamma
    ///     .list_comments()
    ///     .parent_entity_type(CommentParentEntityType::Event)
    ///     .parent_entity_id(16167)
    ///     .limit(20)
    ///     .send()
    ///     .await?;
    /// # let _ = comments;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_comments(&self) -> ListComments {
        ListComments {
            client: self.clone(),
            params: ListCommentsParams::default(),
        }
    }

    /// Gets the comments of a comment thread by comment id.
    ///
    /// See <https://docs.polymarket.com/api-reference/comments/get-comments-by-comment-id>.
    pub fn get_comments_by_id(&self, id: impl Into<CommentId>) -> GetCommentsById {
        GetCommentsById {
            client: self.clone(),
            id: id.into(),
            params: GetCommentsByIdParams::default(),
        }
    }

    /// Lists the comments written by a user (offset pagination).
    ///
    /// See <https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address>.
    pub fn list_comments_by_user(&self, user_address: impl Into<Address>) -> ListCommentsByUser {
        ListCommentsByUser {
            client: self.clone(),
            user_address: user_address.into(),
            params: PageParams::default(),
        }
    }
}

/// Request builder for [`GammaClient::list_comments`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListComments {
    client: GammaClient,
    params: ListCommentsParams,
}

#[derive(Debug, Clone, Default)]
struct ListCommentsParams {
    limit: Option<u64>,
    offset: Option<u64>,
    order: Option<String>,
    ascending: Option<bool>,
    parent_entity_type: Option<CommentParentEntityType>,
    parent_entity_id: Option<i64>,
    get_positions: Option<bool>,
    holders_only: Option<bool>,
}

impl ListComments {
    setters! {
        /// Maximum number of comments per page.
        limit: u64;
        /// Number of comments to skip.
        offset: u64;
        /// Comma-separated list of fields to order by.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`).
        ascending: bool;
        /// Only comments on this type of entity.
        parent_entity_type: CommentParentEntityType;
        /// Only comments on the entity with this id.
        parent_entity_id: i64;
        /// Include the authors' positions.
        get_positions: bool;
        /// Only comments by holders.
        holders_only: bool;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Comment>> {
        let p = &self.params;
        let mut q = Query::new();
        q.push_opt("limit", p.limit)
            .push_opt("offset", offset)
            .push_opt("order", p.order.as_deref())
            .push_opt("ascending", p.ascending)
            .push_opt("parent_entity_type", p.parent_entity_type.as_ref())
            .push_opt("parent_entity_id", p.parent_entity_id)
            .push_opt("get_positions", p.get_positions)
            .push_opt("holders_only", p.holders_only);
        self.client
            .transport
            .get(&["comments"])
            .query(q)
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Comment>> {
        self.fetch(self.params.offset).await
    }

    /// Streams every comment from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page or the first page shorter than
    /// [`limit`](Self::limit) (when set).
    pub fn into_stream(self) -> impl Stream<Item = Result<Comment>> + Send + 'static {
        let start = self.params.offset.unwrap_or(0);
        let page_size = self.params.limit;
        offset_stream(start, page_size, move |offset| {
            let request = self.clone();
            async move { request.fetch(Some(offset)).await }
        })
    }
}

/// Request builder for [`GammaClient::get_comments_by_id`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetCommentsById {
    client: GammaClient,
    id: CommentId,
    params: GetCommentsByIdParams,
}

#[derive(Debug, Clone, Default)]
struct GetCommentsByIdParams {
    get_positions: Option<bool>,
}

impl GetCommentsById {
    setters! {
        /// Include the authors' positions.
        get_positions: bool;
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Comment>> {
        let mut query = Query::new();
        query.push_opt("get_positions", self.params.get_positions);
        self.client
            .transport
            .get(&["comments", self.id.as_str()])
            .query(query)
            .send()
            .await
    }
}

/// Request builder for [`GammaClient::list_comments_by_user`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListCommentsByUser {
    client: GammaClient,
    user_address: Address,
    params: PageParams,
}

impl ListCommentsByUser {
    setters! {
        /// Maximum number of comments per page.
        limit: u64;
        /// Number of comments to skip.
        offset: u64;
        /// Comma-separated list of fields to order by.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`).
        ascending: bool;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Comment>> {
        self.client
            .transport
            .get(&["comments", "user_address", self.user_address.as_str()])
            .query(self.params.query(offset))
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Comment>> {
        self.fetch(self.params.offset).await
    }

    /// Streams every comment from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page or the first page shorter than
    /// [`limit`](Self::limit) (when set).
    pub fn into_stream(self) -> impl Stream<Item = Result<Comment>> + Send + 'static {
        let start = self.params.offset.unwrap_or(0);
        let page_size = self.params.limit;
        offset_stream(start, page_size, move |offset| {
            let request = self.clone();
            async move { request.fetch(Some(offset)).await }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names and types from `components/schemas/Comment`, `CommentProfile`,
    /// `CommentPosition` and `Reaction` in `docs/specs/gamma-openapi.yaml` (the docs publish
    /// no example body).
    #[test]
    fn deserializes_comment_wire_names_and_types() {
        let json = r#"{
            "id": "100",
            "body": "Nice",
            "parentEntityType": "Event",
            "parentEntityID": 16167,
            "parentCommentID": "99",
            "userAddress": "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b",
            "createdAt": "2024-01-01T00:00:00Z",
            "profile": {
                "name": "alice",
                "isMod": false,
                "proxyWallet": "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b",
                "positions": [{"tokenId": "123", "positionSize": "10.5"}]
            },
            "reactions": [{"id": "1", "commentID": 100, "reactionType": "HEART"}],
            "reactionCount": 1
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, Some(CommentId::from("100")));
        assert_eq!(comment.parent_entity_id, Some(16_167));
        assert_eq!(comment.parent_comment_id, Some(CommentId::from("99")));
        let profile = comment.profile.as_ref().unwrap();
        assert_eq!(profile.is_mod, Some(false));
        let position = &profile.positions.as_ref().unwrap()[0];
        assert_eq!(position.token_id, Some(TokenId::from("123")));
        assert_eq!(position.position_size, Some(Decimal::new(105, 1)));
        let reaction = &comment.reactions.as_ref().unwrap()[0];
        assert_eq!(reaction.comment_id, Some(CommentId::from("100")));

        // `commentID` is an integer on the wire and stays one when re-serialized.
        let value = serde_json::to_value(reaction).unwrap();
        assert_eq!(value["commentID"], serde_json::json!(100));
    }

    #[test]
    fn parent_entity_type_uses_documented_spelling() {
        assert_eq!(CommentParentEntityType::Market.as_str(), "market");
        assert_eq!(CommentParentEntityType::Event.to_string(), "Event");
        assert_eq!(
            CommentParentEntityType::from("Collection"),
            CommentParentEntityType::Unknown("Collection".to_owned())
        );
    }
}
