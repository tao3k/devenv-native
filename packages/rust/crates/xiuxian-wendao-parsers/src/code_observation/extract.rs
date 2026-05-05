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
    let mut observations = Vec::new();

    if let Some(value) = attributes.get("OBSERVE")
        && let Some(observation) = CodeObservation::parse(value)
    {
        observations.push(observation);
    }

    for (key, value) in attributes {
        if key.starts_with("OBSERVE_")
            && let Some(observation) = CodeObservation::parse(value)
        {
            observations.push(observation);
        }
    }

    observations
}
