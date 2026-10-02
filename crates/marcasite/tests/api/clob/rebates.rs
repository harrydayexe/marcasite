//! Maker rebates: `GET /rebates/current`.

use chrono::NaiveDate;
use marcasite::Decimal;
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
    // Shape of `GET /rebates/current` as sent live (captured 2026-09-25 data on 2026-10-02):
    // `date` is an RFC 3339 date-time at midnight UTC (the spec says `YYYY-MM-DD`).
    Mock::given(method("GET"))
        .and(path("/rebates/current"))
        .and(query_param("date", "2026-02-27"))
        .and(query_param("maker_address", MAKER))
        .respond_with(json(
            r#"[{"date":"2026-02-27T00:00:00Z","condition_id":"0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af","asset_address":"0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB","maker_address":"0xFeA4cB3dD4ca7CefD3368653B7D6FF9BcDFca604","rebated_fees_usdc":"0.237519"}]"#,
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

/// A maker without rebates gets the body `null` live (the spec documents `[]`), with HTTP 200.
#[tokio::test]
async fn null_body_is_an_empty_list() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/rebates/current"))
        .respond_with(json("null"))
        .expect(1)
        .mount(&server)
        .await;

    let fees = clob(&server)
        .get_current_rebated_fees(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), "0x01")
        .await
        .unwrap();
    assert!(fees.is_empty());
}

/// The documented empty list and the plain `YYYY-MM-DD` date still decode.
#[tokio::test]
async fn documented_forms_still_decode() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/rebates/current"))
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
    assert_eq!(fees[0].date, date);
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
