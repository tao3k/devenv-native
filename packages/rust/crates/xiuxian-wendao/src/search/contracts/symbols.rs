use serde::{Deserialize, Serialize};
use specta::Type;

/// A single autocomplete suggestion.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AutocompleteSuggestion {
    /// Suggestion text emitted to the caller.
    pub text: String,
    /// Logical suggestion classification.
    pub suggestion_type: String,
}
