use super::{
    CONFIDENCE_THRESHOLD, FuzzySuggestion, PatternSkeleton, SourceFile, cache_stats,
    clear_candidate_cache, extract_capture_name, format_suggestion, jaccard_similarity,
    levenshtein_distance, string_similarity, suggest_pattern_fix, tokenize_pattern,
};

#[test]
fn test_tokenize_pattern_simple() {
    let tokens = tokenize_pattern("fn $NAME()");
    assert!(tokens.contains(&"fn".to_string()));
    assert!(tokens.contains(&"$NAME".to_string()));
    assert!(tokens.contains(&"(".to_string()));
    assert!(tokens.contains(&")".to_string()));
}

#[test]
fn test_tokenize_pattern_with_arrow() {
    let tokens = tokenize_pattern("fn $NAME() -> Result<$$$>");
    assert!(tokens.contains(&"->".to_string()));
    assert!(tokens.contains(&"<".to_string()));
    assert!(tokens.contains(&">".to_string()));
}

#[test]
fn test_extract_pattern_skeleton() {
    let skeleton = PatternSkeleton::extract("fn $NAME($$$ARGS) -> Result<$$$>");

    assert!(skeleton.keywords.contains(&"fn".to_string()));
    assert!(skeleton.keywords.contains(&"Result".to_string()));
    assert!(skeleton.metavariables.contains(&"$NAME".to_string()));
    assert!(skeleton.metavariables.contains(&"$$$ARGS".to_string()));
    assert!(skeleton.structure.contains(&"(".to_string()));
    assert!(skeleton.structure.contains(&")".to_string()));
    assert!(skeleton.structure.contains(&"->".to_string()));
}

#[test]
fn test_jaccard_similarity_identical() {
    let a = vec!["fn", "Result"];
    let b = vec!["fn", "Result"];
    assert!((jaccard_similarity(&a, &b) - 1.0).abs() < 0.001);
}

#[test]
fn test_jaccard_similarity_no_overlap() {
    let a = vec!["fn", "Result"];
    let b = vec!["def", "class"];
    assert!((jaccard_similarity(&a, &b) - 0.0).abs() < 0.001);
}

#[test]
fn test_jaccard_similarity_partial() {
    let a = vec!["fn", "Result"];
    let b = vec!["fn", "Option"];
    // intersection = 1, union = 3
    assert!((jaccard_similarity(&a, &b) - 0.333).abs() < 0.01);
}

#[test]
fn test_levenshtein_distance_identical() {
    assert_eq!(levenshtein_distance("hello", "hello"), 0);
}

#[test]
fn test_levenshtein_distance_one_change() {
    assert_eq!(levenshtein_distance("hello", "hallo"), 1);
}

#[test]
fn test_levenshtein_distance_insertion() {
    assert_eq!(levenshtein_distance("hello", "hellos"), 1);
}

#[test]
fn test_string_similarity() {
    let sim = string_similarity("process_data", "process_records");
    assert!(sim > 0.5);
    assert!(sim < 1.0);
}

#[test]
fn test_extract_capture_name() {
    assert_eq!(extract_capture_name("fn $NAME"), Some("NAME".to_string()));
    assert_eq!(
        extract_capture_name("def $FUNC($$$)"),
        Some("FUNC".to_string())
    );
    assert_eq!(extract_capture_name("class $$$$"), None); // $$$ is skipped
}

#[test]
fn test_suggest_pattern_fix_finds_renamed_symbol() {
    clear_candidate_cache();
    let source = SourceFile {
        path: "src/lib.rs".to_string(),
        content: "fn process_records(data: Vec<u8>) -> Result<()> { todo!() }".to_string(),
    };

    let suggestion =
        suggest_pattern_fix("fn process_data($$$)", xiuxian_ast::Lang::Rust, &[source]);

    let Some(s) = suggestion else {
        panic!("expected a fuzzy suggestion for a renamed symbol");
    };
    assert!(s.suggested_pattern.contains("process_records"));
    assert!(s.confidence >= CONFIDENCE_THRESHOLD);
    assert!(s.source_location.is_some());
}

#[test]
fn test_candidate_cache_stats_and_clear() {
    clear_candidate_cache();
    assert_eq!(cache_stats(), (0, 0));

    let source = SourceFile {
        path: "src/lib.rs".to_string(),
        content: "fn process_records() { }".to_string(),
    };

    let _suggestion =
        suggest_pattern_fix("fn process_data($$$)", xiuxian_ast::Lang::Rust, &[source]);

    let (file_count, candidate_count) = cache_stats();
    assert_eq!(file_count, 1);
    assert!(candidate_count > 0);

    clear_candidate_cache();
    assert_eq!(cache_stats(), (0, 0));
}

#[test]
fn test_suggest_pattern_fix_no_similar_code() {
    let source = SourceFile {
        path: "src/lib.rs".to_string(),
        content: "struct Point { x: i32, y: i32 }".to_string(),
    };

    let suggestion =
        suggest_pattern_fix("fn process_data($$$)", xiuxian_ast::Lang::Rust, &[source]);

    // Should return None because no similar function exists
    assert!(suggestion.is_none());
}

#[test]
fn test_format_suggestion() {
    let suggestion = FuzzySuggestion {
        suggested_pattern: "fn process_records($$$)".to_string(),
        confidence: 0.85,
        source_location: Some("src/lib.rs:42".to_string()),
        replacement_drawer: r#":OBSERVE: lang:rust "fn process_records($$$)""#.to_string(),
    };

    let formatted = format_suggestion(&suggestion);
    assert!(formatted.contains("process_records"));
    assert!(formatted.contains("85%"));
    assert!(formatted.contains("src/lib.rs:42"));
}
