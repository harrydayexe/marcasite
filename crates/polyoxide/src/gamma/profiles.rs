//! Profiles: `/public-profile` and `/profiles/user_address/{user_address}`.

use chrono::{DateTime, Utc};
use polyoxide_core::{Query, Result, serde_util, types::Address, validate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{GammaClient, ImageOptimization};

/// A public profile (`components/schemas/PublicProfileResponse`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PublicProfile {
    /// When the profile was created.
    #[serde(default, with = "serde_util::datetime_option")]
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
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub weighted_volume: Option<Decimal>,
}

/// A user associated with a public profile (`components/schemas/PublicProfileUser`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PublicProfileUser {
    /// User id.
    pub id: Option<String>,
    /// Whether the user is a creator.
    pub creator: Option<bool>,
    /// Whether the user is a moderator (wire name `mod`).
    #[serde(rename = "mod")]
    pub is_mod: Option<bool>,
    /// Whether the user is a community moderator (undocumented; observed live).
    pub community_mod: Option<bool>,
}

/// A user profile (`components/schemas/Profile`), returned by [`GammaClient::get_profile`]
/// and [`GammaClient::search`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
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
    #[serde(default, with = "serde_util::datetime_option")]
    pub cert_req_date: Option<DateTime<Utc>>,
    /// Taker fee tier number, `0` upwards (undocumented; observed live).
    pub taker_tier: Option<i64>,
    /// Taker fee tier name, e.g. `Tier 0`, `Silver`, `Obsidian` (undocumented; observed live,
    /// kept as sent).
    pub taker_tier_name: Option<String>,
    /// Weighted volume used for the taker tier (undocumented; observed live as a JSON
    /// number).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub weighted_volume: Option<Decimal>,
}

impl GammaClient {
    /// Gets the public profile of a wallet address (proxy wallet or user address).
    ///
    /// See <https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let profile = gamma
    ///     .get_public_profile("0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b")
    ///     .await?;
    /// println!("{:?}", profile.name);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if `address` does not match the
    ///   documented pattern `^0x[a-fA-F0-9]{40}$` (checked before sending);
    /// - [`Error::Api`](crate::Error::Api) with status `400` (`type` `"validation error"`)
    ///   for an address the server rejects, or `404` (`type` `"not found error"`) if no
    ///   profile exists;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn get_public_profile(&self, address: impl Into<Address>) -> Result<PublicProfile> {
        let address = address.into();
        validate::evm_address("address", address.as_str())?;
        let mut query = Query::new();
        query.push("address", &address);
        self.transport
            .get(&["public-profile"])
            .query(query)
            .send()
            .await
    }

    /// Gets the profile of a user address.
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getPublicProfileByUserAddress` (no
    /// published doc page).
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if `user_address` does not match
    ///   the documented pattern `^0x[a-fA-F0-9]{40}$` (checked before sending);
    /// - [`Error::Api`](crate::Error::Api) with status `404` if no profile exists;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn get_profile(&self, user_address: impl Into<Address>) -> Result<Profile> {
        let user_address = user_address.into();
        validate::evm_address("user_address", user_address.as_str())?;
        self.transport
            .get(&["profiles", "user_address", user_address.as_str()])
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names and types from `components/schemas/PublicProfileResponse` and
    /// `PublicProfileUser` in `docs/specs/gamma-openapi.yaml`; the address is the documented
    /// example of the `address` parameter.
    #[test]
    fn deserializes_public_profile() {
        let json = r#"{
            "createdAt": "2024-01-01T00:00:00Z",
            "proxyWallet": "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b",
            "profileImage": "https://example.com/p.png",
            "displayUsernamePublic": true,
            "bio": null,
            "pseudonym": "Some-Pseudonym",
            "name": "alice",
            "users": [{"id": "1", "creator": false, "mod": true}],
            "xUsername": "alice_x",
            "verifiedBadge": false
        }"#;
        let profile: PublicProfile = serde_json::from_str(json).unwrap();
        assert_eq!(
            profile.proxy_wallet,
            Some(Address::from("0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b"))
        );
        assert_eq!(profile.bio, None);
        let user = &profile.users.as_ref().unwrap()[0];
        assert_eq!(user.is_mod, Some(true));
        assert_eq!(user.creator, Some(false));
        assert_eq!(profile.x_username.as_deref(), Some("alice_x"));
        let value = serde_json::to_value(user).unwrap();
        assert_eq!(value["mod"], serde_json::json!(true));
    }

    #[test]
    fn deserializes_profile() {
        let profile: Profile = serde_json::from_str(
            r#"{"id":"5","user":12,"walletActivated":true,"isCertReq":false,"certReqDate":null}"#,
        )
        .unwrap();
        assert_eq!(profile.user, Some(12));
        assert_eq!(profile.wallet_activated, Some(true));
        assert_eq!(profile.cert_req_date, None);
    }
}
