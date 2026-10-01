//! Server time: `GET /time`.

use chrono::{DateTime, Utc};
use polyoxide_core::{Result, serde_util};
use serde::Deserialize;

use super::ClobClient;

/// The `/time` response: a bare JSON integer of Unix seconds.
#[derive(Deserialize)]
#[serde(transparent)]
struct ServerTime(#[serde(with = "serde_util::timestamp_seconds")] DateTime<Utc>);

impl ClobClient {
    /// Gets the server's current time (`GET /time`), e.g. to synchronise the local clock.
    ///
    /// The server returns a Unix timestamp in seconds.
    ///
    /// See <https://docs.polymarket.com/api-reference/data/get-server-time>.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn get_server_time(&self) -> Result<DateTime<Utc>> {
        let time: ServerTime = self.transport.get(&["time"]).send().await?;
        Ok(time.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Example `1234567890` from `GET /time` in docs/specs/clob-openapi.yaml.
    #[test]
    fn deserializes_server_time() {
        let time: ServerTime = serde_json::from_str("1234567890").unwrap();
        assert_eq!(time.0.timestamp(), 1_234_567_890);
    }
}
