//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::link_graph::LinkGraphMatchStrategy;

pub(in crate::parsers::link_graph::query) fn looks_like_regex(query: &str) -> bool {
    let q = query.trim();
    if q.is_empty() {
        return false;
    }
    let has_bracket_class = q.contains('[') && q.contains(']');
    let has_regex_group_or_escape = q.contains("(?")
        || q.contains("\\b")
        || q.contains("\\d")
        || q.contains("\\w")
        || q.contains("\\s");
    q.starts_with('^')
        || q.ends_with('$')
        || q.contains(".*")
        || has_bracket_class
        || q.contains('\\')
        || has_regex_group_or_escape
}

pub(in crate::parsers::link_graph::query) fn looks_machine_like(query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return false;
    }
    let is_slugish = q
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    let has_signal = q.chars().any(|c| c.is_ascii_digit()) || q.contains('_') || q.contains('-');
    let has_note_suffix = [".md", ".mdx", ".markdown"].iter().any(|ext| {
        if !q.ends_with(ext) {
            return false;
        }
        let prefix_len = q.len().saturating_sub(ext.len());
        if prefix_len == 0 {
            return false;
        }
        q[..prefix_len]
            .chars()
            .any(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    });
    let pathish = q.contains('/') || has_note_suffix;
    (is_slugish && (has_signal || q.len() >= 24)) || pathish
}

pub(in crate::parsers::link_graph::query) fn infer_strategy_from_residual(
    query: &str,
) -> Option<LinkGraphMatchStrategy> {
    if looks_like_regex(query) {
        Some(LinkGraphMatchStrategy::Re)
    } else if looks_machine_like(query) {
        Some(LinkGraphMatchStrategy::Exact)
    } else {
        None
    }
}
