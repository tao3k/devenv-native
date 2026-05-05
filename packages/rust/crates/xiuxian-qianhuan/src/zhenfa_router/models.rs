//! HTTP request and response models for Qianhuan Zhenfa endpoints.

use serde::{Deserialize, Serialize};

use crate::manifestation::ManifestationRenderRequest;

/// HTTP body for direct Qianhuan render endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpRenderRequest {
    /// Structured render request forwarded to manifestation manager.
    #[serde(flatten)]
    pub request: ManifestationRenderRequest,
}

/// HTTP success payload for direct render endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpRenderResponse {
    /// Rendered output text.
    pub result: String,
}

/// HTTP success payload for reload endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpReloadResponse {
    /// Whether template state changed and was reloaded.
    pub reloaded: bool,
}
