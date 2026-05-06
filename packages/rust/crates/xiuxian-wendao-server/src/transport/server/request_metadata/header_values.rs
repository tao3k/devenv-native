//! Shared metadata header parsing helpers.

use tonic::metadata::MetadataMap;

pub(super) fn split_non_empty_header_values(
    metadata: &MetadataMap,
    header: &'static str,
) -> Vec<String> {
    metadata
        .get(header)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .trim()
        .split(',')
        .filter(|value| !value.is_empty() || metadata.contains_key(header))
        .map(ToString::to_string)
        .collect()
}
