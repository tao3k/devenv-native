//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
/// Check if the query matches the end of a path.
pub(super) fn path_match_suffix(path_lower: &[String], query_lower: &str) -> bool {
    // Try matching query against path suffixes
    let query_parts: Vec<&str> = query_lower.split('/').filter(|s| !s.is_empty()).collect();

    if query_parts.is_empty() {
        return false;
    }

    // Check if path ends with query parts
    if query_parts.len() > path_lower.len() {
        return false;
    }

    let suffix_start = path_lower.len() - query_parts.len();
    query_parts
        .iter()
        .enumerate()
        .all(|(i, query_part)| &path_lower[suffix_start + i] == query_part)
}

pub(super) fn similarity_ratio(left: usize, right: usize) -> f32 {
    f32::from(u16::try_from(left).unwrap_or(u16::MAX))
        / f32::from(u16::try_from(right.max(1)).unwrap_or(u16::MAX))
}
