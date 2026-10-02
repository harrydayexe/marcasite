//! Comments: `/comments`, `/comments/{id}` and `/comments/user_address/{user_address}`.

use crate::Paginated;
use chrono::{DateTime, Utc};
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
    util::{
        MAX_COMMENTS_OFFSET, PageParams, check_integer_id, check_path_text, setters,
        validate_offset,
    },
};

/// Hint appended to the offset-cap error of the comment listings.
const COMMENTS_DEEPER: &str =
    "deeper pages are only available from `/comments/keyset`, which this crate does not wrap";

polyoxide_core::string_id! {
    /// A Gamma comment id (sent as a string in responses; an integer in paths).
    pub struct CommentId;
}

polyoxide_core::string_enum! {
    /// The type of entity a comment is attached to (the `parent_entity_type` of
    /// [`GammaClient::list_comments`] and of [`Comment::parent_entity_type`]).
    ///
    /// The spec lists `Event`, `Series` and `market`; live accepts `Event`, `Series` and
    /// `PerpsAsset` and rejects `market` with a `422` (see `SPEC_DEVIATIONS.md`), so there is
    /// no `Market` variant.
    pub enum CommentParentEntityType {
        /// An event (`parent_entity_id` is an event id).
        Event => "Event",
        /// A series (`parent_entity_id` is a series id).
        Series => "Series",
        /// A perpetuals asset (undocumented; accepted live).
        PerpsAsset => "PerpsAsset",
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
    /// string with no values; live sends `Event`, `Series` or `PerpsAsset`, and any other
    /// value is kept as [`CommentParentEntityType::Unknown`].
    pub parent_entity_type: Option<CommentParentEntityType>,
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
    /// Media attached to the comment, e.g. GIFs (undocumented; observed live; absent on
    /// comments without media).
    pub media: Option<Vec<CommentMedia>>,
}

/// Media attached to a comment (element of [`Comment::media`]).
///
/// Not in the Gamma spec; the shape is as observed live (see `SPEC_DEVIATIONS.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CommentMedia {
    /// Media id.
    pub id: Option<String>,
    /// Id of the comment the media belongs to (wire name `commentID`, an integer).
    #[serde(rename = "commentID", default, with = "serde_util::integer_id_option")]
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
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
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
    /// The author's positions. See the `get_positions` request flag (documented only as a
    /// boolean).
    pub positions: Option<Vec<CommentPosition>>,
}

/// A position held by a comment's author (`components/schemas/CommentPosition`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CommentPosition {
    /// Outcome token id.
    pub token_id: Option<TokenId>,
    /// Position size. The spec types this as a string (`positionSize`); it is parsed as a
    /// decimal (an empty string or `null` becomes `None`, other non-numeric text fails
    /// decoding) and serializes back as a JSON string.
    #[serde(default, with = "serde_util::string_or_number_option")]
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
    #[serde(rename = "commentID", default, with = "serde_util::integer_id_option")]
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
    /// Lists the comments on one entity (offset pagination).
    ///
    /// `parent_entity_type` and `parent_entity_id` are required: live answers a request
    /// without either with a `422`. The spec documents neither as required, and lists
    /// `market` (not `PerpsAsset`) as a type; see `SPEC_DEVIATIONS.md`.
    ///
    /// See <https://docs.polymarket.com/api-reference/comments/list-comments>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use polyoxide::gamma::CommentParentEntityType;
    ///
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let comments = gamma
    ///     .list_comments(CommentParentEntityType::Event, 16167)
    ///     .limit(20)
    ///     .send()
    ///     .await?;
    /// # let _ = comments;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_comments(
        &self,
        parent_entity_type: impl Into<CommentParentEntityType>,
        parent_entity_id: i64,
    ) -> ListComments {
        ListComments {
            client: self.clone(),
            parent_entity_type: parent_entity_type.into(),
            parent_entity_id,
            params: ListCommentsParams::default(),
        }
    }

    /// Gets comments by comment id (`GET /comments/{id}`).
    ///
    /// The docs do not describe which comments the list holds beyond the summary "Get
    /// comments by comment id".
    ///
    /// See <https://docs.polymarket.com/api-reference/comments/get-comments-by-comment-id>.
    pub fn get_comments_by_id(&self, id: impl Into<CommentId>) -> GetCommentsById {
        GetCommentsById {
            client: self.clone(),
            id: id.into(),
            params: GetCommentsByIdParams::default(),
        }
    }

    /// Lists the comments of a user address (offset pagination).
    ///
    /// The spec types `user_address` as a plain string with no pattern, so it is not
    /// checked against the EVM address pattern; it must only be non-empty and not `.` or
    /// `..`.
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
    parent_entity_type: CommentParentEntityType,
    parent_entity_id: i64,
    params: ListCommentsParams,
}

#[derive(Debug, Clone, Default)]
struct ListCommentsParams {
    limit: Option<u32>,
    offset: Option<u32>,
    order: Option<String>,
    ascending: Option<bool>,
    get_positions: Option<bool>,
    holders_only: Option<bool>,
}

