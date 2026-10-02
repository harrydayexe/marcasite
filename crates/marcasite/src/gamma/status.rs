//! Health check: `/status`.

use marcasite_core::Result;

use super::GammaClient;

impl GammaClient {
    /// Checks the health of the Gamma API (`GET /status`).
    ///
    /// Returns the plain-text body of a successful response (the spec's example is `"OK"`).
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getGammaStatus` (no published doc
    /// page).
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// let gamma = marcasite::gamma::GammaClient::new()?;
    /// let status = gamma.get_status().await?;
    /// println!("Gamma API status: {status}");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The spec documents no error response for this endpoint; see
    /// [`Error`](crate::Error) for the failures every request can have.
    pub async fn get_status(&self) -> Result<String> {
        let response = self.transport.get(&["status"]).send_raw().await?;
        Ok(response.text())
    }
}
