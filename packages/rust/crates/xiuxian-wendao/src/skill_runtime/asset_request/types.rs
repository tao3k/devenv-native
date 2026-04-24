/// Strongly-typed handle for building semantic Wendao asset requests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WendaoAssetHandle;

/// Chainable, typed Wendao URI request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRequest {
    uri: String,
}

impl AssetRequest {
    /// Creates one request from a full semantic URI.
    #[must_use]
    pub fn new(uri: String) -> Self {
        Self { uri }
    }

    /// Returns full semantic URI.
    #[must_use]
    pub fn uri(&self) -> &str {
        &self.uri
    }
}
