use std::collections::HashMap;

use xiuxian_wendao_parsers::{CodeObservation, extract_observations, path_matches_scope};

fn parse_valid_observation(input: &str) -> CodeObservation {
    match CodeObservation::parse(input) {
        Some(observation) => observation,
        None => panic!("expected valid observation: {input}"),
    }
}

#[test]
fn test_parse_simple_pattern() {
    let input = r#"lang:rust "fn $NAME($$$ARGS) -> Result<$$$RET, $$$ERR>""#;
    let obs = parse_valid_observation(input);
    assert_eq!(obs.language, "rust");
    assert_eq!(obs.pattern, "fn $NAME($$$ARGS) -> Result<$$$RET, $$$ERR>");
    assert!(obs.scope.is_none());
}

#[test]
fn test_parse_with_scope() {
    let input = r#"lang:rust scope:"src/api/**" "fn $NAME($$$) -> Result<$$$>""#;
    let obs = parse_valid_observation(input);
    assert_eq!(obs.language, "rust");
    assert_eq!(obs.scope, Some("src/api/**".to_string()));
    assert_eq!(obs.pattern, "fn $NAME($$$) -> Result<$$$>");
}

#[test]
fn test_parse_with_complex_scope() {
    let input = r#"lang:rust scope:"packages/core/**/*.rs" "fn init()""#;
    let obs = parse_valid_observation(input);
    assert_eq!(obs.scope, Some("packages/core/**/*.rs".to_string()));
}

#[test]
fn test_parse_python_pattern() {
    let input = r#"lang:python "def $NAME($$$): $$$BODY""#;
    let obs = parse_valid_observation(input);
    assert_eq!(obs.language, "python");
    assert_eq!(obs.pattern, "def $NAME($$$): $$$BODY");
}

#[test]
fn test_parse_with_escaped_quotes() {
    let input = r#"lang:rust "fn foo() { let s = \"hello\"; }""#;
    let obs = parse_valid_observation(input);
    assert_eq!(obs.pattern, r#"fn foo() { let s = "hello"; }"#);
}

#[test]
fn test_parse_scope_with_escaped_quotes() {
    let input = r#"lang:rust scope:"path/with\"quote/**" "fn foo()""#;
    let obs = parse_valid_observation(input);
    assert_eq!(obs.scope, Some(r#"path/with"quote/**"#.to_string()));
}

#[test]
fn test_parse_missing_lang_prefix() {
    let input = r#""fn $NAME()""#;
    assert!(CodeObservation::parse(input).is_none());
}

#[test]
fn test_parse_missing_quotes() {
    let input = r"lang:rust fn $NAME()";
    assert!(CodeObservation::parse(input).is_none());
}

#[test]
fn test_parse_empty_language() {
    let input = r#"lang: "fn $NAME()""#;
    assert!(CodeObservation::parse(input).is_none());
}

#[test]
fn test_matches_scope_no_scope() {
    let obs = parse_valid_observation(r#"lang:rust "fn main()""#);
    assert!(obs.matches_scope("src/api/handler.rs"));
    assert!(obs.matches_scope("any/path.rs"));
}

#[test]
fn test_matches_scope_with_glob() {
    let obs = parse_valid_observation(r#"lang:rust scope:"src/api/**" "fn main()""#);
    assert!(obs.matches_scope("src/api/handler.rs"));
    assert!(obs.matches_scope("src/api/deep/nested/mod.rs"));
    assert!(!obs.matches_scope("src/db/handler.rs"));
    assert!(!obs.matches_scope("other/api/handler.rs"));
}

#[test]
fn test_matches_scope_double_star_suffix() {
    let obs = parse_valid_observation(r#"lang:rust scope:"**/*.rs" "fn main()""#);
    assert!(obs.matches_scope("src/api/handler.rs"));
    assert!(obs.matches_scope("lib.rs"));
    assert!(obs.matches_scope("deep/nested/path/mod.rs"));
    assert!(!obs.matches_scope("src/api/handler.py"));
}

#[test]
fn test_matches_scope_package_specific() {
    let obs = parse_valid_observation(r#"lang:rust scope:"packages/core/**" "fn init()""#);
    assert!(obs.matches_scope("packages/core/src/lib.rs"));
    assert!(obs.matches_scope("packages/core/api/handler.rs"));
    assert!(!obs.matches_scope("packages/api/src/lib.rs"));
}

#[test]
fn test_path_matches_scope_simple() {
    assert!(path_matches_scope("src/lib.rs", "src/lib.rs"));
    assert!(!path_matches_scope("src/other.rs", "src/lib.rs"));
}

#[test]
fn test_path_matches_scope_wildcard() {
    assert!(path_matches_scope("src/lib.rs", "src/*.rs"));
    assert!(path_matches_scope("src/api.rs", "src/*.rs"));
    assert!(!path_matches_scope("src/sub/lib.rs", "src/*.rs"));
}

#[test]
fn test_path_matches_scope_double_star() {
    assert!(path_matches_scope("src/api/handler.rs", "src/**"));
    assert!(path_matches_scope("src/deep/nested/path.rs", "src/**"));
    assert!(!path_matches_scope("lib/src/path.rs", "src/**"));
}

#[test]
fn test_path_matches_scope_multiple_double_star_segments() {
    assert!(path_matches_scope(
        "packages/core/src/deep/nested/path/lib.rs",
        "packages/**/src/**/nested/**/lib.rs"
    ));
    assert!(!path_matches_scope(
        "packages/core/src/deep/path/lib.rs",
        "packages/**/src/**/nested/**/lib.rs"
    ));
}

#[test]
fn test_validate_pattern_retired() {
    let obs = parse_valid_observation(r#"lang:rust "fn $NAME()""#);
    let result = obs.validate_pattern();
    assert!(result.is_err());
    match result {
        Err(error) => assert!(error.contains("language-provider boundary")),
        Ok(()) => panic!("expected retired validation failure"),
    }
}

#[test]
fn test_extract_observations_single() {
    let mut attrs = HashMap::new();
    attrs.insert(
        "OBSERVE".to_string(),
        r#"lang:rust "fn $NAME()""#.to_string(),
    );

    let observations = extract_observations(&attrs);
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].language, "rust");
}

#[test]
fn test_extract_observations_multiple() {
    let mut attrs = HashMap::new();
    attrs.insert(
        "OBSERVE_1".to_string(),
        r#"lang:rust "fn $NAME()""#.to_string(),
    );
    attrs.insert(
        "OBSERVE_2".to_string(),
        r#"lang:python "def $NAME():""#.to_string(),
    );

    let observations = extract_observations(&attrs);
    assert_eq!(observations.len(), 2);
}

#[test]
fn test_extract_observations_none() {
    let attrs = HashMap::new();
    let observations = extract_observations(&attrs);
    assert!(observations.is_empty());
}

#[test]
fn test_display() {
    let obs = parse_valid_observation(r#"lang:rust "fn main()""#);
    assert_eq!(obs.to_string(), r#":OBSERVE: lang:rust "fn main()""#);
}

#[test]
fn test_display_with_scope() {
    let obs = parse_valid_observation(r#"lang:rust scope:"src/api/**" "fn main()""#);
    assert_eq!(
        obs.to_string(),
        r#":OBSERVE: lang:rust scope:"src/api/**" "fn main()""#
    );
}

#[test]
fn test_with_line() {
    let obs = parse_valid_observation(r#"lang:rust "fn main()""#).with_line(42);
    assert_eq!(obs.line_number, Some(42));
}
