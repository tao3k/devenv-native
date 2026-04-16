use std::collections::HashSet;

use crate::search::attachment::build::attachment_kind_label;
use crate::search::ranking::RetainedWindow;

use super::types::{AttachmentCandidate, MIN_RETAINED_ATTACHMENTS, RETAINED_ATTACHMENT_MULTIPLIER};

pub(crate) fn normalize_extension_filters(extensions: &[String]) -> HashSet<String> {
    extensions
        .iter()
        .map(|value| value.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}

pub(crate) fn normalize_kind_filters(
    kinds: &[crate::link_graph::LinkGraphAttachmentKind],
) -> HashSet<String> {
    kinds
        .iter()
        .copied()
        .map(attachment_kind_label)
        .map(ToString::to_string)
        .collect()
}

pub(crate) fn build_query_tokens(normalized_query: &str) -> Vec<String> {
    normalized_query
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

pub(crate) fn retained_window(limit: usize) -> RetainedWindow {
    RetainedWindow::new(
        limit,
        RETAINED_ATTACHMENT_MULTIPLIER,
        MIN_RETAINED_ATTACHMENTS,
    )
}

pub(crate) fn compare_candidates(
    left: &AttachmentCandidate,
    right: &AttachmentCandidate,
) -> std::cmp::Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| left.attachment_path.cmp(&right.attachment_path))
        .then_with(|| left.source_path.cmp(&right.source_path))
}

pub(crate) fn candidate_score(
    normalized_query: &str,
    query_tokens: &[String],
    fields: &[&str; 5],
) -> f64 {
    if normalized_query.is_empty() {
        return 1.0;
    }

    let query_hit = fields.iter().any(|value| value.contains(normalized_query));
    let token_hit_count = query_tokens
        .iter()
        .filter(|token| fields.iter().any(|value| value.contains(token.as_str())))
        .count();
    if !query_hit && token_hit_count == 0 {
        return 0.0;
    }

    let exact_name = if fields[1] == normalized_query {
        1.0
    } else {
        0.0
    };
    let path_hit = if fields[0].contains(normalized_query) {
        1.0
    } else {
        0.0
    };
    let token_ratio = if query_tokens.is_empty() {
        0.0
    } else {
        usize_to_f64_saturating(token_hit_count) / usize_to_f64_saturating(query_tokens.len())
    };
    (exact_name * 0.5 + path_hit * 0.3 + token_ratio * 0.2).clamp(0.0, 1.0)
}

pub(crate) fn usize_to_f64_saturating(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}