impl ListComments {
    setters! {
        /// Maximum number of comments per page (`limit`; the docs give a minimum of `0` and
        /// no maximum).
        limit: u32;
        /// Number of comments to skip (`offset`). Live rejects values above 200 (a `422`;
        /// the listing is meant to be walked with `/comments/keyset`, which this crate does
        /// not wrap), so larger values fail client-side with
        /// [`Error::Validation`](crate::Error::Validation).
        offset: u32;
        /// Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
        /// field names of the response type (e.g. `createdAt` or `reactionCount`); snake_case names
        /// such as `start_date` are rejected with a `422` (`order fields are not valid`), although
        /// the spec's keyset example uses them. See `SPEC_DEVIATIONS.md`.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`) (`ascending`).
        ascending: bool;
        /// The `get_positions` flag (documented only as a boolean).
        get_positions: bool;
        /// The `holders_only` filter (documented only as a boolean).
        holders_only: bool;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Comment>> {
        validate_offset(offset, MAX_COMMENTS_OFFSET, COMMENTS_DEEPER)?;
        let p = &self.params;
        let mut q = Query::new();
        q.push_opt("limit", p.limit)
            .push_opt("offset", offset)
            .push_opt("order", p.order.as_deref())
            .push_opt("ascending", p.ascending)
            .push("parent_entity_type", &self.parent_entity_type)
            .push("parent_entity_id", self.parent_entity_id)
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
    /// - [`Error::Validation`](crate::Error::Validation) (parameter `offset`) if
    ///   [`offset`](Self::offset) exceeds 200, checked before sending;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Comment>> {
        self.fetch(self.params.offset.map(u64::from)).await
    }

    /// Streams every comment from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page, or right after yielding the first error.
    /// A page shorter than [`limit`](Self::limit) does not end it, because the docs give no
    /// maximum `limit` and the server may return fewer comments, so the last request
    /// returns an empty page.
    ///
    /// Live rejects offsets above 200: past that the stream yields one
    /// [`Error::Validation`](crate::Error::Validation) (parameter `offset`, without sending a
    /// request) and ends.
    pub fn into_stream(self) -> Paginated<Comment> {
        let start = self.params.offset.map_or(0, u64::from);
        offset_stream(start, move |offset| {
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
        /// The `get_positions` flag (documented only as a boolean).
        get_positions: bool;
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) (parameter `id`) if the id is not
    ///   an integer (one or more ASCII digits), checked before sending;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Comment>> {
        check_integer_id("id", self.id.as_str())?;
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
        /// Maximum number of comments per page (`limit`; the docs give a minimum of `0` and
        /// no maximum).
        limit: u32;
        /// Number of comments to skip (`offset`). Live rejects values above 200 (a `422`;
        /// the listing is meant to be walked with `/comments/keyset`, which this crate does
        /// not wrap), so larger values fail client-side with
        /// [`Error::Validation`](crate::Error::Validation).
        offset: u32;
        /// Comma-separated list of fields to order by (`order`). Live expects the camelCase JSON
        /// field names of the response type (e.g. `createdAt` or `reactionCount`); snake_case names
        /// such as `start_date` are rejected with a `422` (`order fields are not valid`), although
        /// the spec's keyset example uses them. See `SPEC_DEVIATIONS.md`.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`) (`ascending`).
        ascending: bool;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Comment>> {
        check_path_text("user_address", self.user_address.as_str())?;
        validate_offset(offset, MAX_COMMENTS_OFFSET, COMMENTS_DEEPER)?;
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
    /// - [`Error::Validation`](crate::Error::Validation) (parameter `user_address`) if the
    ///   address is empty, `.` or `..`, or (parameter `offset`) if [`offset`](Self::offset)
    ///   exceeds 200, checked before sending;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Comment>> {
        self.fetch(self.params.offset.map(u64::from)).await
    }

    /// Streams every comment from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page, or right after yielding the first error.
    /// A page shorter than [`limit`](Self::limit) does not end it, because the docs give no
    /// maximum `limit` and the server may return fewer comments, so the last request
    /// returns an empty page.
    ///
    /// Live rejects offsets above 200: past that the stream yields one
    /// [`Error::Validation`](crate::Error::Validation) (parameter `offset`, without sending a
    /// request) and ends.
    pub fn into_stream(self) -> Paginated<Comment> {
        let start = self.params.offset.map_or(0, u64::from);
        offset_stream(start, move |offset| {
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

    /// `positionSize` is string-typed: `""` is "absent", other non-numeric text fails, and
    /// the value serializes back as a string.
    #[test]
    fn position_size_is_a_string_typed_amount() {
        let position: CommentPosition =
            serde_json::from_str(r#"{"tokenId":"1","positionSize":""}"#).unwrap();
        assert_eq!(position.position_size, None);
        let position: CommentPosition = serde_json::from_str(r#"{"positionSize":"10.5"}"#).unwrap();
        assert_eq!(position.position_size, Some(Decimal::new(105, 1)));
        assert_eq!(
            serde_json::to_value(&position).unwrap()["positionSize"],
            serde_json::json!("10.5")
        );
        assert!(serde_json::from_str::<CommentPosition>(r#"{"positionSize":"many"}"#).is_err());
        // An invalid nested amount fails the whole comment.
        let err = serde_json::from_str::<Comment>(
            r#"{"profile":{"positions":[{"positionSize":"many"}]}}"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("many"), "{err}");
        let comment: Comment =
            serde_json::from_str(r#"{"profile":{"positions":[{"positionSize":""}]}}"#).unwrap();
        assert_eq!(
            comment.profile.unwrap().positions.unwrap()[0].position_size,
            None
        );
    }

    #[test]
    fn parent_entity_type_uses_documented_spelling() {
        assert_eq!(CommentParentEntityType::PerpsAsset.as_str(), "PerpsAsset");
        assert_eq!(CommentParentEntityType::Event.to_string(), "Event");
        assert_eq!(
            CommentParentEntityType::from("Collection"),
            CommentParentEntityType::Unknown("Collection".to_owned())
        );
        // The spec's `market` is rejected live, so it is not a known variant.
        assert_eq!(
            CommentParentEntityType::from("market"),
            CommentParentEntityType::Unknown("market".to_owned())
        );
    }
}
