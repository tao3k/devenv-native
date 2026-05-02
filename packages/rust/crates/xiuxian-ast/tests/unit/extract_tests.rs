use xiuxian_ast::{
    Lang, extract_items, extract_skeleton, get_skeleton_patterns, semantic_fingerprint,
};

#[test]
fn test_extract_python_functions() {
    let content = r#"
def hello(name: str) -> str:
    '''Greet someone.'''
    return f"Hello, {name}!"

def goodbye():
    pass
"#;

    let results = extract_items(content, "def $NAME", Lang::Python, None);
    // Should find 2 top-level functions (not method inside class)
    assert_eq!(results.len(), 2);

    // Check first function
    let hello = &results[0];
    assert!(hello.text.starts_with("def hello"));
    assert!(hello.captures.contains_key("NAME"));
    assert_eq!(hello.captures["NAME"], "hello");
    assert!(hello.line_start >= 2);
    assert!(hello.line_end >= hello.line_start);
}

#[test]
fn test_extract_with_capture_filter() {
    let content = r#"
def hello(name: str) -> str:
    return f"Hello, {name}!"

def goodbye():
    pass
"#;

    // Use simple pattern without ARGS to match both functions
    let results = extract_items(content, "def $NAME", Lang::Python, Some(vec!["NAME"]));
    assert_eq!(results.len(), 2);

    for r in &results {
        assert!(r.captures.contains_key("NAME"));
    }
}

#[test]
fn test_extract_rust_functions() {
    let content = r#"
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn goodbye() {
    println!("Goodbye");
}
"#;

    let results = extract_items(content, "fn $NAME", Lang::Rust, None);
    assert_eq!(results.len(), 2);

    let hello = &results[0];
    assert!(hello.text.starts_with("fn hello"));
    assert_eq!(hello.captures["NAME"], "hello");
}

