use crate::search::contracts::AutocompleteSuggestion;

use super::types::LocalSymbolCandidate;

pub(crate) fn compare_candidates(
    left: &LocalSymbolCandidate,
    right: &LocalSymbolCandidate,
) -> std::cmp::Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| left.name.cmp(&right.name))
        .then_with(|| left.path.cmp(&right.path))
        .then_with(|| left.line_start.cmp(&right.line_start))
}

pub(crate) fn compare_suggestions(
    left: &AutocompleteSuggestion,
    right: &AutocompleteSuggestion,
) -> std::cmp::Ordering {
    suggestion_rank(left)
        .cmp(&suggestion_rank(right))
        .then_with(|| left.text.cmp(&right.text))
}

pub(crate) fn candidate_score(
    query_lower: &str,
    name_folded: &str,
    signature: &str,
    owner_title: &str,
) -> f64 {
    if name_folded == query_lower {
        return 1.0;
    }
    if name_folded.starts_with(query_lower) {
        return 0.97;
    }
    if name_folded.contains(query_lower) {
        return 0.93;
    }

    let signature_folded = signature.to_ascii_lowercase();
    if signature_folded.contains(query_lower) {
        return 0.86;
    }
    let owner_folded = owner_title.to_ascii_lowercase();
    if !owner_folded.is_empty() && owner_folded.contains(query_lower) {
        return 0.81;
    }
    0.0
}

pub(crate) fn autocomplete_matches_prefix(normalized_text: &str, normalized_prefix: &str) -> bool {
    if normalized_text.starts_with(normalized_prefix) {
        return true;
    }

    normalized_text
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .any(|token| !token.is_empty() && token.starts_with(normalized_prefix))
}

pub(crate) fn autocomplete_suggestion_type(
    language: &str,
    node_kind: Option<&str>,
) -> &'static str {
    if language != "markdown" {
        return "symbol";
    }

    match node_kind {
        Some("property" | "observation") => "metadata",
        _ => "heading",
    }
}

fn suggestion_rank(suggestion: &AutocompleteSuggestion) -> usize {
    match suggestion.suggestion_type.as_str() {
        "symbol" => 0,
        "heading" => 1,
        "metadata" => 2,
        _ => 3,
    }
}
