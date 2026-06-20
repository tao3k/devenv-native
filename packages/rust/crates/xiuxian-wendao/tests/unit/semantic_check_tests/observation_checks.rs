use super::support::{create_node_with_observations, parse_observation};
use super::{SourceFile, check_code_observations};

#[test]
fn test_check_code_observations_valid_pattern() {
    let obs = parse_observation(r#"lang:rust "fn $NAME($$$) -> Result<$$$>""#);
    let node = create_node_with_observations("test.md#valid", vec![obs]);

    let mut issues = Vec::new();
    check_code_observations(&node, "test.md", &[], None, &mut issues);

    assert!(issues.is_empty());
}

#[test]
fn test_check_code_observations_unsupported_language() {
    let obs = parse_observation(r#"lang:brainfuck "+-<>""#);
    let node = create_node_with_observations("test.md#unsupported", vec![obs]);

    let mut issues = Vec::new();
    check_code_observations(&node, "test.md", &[], None, &mut issues);

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].issue_type, "invalid_observation_language");
    assert!(issues[0].message.contains("Unsupported language"));
    assert!(issues[0].message.contains("brainfuck"));
}

#[test]
fn test_check_code_observations_multiple_issues() {
    let obs1 = parse_observation(r#"lang:rust "fn $NAME()""#);
    let obs2 = parse_observation(r#"lang:brainfuck "+-<>""#);
    let obs3 = parse_observation(r#"lang:python "def $NAME():""#);

    let node = create_node_with_observations("test.md#mixed", vec![obs1, obs2, obs3]);

    let mut issues = Vec::new();
    check_code_observations(&node, "test.md", &[], None, &mut issues);

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].issue_type, "invalid_observation_language");
}

#[test]
fn test_check_code_observations_no_observations() {
    let node = create_node_with_observations("test.md#none", Vec::new());

    let mut issues = Vec::new();
    check_code_observations(&node, "test.md", &[], None, &mut issues);

    assert!(issues.is_empty());
}

#[test]
fn test_check_code_observations_python_valid() {
    let obs = parse_observation(r#"lang:python "def $NAME($$$): $$$BODY""#);
    let node = create_node_with_observations("test.md#python", vec![obs]);

    let mut issues = Vec::new();
    check_code_observations(&node, "test.md", &[], None, &mut issues);

    assert!(issues.is_empty());
}

#[test]
fn test_check_code_observations_typescript_valid() {
    let obs = parse_observation(r#"lang:typescript "function $NAME($$$): $$$RET""#);
    let node = create_node_with_observations("test.md#ts", vec![obs]);

    let mut issues = Vec::new();
    check_code_observations(&node, "test.md", &[], None, &mut issues);

    assert!(issues.is_empty());
}

#[test]
fn test_check_code_observations_with_fuzzy_suggestion() {
    let obs = parse_observation(r#"lang:rust "fn nonexistent_function($$$)""#);
    let node = create_node_with_observations("test.md#fuzzy", vec![obs]);

    let source = SourceFile {
        path: "src/lib.rs".to_string(),
        content: "fn existing_function(x: i32) -> i32 { x + 1 }".to_string(),
    };

    let mut issues = Vec::new();
    check_code_observations(&node, "test.md", &[source], None, &mut issues);

    assert!(issues.is_empty());
}
