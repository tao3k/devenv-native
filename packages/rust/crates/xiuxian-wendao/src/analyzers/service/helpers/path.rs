//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
pub(crate) fn hierarchy_segments_from_path(path: &str) -> Option<Vec<String>> {
    let segments = path
        .split('/')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    (!segments.is_empty()).then_some(segments)
}