#[test]
fn test_extract_classes() {
    let content = r"
class MyClass:
    def method(self):
        pass

class AnotherClass:
    pass
";

    let results = extract_items(content, "class $NAME", Lang::Python, None);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_extract_empty_result() {
    let content = "let x = 42;";
    let results = extract_items(content, "def $NAME", Lang::Python, None);
    assert!(results.is_empty());
}

#[test]
fn test_line_numbers() {
    let content = "x = 1\ny = 2\nz = 3\n";
    let results = extract_items(content, "$NAME = $VALUE", Lang::Python, Some(vec!["NAME"]));

    // Should find 3 matches
    assert_eq!(results.len(), 3);

    // Check line numbers (1-indexed)
    assert_eq!(results[0].line_start, 1);
    assert_eq!(results[1].line_start, 2);
    assert_eq!(results[2].line_start, 3);
}

#[test]
fn test_invalid_pattern() {
    let content = "def hello(): pass";
    let results = extract_items(content, "invalid[pattern", Lang::Python, None);
    assert!(results.is_empty());
}

#[test]
fn test_extract_variables() {
    let content = r#"
x = 1
y = 2
name = "hello"
"#;

    let results = extract_items(content, "$NAME = $VALUE", Lang::Python, None);
    assert_eq!(results.len(), 3);

    for r in &results {
        assert!(r.captures.contains_key("NAME"));
        assert!(r.captures.contains_key("VALUE"));
    }
}

#[test]
fn test_extract_toml_items() {
    let content = r#"
[package]
name = "searchrust"
version = "0.1.0"
"#;

    let table_results = extract_items(content, "[$NAME]", Lang::Toml, Some(vec!["NAME"]));
    assert_eq!(table_results.len(), 1);
    assert_eq!(table_results[0].captures["NAME"], "package");
    assert!(table_results[0].text.contains("[package]"));

    let field_results = extract_items(
        content,
        "$NAME = $VALUE",
        Lang::Toml,
        Some(vec!["NAME", "VALUE"]),
    );
    assert_eq!(field_results.len(), 2);
    assert_eq!(field_results[0].captures["NAME"], "name");
    assert_eq!(field_results[0].captures["VALUE"], "\"searchrust\"");
    assert_eq!(field_results[1].captures["NAME"], "version");
    assert_eq!(field_results[1].captures["VALUE"], "\"0.1.0\"");
}

#[test]
fn test_extract_skeleton_python() {
    let content = r#"
def hello(name: str) -> str:
    """Greet someone by name."""
    return f"Hello, {name}!"

def goodbye():
    """Say goodbye."""
    print("Goodbye")

class MyClass:
    """A sample class."""
    def method(self):
        """A method."""
        pass
"#;

    let skeleton = extract_skeleton(content, Lang::Python);

    // Should contain function/class names
    assert!(
        skeleton.contains("hello"),
        "Should contain 'hello' function"
    );
    assert!(
        skeleton.contains("goodbye"),
        "Should contain 'goodbye' function"
    );
    assert!(skeleton.contains("MyClass"), "Should contain 'MyClass'");

    // Should NOT contain implementation details
    assert!(
        !skeleton.contains("return f"),
        "Should not contain 'return f'"
    );
    assert!(!skeleton.contains("print("), "Should not contain 'print('");
    assert!(!skeleton.contains("pass"), "Should not contain 'pass'");

    // Each signature should be on its own line
    let lines: Vec<&str> = skeleton.lines().collect();
    assert!(lines.len() >= 3, "Should have at least 3 signatures");
}

#[test]
fn test_extract_skeleton_rust() {
    let content = r#"
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub struct User {
    id: u32,
    name: String,
}

impl User {
    pub fn new(id: u32, name: String) -> Self {
        User { id, name }
    }
}
"#;

    let skeleton = extract_skeleton(content, Lang::Rust);

    // Should contain signatures (truncated at '{')
    assert!(skeleton.contains("fn hello"), "Should contain 'fn hello'");
    assert!(
        skeleton.contains("pub struct User"),
        "Should contain 'pub struct User'"
    );
    assert!(skeleton.contains("impl User"), "Should contain 'impl User'");

    // Should NOT contain implementation
    assert!(
        !skeleton.contains("format!"),
        "Should not contain 'format!'"
    );
    assert!(
        !skeleton.contains("User { id, name }"),
        "Should not contain struct init"
    );
}

#[test]
fn test_semantic_fingerprint_reuses_rust_structure_for_comment_only_churn() {
    let first = semantic_fingerprint(
        r#"
fn hello(name: &str) -> String {
    format!("Hello, {name}")
}
"#,
        Lang::Rust,
    )
    .unwrap_or_else(|| panic!("semantic fingerprint should exist"));
    let second = semantic_fingerprint(
        r#"
fn hello(name: &str) -> String {
    // semantic no-op
    format!("Hello, {name}")
}
"#,
        Lang::Rust,
    )
    .unwrap_or_else(|| panic!("semantic fingerprint should exist"));

    assert_eq!(first, second);
}

#[test]
fn test_semantic_fingerprint_invalidates_on_rust_signature_change() {
    let first = semantic_fingerprint(
        r#"
fn hello(name: &str) -> String {
    format!("Hello, {name}")
}
"#,
        Lang::Rust,
    )
    .unwrap_or_else(|| panic!("semantic fingerprint should exist"));
    let second = semantic_fingerprint(
        r#"
fn hello(name: &str, suffix: &str) -> String {
    format!("Hello, {name}{suffix}")
}
"#,
        Lang::Rust,
    )
    .unwrap_or_else(|| panic!("semantic fingerprint should exist"));

    assert_ne!(first, second);
}

#[test]
fn test_get_skeleton_patterns() {
    let py_patterns = get_skeleton_patterns(Lang::Python);
    assert!(py_patterns.contains(&"def $NAME"));
    assert!(py_patterns.contains(&"class $NAME"));

    let rs_patterns = get_skeleton_patterns(Lang::Rust);
    assert!(rs_patterns.contains(&"fn $NAME"));
    assert!(rs_patterns.contains(&"pub fn $NAME"));

    let toml_patterns = get_skeleton_patterns(Lang::Toml);
    assert!(toml_patterns.contains(&"$NAME = $VALUE"));
    assert!(toml_patterns.contains(&"[$NAME]"));
}

#[test]
fn test_extract_skeleton_toml() {
    let content = r#"
[package]
name = "searchrust"
version = "0.1.0"
edition = "2021"
"#;

    let skeleton = extract_skeleton(content, Lang::Toml);

    assert!(
        skeleton.contains("[package]"),
        "Should contain package table"
    );
    assert!(
        skeleton.contains("name = \"searchrust\""),
        "Should contain name key"
    );
    assert!(
        skeleton.contains("version = \"0.1.0\""),
        "Should contain version key"
    );
}
