//! Rule-based extraction of `QueryIntent` signals from user text.

use super::models::QueryIntent;
use super::vocabulary::{ACTION_VERBS, DOMAIN_TARGETS, STOP_WORDS};
use std::collections::HashSet;

/// Extract a structured `QueryIntent` from a raw natural-language query.
///
/// The algorithm is zero-allocation-heavy and rule-based — no ML model needed.
/// It runs in microseconds, making it suitable for hot-path routing.
#[must_use]
pub fn extract_intent(query: &str) -> QueryIntent {
    let normalized = query.trim().to_lowercase();
    if normalized.is_empty() {
        return empty_query_intent(normalized);
    }

    let tokens = tokenize_intent_query(normalized.as_str());
    let keywords = extract_keywords(tokens.as_slice());
    let action = first_canonical_action(tokens.as_slice());
    let target = first_canonical_target(tokens.as_slice(), action.as_deref())
        .or_else(|| inferred_target_from_action(action.as_deref()));
    let context = context_keywords(keywords.as_slice(), action.as_deref(), target.as_deref());

    QueryIntent {
        action,
        target,
        context,
        keywords,
        normalized_query: normalized,
    }
}

fn empty_query_intent(normalized_query: String) -> QueryIntent {
    QueryIntent {
        normalized_query,
        ..Default::default()
    }
}

fn tokenize_intent_query(normalized: &str) -> Vec<&str> {
    normalized
        .split(is_intent_query_separator)
        .filter(|token| !token.is_empty())
        .collect()
}

fn is_intent_query_separator(candidate: char) -> bool {
    candidate.is_whitespace() || matches!(candidate, '.' | '_' | '-' | '/' | ',')
}

fn extract_keywords(tokens: &[&str]) -> Vec<String> {
    let stop_set: HashSet<&str> = STOP_WORDS.iter().copied().collect();
    tokens
        .iter()
        .copied()
        .filter(|token| token.len() >= 2 && !stop_set.contains(*token))
        .map(std::string::ToString::to_string)
        .collect()
}

fn first_canonical_action(tokens: &[&str]) -> Option<String> {
    tokens
        .iter()
        .copied()
        .find_map(|token| canonical_vocabulary_value(ACTION_VERBS, token))
}

fn first_canonical_target(tokens: &[&str], action: Option<&str>) -> Option<String> {
    tokens
        .iter()
        .copied()
        .filter(|token| action != Some(*token))
        .find_map(|token| canonical_vocabulary_value(DOMAIN_TARGETS, token))
}

fn canonical_vocabulary_value(vocabulary: &[(&str, &str)], token: &str) -> Option<String> {
    vocabulary
        .iter()
        .find_map(|(word, canonical)| (*word == token).then(|| (*canonical).to_string()))
}

fn inferred_target_from_action(action: Option<&str>) -> Option<String> {
    match action {
        Some(
            "commit" | "push" | "pull" | "merge" | "rebase" | "branch" | "checkout" | "diff"
            | "status" | "log" | "stash",
        ) => Some("git".to_string()),
        Some("crawl" | "research") => Some("web".to_string()),
        Some("embed" | "index") => Some("database".to_string()),
        _ => None,
    }
}

fn context_keywords(
    keywords: &[String],
    action: Option<&str>,
    target: Option<&str>,
) -> Vec<String> {
    let action_tokens = canonical_source_tokens(ACTION_VERBS, action);
    let target_tokens = canonical_source_tokens(DOMAIN_TARGETS, target);
    keywords
        .iter()
        .filter(|keyword| {
            !action_tokens.contains(keyword.as_str()) && !target_tokens.contains(keyword.as_str())
        })
        .cloned()
        .collect()
}

fn canonical_source_tokens<'a>(
    vocabulary: &'a [(&str, &str)],
    canonical_value: Option<&str>,
) -> HashSet<&'a str> {
    vocabulary
        .iter()
        .filter_map(|(word, canonical)| (canonical_value == Some(*canonical)).then_some(*word))
        .collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/intent/extract.rs"]
mod tests;
