use crate::search::fuzzy::{
    FuzzyMatcher, FuzzySearchOptions, LexicalMatcher, edit_distance, levenshtein_distance,
    shared_prefix_len,
};

#[test]
fn levenshtein_distance_keeps_transposition_as_two_without_flag() {
    assert_eq!(levenshtein_distance("storage", "stroage"), 2);
}

#[test]
fn edit_distance_supports_transposition_when_enabled() {
    assert_eq!(edit_distance("storage", "stroage", true), 1);
}

#[test]
fn lexical_matcher_respects_prefix_requirement() {
    let candidates = vec!["spawn".to_string(), "plan".to_string()];

    let matcher = LexicalMatcher::new(
        &candidates,
        String::as_str,
        FuzzySearchOptions::new(1, 1, true),
    );

    let search_results = matcher
        .search("spawnn", 10)
        .expect("lexical matcher succeeds");
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].matched_text, "spawn");
}

#[test]
fn shared_prefix_len_handles_unicode_case_pairs() {
    assert_eq!(shared_prefix_len("Äpfel", "äPFEL"), 5);
}

#[test]
fn lexical_matcher_respects_unicode_prefix_requirement() {
    let candidates = vec!["Äpfel".to_string(), "Banane".to_string()];

    let matcher = LexicalMatcher::new(
        &candidates,
        String::as_str,
        FuzzySearchOptions::new(1, 1, true),
    );

    let search_results = matcher
        .search("äpfelx", 10)
        .expect("unicode lexical matcher succeeds");
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].matched_text, "Äpfel");
}

#[test]
fn lexical_matcher_clears_thread_local_buffers_between_searches() {
    let candidates = vec![
        "spawn".to_string(),
        "plan".to_string(),
        "storage".to_string(),
    ];

    let matcher = LexicalMatcher::new(
        &candidates,
        String::as_str,
        FuzzySearchOptions::new(1, 1, true),
    );

    let first = matcher
        .search("spawnn", 10)
        .expect("first lexical matcher search succeeds");
    let second = matcher
        .search("plam", 10)
        .expect("second lexical matcher search succeeds");

    assert_eq!(first.len(), 1);
    assert_eq!(first[0].matched_text, "spawn");
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].matched_text, "plan");
}

#[test]
fn camel_case_symbol_profile_relaxes_prefix_length() {
    assert_eq!(FuzzySearchOptions::camel_case_symbol().prefix_length, 0);
}
