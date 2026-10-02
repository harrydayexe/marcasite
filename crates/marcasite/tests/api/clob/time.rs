//! Server time: `GET /time`.

use marcasite::Error;
use wiremock::{
    Mock,
    matchers::{method, path},
};

use super::{clob, json};
use crate::common;

#[tokio::test]
async fn get_server_time() {
    let server = common::server().await;
    // Example response of `GET /time` in docs/polymarket/specs/clob-openapi.yaml.
    Mock::given(method("GET"))
        .and(path("/time"))
        .respond_with(json("1234567890"))
        .expect(1)
        .mount(&server)
        .await;

    let time = clob(&server).get_server_time().await.unwrap();
    assert_eq!(time.timestamp(), 1_234_567_890);
}

#[tokio::test]
async fn non_numeric_time_is_a_decode_error() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/time"))
        .respond_with(json(r#""soon""#))
        .mount(&server)
        .await;

    let err = clob(&server).get_server_time().await.unwrap_err();
    assert!(matches!(err, Error::Decode(_)), "{err:?}");
}
