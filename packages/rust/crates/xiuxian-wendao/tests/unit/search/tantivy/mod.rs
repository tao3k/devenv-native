use crate::search::fuzzy::FuzzySearchOptions;

use super::compare::{best_match_candidate, collect_lowercase_chars};
use super::fragments::for_each_candidate_fragment;
use super::identifier::populate_identifier_boundaries;
use super::tokenizer::{CodeTokenizer, collect_search_tokens};
use super::{SearchDocument, SearchDocumentIndex, SearchDocumentMatchField};
use tantivy::tokenizer::{TokenStream, Tokenizer};

fn adjacent_identifier_fragments(value: &str) -> Vec<&str> {
    let mut boundaries = Vec::new();
    populate_identifier_boundaries(value, &mut boundaries);
    boundaries
        .windows(2)
        .map(|range| &value[range[0]..range[1]])
        .collect()
}

#[test]
fn search_document_index_supports_exact_lookup() {
    let index = SearchDocumentIndex::new();
    index
        .add_documents(vec![SearchDocument {
            id: "page:1".to_string(),
            title: "Solve Linear Systems".to_string(),
            kind: "reference".to_string(),
            path: "docs/solve.md".to_string().into(),
            scope: "repo".to_string(),
            namespace: "solve-guide".to_string(),
            terms: vec!["solver".to_string(), "matrix".to_string()],
        }])
        .unwrap_or_else(|error| panic!("shared document indexing succeeds: {error}"));

    let results = index
        .search_exact("solver", 10)
        .unwrap_or_else(|error| panic!("exact search succeeds: {error}"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "page:1");
}

#[test]
fn search_document_index_exposes_lightweight_exact_hits() {
    let index = SearchDocumentIndex::new();
    index
        .add_documents(vec![SearchDocument {
            id: "module:1".to_string(),
            title: "DifferentialEquations".to_string(),
            kind: "module".to_string(),
            path: "src/DifferentialEquations.jl".to_string().into(),
            scope: "repo".to_string(),
            namespace: "SciML".to_string(),
            terms: vec!["DiffEq".to_string()],
        }])
        .unwrap_or_else(|error| panic!("shared document indexing succeeds: {error}"));

    let hits = index
        .search_exact_hits("DiffEq", 10)
        .unwrap_or_else(|error| panic!("exact hit search succeeds: {error}"));
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "module:1");
    assert_eq!(hits[0].matched_field, None);
}

#[test]
fn tantivy_matcher_uses_best_fragment_for_fuzzy_titles() {
    let index = SearchDocumentIndex::new();
    index
        .add_documents(vec![SearchDocument {
            id: "page:1".to_string(),
            title: "Solve Linear Systems".to_string(),
            kind: "reference".to_string(),
            path: "docs/solve.md".to_string().into(),
            scope: "repo".to_string(),
            namespace: "solve-guide".to_string(),
            terms: vec!["solver".to_string()],
        }])
        .unwrap_or_else(|error| panic!("shared document indexing succeeds: {error}"));

    let results = index
        .search_fuzzy("slove", 10, FuzzySearchOptions::document_search())
        .unwrap_or_else(|error| panic!("fuzzy search succeeds: {error}"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].item.id, "page:1");
    assert_eq!(results[0].matched_text, "Solve");
}

#[test]
fn search_document_index_supports_multifield_fuzzy_hits() {
    let index = SearchDocumentIndex::new();
    index
        .add_documents(vec![SearchDocument {
            id: "symbol:1".to_string(),
            title: "OrdinaryDiffEq".to_string(),
            kind: "module".to_string(),
            path: "src/solvers/ordinary.jl".to_string().into(),
            scope: "repo".to_string(),
            namespace: "DifferentialEquations".to_string(),
            terms: vec!["solve_linear_system".to_string()],
        }])
        .unwrap_or_else(|error| panic!("shared document indexing succeeds: {error}"));

    let hits = index
        .search_fuzzy_hits("sysem", 10, FuzzySearchOptions::document_search())
        .unwrap_or_else(|error| panic!("fuzzy hit search succeeds: {error}"));
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "symbol:1");
    assert_eq!(hits[0].matched_field, Some(SearchDocumentMatchField::Terms));
    assert_eq!(hits[0].matched_text.as_deref(), Some("system"));
}

