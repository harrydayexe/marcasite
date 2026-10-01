//! Health check: `/status`.

use polyoxide_core::Result;

use super::GammaClient;

impl GammaClient {
    /// Checks the health of the Gamma API.
    ///
    /// Returns the plain-text body of a successful response (documented as `"OK"`).
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getGammaStatus` (no published doc
    /// page).
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// assert_eq!(gamma.status().await?, "OK");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn status(&self) -> Result<String> {
        let response = self.transport.get(&["status"]).send_raw().await?;
        Ok(response.text())
    }
}
