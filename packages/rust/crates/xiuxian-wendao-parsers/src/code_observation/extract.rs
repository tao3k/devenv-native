//! Extraction logic for Markdown `:OBSERVE:` property entries.

use super::CodeObservation;
use std::collections::HashMap;
use std::hash::BuildHasher;

/// Extract all `:OBSERVE:` entries from property drawer attributes.
///
/// Supports multiple observation patterns per section by using:
/// - Single `:OBSERVE:` with the full format
/// - Multiple `:OBSERVE:` entries (numbered or repeated)
#[must_use]
pub fn extract_observations<S: BuildHasher>(
    attributes: &HashMap<String, String, S>,
) -> Vec<CodeObservation> {
    attributes
        .get("OBSERVE")
        .and_then(|value| CodeObservation::parse(value))
        .into_iter()
        .chain(
            attributes
                .iter()
                .filter(|(key, _)| key.starts_with("OBSERVE_"))
                .filter_map(|(_, value)| CodeObservation::parse(value)),
        )
        .collect()
}
