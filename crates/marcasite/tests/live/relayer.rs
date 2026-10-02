//! Relayer API live tests (`GET /nonce`, `/relay-payload`, `/deployed`, `/transaction`).
//!
//! `GET /transaction` needs a real relayer transaction id. The only read-only sources are
//! authenticated (`GET /transactions`) or the documented example id, which the live API does
//! not know, so the documented not-found and validation paths are tested instead.

use marcasite::{
    Error,
    relayer::{DeploymentStatus, Nonce, NonceType, RelayPayload, WalletType},
};

use crate::common::{RELAYER, check, get, pm, sample};

/// The example id from `docs/polymarket/api-reference/relayer/get-a-transaction-by-id.md`.
const DOCUMENTED_TRANSACTION_ID: &str = "0190b317-a1d3-7bec-9b91-eeb6dcd3a620";

fn is_decimal(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn is_address(s: &str) -> bool {
    s.len() == 42 && s.starts_with("0x") && s[2..].bytes().all(|b| b.is_ascii_hexdigit())
}

#[tokio::test]
#[ignore = "live network"]
async fn get_nonce() {
    let s = sample().await;
    for (nonce_type, wire) in [(NonceType::Proxy, "PROXY"), (NonceType::Safe, "SAFE")] {
        let nonce = pm()
            .relayer()
            .get_nonce(s.user.as_str(), nonce_type)
            .await
            .unwrap();
        assert!(is_decimal(nonce.nonce.as_deref().unwrap()), "{nonce:?}");
        check::<Nonce>(
            &format!("GET /nonce type={wire}"),
            &get(
                RELAYER,
                "/nonce",
                &[("address", s.user.as_str()), ("type", wire)],
            )
            .await,
        );
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn get_relay_payload() {
    let s = sample().await;
    for (nonce_type, wire) in [(NonceType::Proxy, "PROXY"), (NonceType::Safe, "SAFE")] {
        let payload = pm()
            .relayer()
            .get_relay_payload(s.user.as_str(), nonce_type)
            .await
            .unwrap();
        assert!(is_address(payload.address.as_ref().unwrap().as_str()));
        assert!(is_decimal(payload.nonce.as_deref().unwrap()), "{payload:?}");
        check::<RelayPayload>(
            &format!("GET /relay-payload type={wire}"),
            &get(
                RELAYER,
                "/relay-payload",
                &[("address", s.user.as_str()), ("type", wire)],
            )
            .await,
        );
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn check_deployed() {
    let s = sample().await;
    let default = pm()
        .relayer()
        .check_deployed(s.user.as_str())
        .send()
        .await
        .unwrap();
    assert!(default.deployed.is_some());
    for (wallet_type, wire) in [(WalletType::Safe, "SAFE"), (WalletType::Wallet, "WALLET")] {
        let status = pm()
            .relayer()
            .check_deployed(s.user.as_str())
            .wallet_type(wallet_type)
            .send()
            .await
            .unwrap();
        assert!(status.deployed.is_some());
        check::<DeploymentStatus>(
            &format!("GET /deployed type={wire}"),
            &get(
                RELAYER,
                "/deployed",
                &[("address", s.user.as_str()), ("type", wire)],
            )
            .await,
        );
    }
    check::<DeploymentStatus>(
        "GET /deployed (no type)",
        &get(RELAYER, "/deployed", &[("address", s.user.as_str())]).await,
    );
}

/// SPEC_DEVIATIONS.md, Relayer: `/nonce` and `/deployed` answer `200` for a malformed
/// address where the spec says `400`. The SDK validates the address client-side, so this
/// only pins the live behaviour. (`/relay-payload` shows the same.)
#[tokio::test]
#[ignore = "live network"]
async fn malformed_addresses_are_accepted_by_live() {
    for (path, query) in [
        ("/nonce", [("address", "not-an-address"), ("type", "PROXY")]),
        (
            "/relay-payload",
            [("address", "not-an-address"), ("type", "SAFE")],
        ),
        (
            "/deployed",
            [("address", "not-an-address"), ("type", "SAFE")],
        ),
    ] {
        let response = reqwest::Client::new()
            .get(format!("{RELAYER}{path}"))
            .query(&query)
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status().as_u16(),
            200,
            "GET {path} with a malformed address is no longer a 200: update SPEC_DEVIATIONS.md"
        );
    }
    // The SDK refuses before sending.
    let err = pm()
        .relayer()
        .get_nonce("not-an-address", NonceType::Proxy)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
    let err = pm()
        .relayer()
        .check_deployed("not-an-address")
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
}

/// The documented 404 (`{"error":"transaction not found"}`) for an unknown id.
#[tokio::test]
#[ignore = "live network"]
async fn get_transaction_not_found() {
    let err = pm()
        .relayer()
        .get_transaction(DOCUMENTED_TRANSACTION_ID)
        .await
        .unwrap_err();
    assert!(err.is_not_found(), "{err:?}");
    let api = err.api_error().expect("a typed API error");
    assert_eq!(api.status().as_u16(), 404);
    assert_eq!(api.message(), Some("transaction not found"));
}

/// An empty id is rejected client-side; the live API answers 400 `invalid id` for it.
#[tokio::test]
#[ignore = "live network"]
async fn get_transaction_empty_id_is_rejected() {
    let err = pm().relayer().get_transaction("").await.unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
}
