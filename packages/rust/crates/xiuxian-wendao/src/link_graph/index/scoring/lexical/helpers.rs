//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::link_graph::index::LinkGraphDocument;

pub(super) fn doc_contains_token(
    doc: &LinkGraphDocument,
    token: &str,
    case_sensitive: bool,
) -> bool {
    if token.is_empty() {
        return false;
    }
    if case_sensitive {
        doc.id.contains(token)
            || doc.stem.contains(token)
            || doc.title.contains(token)
            || doc.path.contains(token)
            || doc.tags.iter().any(|tag| tag.contains(token))
            || doc.search_text.contains(token)
    } else {
        doc.id_lower.contains(token)
            || doc.stem_lower.contains(token)
            || doc.title_lower.contains(token)
            || doc.path_lower.contains(token)
            || doc.tags_lower.iter().any(|tag| tag.contains(token))
            || doc.search_text_lower.contains(token)
    }
}

pub(super) fn count_substring_occurrences(haystack: &str, needle: &str) -> usize {
    if haystack.is_empty() || needle.is_empty() {
        return 0;
    }
    haystack.match_indices(needle).count()
}

pub(in crate::link_graph::index) fn token_match_ratio(
    haystack: &str,
    query_tokens: &[String],
) -> f64 {
    if query_tokens.is_empty() {
        return 0.0;
    }
    let matched = query_tokens
        .iter()
        .filter(|token| !token.is_empty() && haystack.contains(token.as_str()))
        .count();
    (usize_to_f64_saturating(matched) / usize_to_f64_saturating(query_tokens.len())).clamp(0.0, 1.0)
}

fn usize_to_f64_saturating(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}
