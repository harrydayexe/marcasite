//! `/public-profile` and `/profiles/user_address/{user_address}`.

use polyoxide::{Error, types::Address};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use super::{fixture, json_value, pairs, query_of, requests};
use crate::common;

/// The documented example of the `address` parameter.
const ADDRESS: &str = "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b";

#[tokio::test]
async fn get_public_profile_sends_address_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/public-profile"))
        .respond_with(json_value(&fixture("PublicProfileResponse")))
        .expect(1)
        .mount(&server)
        .await;

    let profile = common::polymarket(&server)
        .gamma()
        .get_public_profile(ADDRESS)
        .await
        .unwrap();
    assert_eq!(query_of(&server, 0).await, pairs(&[("address", ADDRESS)]));
    assert_eq!(profile.proxy_wallet, Some(Address::from(ADDRESS)));
    assert_eq!(
        profile.profile_image.as_deref(),
        Some("https://example.com/profileImage")
    );
    let user = &profile.users.unwrap()[0];
    assert_eq!(user.is_mod, Some(true));
    assert_eq!(profile.verified_badge, Some(true));
}

/// Error bodies use the documented examples of `components/schemas/PublicProfileError`.
#[tokio::test]
async fn get_public_profile_parses_documented_errors() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/public-profile"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            r#"{"type":"not found error","error":"profile not found"}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .gamma()
        .get_public_profile(ADDRESS)
        .await
        .unwrap_err();
    assert!(err.is_not_found());
    let api = err.api_error().unwrap();
    assert_eq!(api.error_type(), Some("not found error"));
    assert_eq!(api.message(), Some("profile not found"));

    server.reset().await;
    Mock::given(method("GET"))
        .and(path("/public-profile"))
        .respond_with(ResponseTemplate::new(400).set_body_raw(
            r#"{"type":"validation error","error":"invalid address"}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    let err = common::polymarket(&server)
        .gamma()
        .get_public_profile(ADDRESS)
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 400);
    assert_eq!(api.error_type(), Some("validation error"));
}

#[tokio::test]
async fn invalid_addresses_are_rejected_before_sending() {
    let server = common::server().await;
    let gamma = common::polymarket(&server).gamma().clone();

    let err = gamma.get_public_profile("0x123").await.unwrap_err();
    let Error::Validation(v) = &err else {
        panic!("expected Error::Validation, got {err:?}")
    };
    assert_eq!(v.parameter(), "address");

    let err = gamma.get_profile("not-an-address").await.unwrap_err();
    let Error::Validation(v) = &err else {
        panic!("expected Error::Validation, got {err:?}")
    };
    assert_eq!(v.parameter(), "user_address");

    assert!(requests(&server).await.is_empty());
}

#[tokio::test]
async fn get_profile_by_user_address() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path(format!("/profiles/user_address/{ADDRESS}").as_str()))
        .respond_with(json_value(&fixture("Profile")))
        .expect(1)
        .mount(&server)
        .await;

    let profile = common::polymarket(&server)
        .gamma()
        .get_profile(ADDRESS)
        .await
        .unwrap();
    assert_eq!(profile.user, Some(7));
    assert_eq!(profile.proxy_wallet, Some(Address::from(ADDRESS)));
    assert!(profile.cert_req_date.is_some());
}
