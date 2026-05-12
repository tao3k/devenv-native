//! `analyzers::query::refine` owns Wendao analyzers query refine behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Request to refine documentation for a specific code entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RefineEntityDocRequest {
    /// Owning repository identifier.
    pub repo_id: String,
    /// Entity identifier to refine (e.g. wendao URI).
    pub entity_id: String,
    /// Optional user provided hints or context to guide the refinement.
    pub user_hints: Option<String>,
}

/// Response for a documentation refinement request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct RefineEntityDocResponse {
    /// Owning repository identifier.
    pub repo_id: String,
    /// Entity identifier refined.
    pub entity_id: String,
    /// The refined content (Markdown).
    pub refined_content: String,
    /// Final verification state after Skeptic audit.
    pub verification_state: String,
}