#[test]
fn search_document_index_supports_phrase_prefix_hits() {
    let index = SearchDocumentIndex::new();
    index
        .add_documents(vec![SearchDocument {
            id: "page:2".to_string(),
            title: "Solve Linear Systems".to_string(),
            kind: "reference".to_string(),
            path: "docs/linear/solve.md".to_string().into(),
            scope: "repo".to_string(),
            namespace: "linear-guide".to_string(),
            terms: vec!["linear algebra".to_string()],
        }])
        .unwrap_or_else(|error| panic!("shared document indexing succeeds: {error}"));

    let hits = index
        .search_prefix_hits("solve linear sy", 10)
        .unwrap_or_else(|error| panic!("prefix hit search succeeds: {error}"));
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "page:2");
}

#[test]
fn code_tokenizer_splits_camel_snake_and_digit_boundaries() {
    let index = SearchDocumentIndex::new();
    let tokens = collect_search_tokens(&index.index, "solveLinear_system2D HTTPRequest");
    assert_eq!(
        tokens,
        vec![
            "solve".to_string(),
            "linear".to_string(),
            "system".to_string(),
            "2".to_string(),
            "d".to_string(),
            "http".to_string(),
            "request".to_string()
        ]
    );
}

#[test]
fn code_tokenizer_streams_unicode_prefix_terms() {
    let mut tokenizer = CodeTokenizer;
    let mut stream = tokenizer.token_stream("ÄpfelÜber");
    let mut tokens = Vec::new();
    stream.process(&mut |token| tokens.push(token.text.clone()));
    assert_eq!(tokens, vec!["Äpfel".to_string(), "Über".to_string()]);
}

#[test]
fn candidate_fragments_split_camel_case_and_identifier_spans() {
    let mut seen_ranges = Vec::new();
    let mut boundary_scratch = Vec::new();
    let mut fragments = Vec::new();
    for_each_candidate_fragment(
        "solveLinearSystem2D",
        &mut seen_ranges,
        &mut boundary_scratch,
        |fragment| fragments.push(fragment.to_string()),
    );
    assert!(fragments.iter().any(|fragment| fragment == "solve"));
    assert!(fragments.iter().any(|fragment| fragment == "Linear"));
    assert!(fragments.iter().any(|fragment| fragment == "System"));
    assert!(fragments.iter().any(|fragment| fragment == "2"));
    assert!(fragments.iter().any(|fragment| fragment == "D"));
    assert!(fragments.iter().any(|fragment| fragment == "LinearSystem"));
}

#[test]
fn populate_identifier_boundaries_tracks_camel_case_and_digit_edges() {
    let fragments = adjacent_identifier_fragments("solveLinearSystem2D");
    assert_eq!(fragments, vec!["solve", "Linear", "System", "2", "D"]);
}

#[test]
fn populate_identifier_boundaries_tracks_acronym_to_word_edges() {
    let fragments = adjacent_identifier_fragments("HTTPRequest");
    assert_eq!(fragments, vec!["HTTP", "Request"]);
}

#[test]
fn candidate_fragments_deduplicate_case_insensitive_repeats() {
    let mut seen_ranges = Vec::new();
    let mut boundary_scratch = Vec::new();
    let mut fragments = Vec::new();
    for_each_candidate_fragment(
        "Solve solve SOLVE",
        &mut seen_ranges,
        &mut boundary_scratch,
        |fragment| fragments.push(fragment.to_string()),
    );

    let solve_count = fragments
        .iter()
        .filter(|fragment| fragment.eq_ignore_ascii_case("solve"))
        .count();
    assert_eq!(solve_count, 1);
}

#[test]
fn best_match_candidate_prefers_camel_case_subfragments() {
    let mut query_chars = Vec::new();
    let mut candidate_chars = Vec::new();
    let mut scratch = Vec::new();
    let mut seen_ranges = Vec::new();
    let mut boundary_scratch = Vec::new();
    collect_lowercase_chars("equations", &mut query_chars);
    let best = best_match_candidate(
        "equations",
        query_chars.as_slice(),
        "DifferentialEquations",
        FuzzySearchOptions::document_search(),
        &mut candidate_chars,
        &mut scratch,
        &mut seen_ranges,
        &mut boundary_scratch,
    )
    .unwrap_or_else(|| panic!("best fragment should be found"));
    assert_eq!(best.0, "Equations");
    assert_eq!(best.1.distance, 0);
}
