//! Network endpoint config for `SearchStrategyFlow` Flight materialization.

use super::constants::DEFAULT_TIMEOUT_SECONDS;

const DEFAULT_BACKEND_REPO_ID: &str = "main";

/// Network endpoint settings for Studio-backed `SearchStrategyFlow` Flight
/// materialization through the Rust bridge client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchStrategyFlowFlightMaterializationConfig {
    /// Base URL of the Studio Arrow Flight endpoint.
    pub(crate) base_url: String,
    /// Repo id used by native Wendao Flight query contracts.
    pub(crate) repo_id: String,
    /// Per-route request timeout in seconds.
    pub(crate) timeout_seconds: u64,
}

impl SearchStrategyFlowFlightMaterializationConfig {
    /// Creates a Flight materialization config.
    ///
    /// # Errors
    ///
    /// Returns an error when the endpoint or repo id is blank.
    pub fn new(base_url: impl Into<String>, repo_id: impl Into<String>) -> Result<Self, String> {
        let base_url = base_url.into();
        if base_url.trim().is_empty() {
            return Err("SearchStrategyFlow Flight base URL must not be blank".to_owned());
        }
        let repo_id = repo_id.into();
        if repo_id.trim().is_empty() {
            return Err("SearchStrategyFlow Flight repo id must not be blank".to_owned());
        }
        Ok(Self {
            base_url,
            repo_id,
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
        })
    }

    /// Creates a Flight materialization config using the backend-owned default
    /// repository admission context.
    ///
    /// # Errors
    ///
    /// Returns an error when the endpoint is blank.
    pub fn new_with_backend_default_repo(base_url: impl Into<String>) -> Result<Self, String> {
        Self::new(base_url, DEFAULT_BACKEND_REPO_ID)
    }

    /// Sets the per-route request timeout.
    #[must_use]
    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds.max(1);
        self
    }
}
