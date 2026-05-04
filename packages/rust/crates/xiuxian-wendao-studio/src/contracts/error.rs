use serde::{Deserialize, Serialize};
use specta::Type;

/// Base error for Studio API.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Optional failure details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}
