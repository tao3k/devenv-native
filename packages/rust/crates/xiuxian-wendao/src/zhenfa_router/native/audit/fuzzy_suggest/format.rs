//! `zhenfa_router::native::audit::fuzzy_suggest::format` owns Wendao audit fuzzy suggest format behavior.

use super::types::FuzzySuggestion;

/// Format a fuzzy suggestion for display in diagnostics.
#[must_use]
pub fn format_suggestion(suggestion: &FuzzySuggestion) -> String {
    format!(
        "Consider updating pattern to: {}\nConfidence: {:.0}%\n{}",
        suggestion.suggested_pattern,
        suggestion.confidence * 100.0,
        suggestion
            .source_location
            .as_ref()
            .map(|l| format!("Found similar code at: {l}"))
            .unwrap_or_default()
    )
}
