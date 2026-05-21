//! `search::contracts::symbols` owns Wendao search contracts symbols behavior.

use serde::{Deserialize, Serialize};
use specta::Type;

/// A single autocomplete suggestion.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct AutocompleteSuggestion {
    /// Suggestion text emitted to the caller.
    pub text: String,
    /// Logical suggestion classification.
    pub suggestion_type: String,
}
