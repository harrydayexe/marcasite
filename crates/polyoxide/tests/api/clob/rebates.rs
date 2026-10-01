//! Maker rebates: `GET /rebates/current`.

use chrono::NaiveDate;
use polyoxide::Decimal;
use wiremock::{
    Mock,
    matchers::{method, path, query_param},
};

use super::{api_error, clob, json};
use crate::common;

const MAKER: &str = "0xFeA4cB3dD4ca7CefD3368653B7D6FF9BcDFca604";

#[tokio::test]
async fn get_current_rebated_fees() {
    let server = common::server().await;
    // Example response of `GET /rebates/current` in docs/specs/clob-openapi.yaml.
    Mock::given(method("GET"))
        .and(path("/rebates/current"))
        .and(query_param("date", "2026-02-27"))
        .and(query_param("maker_address", MAKER))
        .respond_with(json(
            r#"[{"date":"2026-02-27","condition_id":"0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af","asset_address":"0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB","maker_address":"0xFeA4cB3dD4ca7CefD3368653B7D6FF9BcDFca604","rebated_fees_usdc":"0.237519"}]"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let date = NaiveDate::from_ymd_opt(2026, 2, 27).unwrap();
    let fees = clob(&server)
        .get_current_rebated_fees(date, MAKER)
        .await
        .unwrap();
    assert_eq!(fees.len(), 1);
    assert_eq!(fees[0].date, date);
    assert_eq!(fees[0].rebated_fees_usdc, Decimal::new(237_519, 6));
}

#[tokio::test]
async fn invalid_maker_address_is_an_api_error() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/rebates/current"))
        .respond_with(api_error(400, "Invalid maker_address"))
        .expect(1)
        .mount(&server)
        .await;

    let err = clob(&server)
        .get_current_rebated_fees(NaiveDate::from_ymd_opt(2026, 2, 27).unwrap(), "nope")
        .await
        .unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(400));
    assert_eq!(
        err.api_error().unwrap().message(),
        Some("Invalid maker_address")
    );
}
