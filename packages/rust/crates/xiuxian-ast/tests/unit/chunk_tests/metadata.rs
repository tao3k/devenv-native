use super::support::chunk_or_panic;
use xiuxian_ast::Lang;

#[test]
fn test_chunk_preserves_order() {
    let content = r"
class First:
    pass

def second():
    pass

class Third:
    pass

def fourth():
    pass
";

    let chunks = chunk_or_panic(
        content,
        "test.py",
        Lang::Python,
        &["def $NAME", "class $NAME"],
        1,
        0,
    );

    assert_eq!(chunks.len(), 4);
    assert!(chunks[0].id.contains("First"));
    assert!(chunks[1].id.contains("second"));
    assert!(chunks[2].id.contains("Third"));
    assert!(chunks[3].id.contains("fourth"));
}

#[test]
fn test_chunk_metadata_extraction() {
    let content = r#"
def process_user_data(user_id: int, name: str, email: str) -> bool:
    """Process user data."""
    return True
"#;

    let chunks = chunk_or_panic(content, "test.py", Lang::Python, &["def $NAME"], 1, 0);

    assert_eq!(chunks.len(), 1);
    let chunk = &chunks[0];

    assert!(chunk.metadata.contains_key("NAME"));
    assert_eq!(chunk.metadata["NAME"], "process_user_data");
}

#[test]
fn test_chunk_with_single_quoted_docstring() {
    let content = r"
def hello():
    '''Single quoted docstring.'''
    pass
";

    let chunks = chunk_or_panic(content, "test.py", Lang::Python, &["def $NAME"], 1, 0);

    assert_eq!(chunks.len(), 1);
    assert_eq!(
        chunks[0].docstring,
        Some("Single quoted docstring.".to_string())
    );
}

#[test]
fn test_chunk_multiple_patterns_same_file() {
    let content = r"
def foo():
    pass

class Bar:
    pass

def baz():
    pass
";

    let chunks = chunk_or_panic(
        content,
        "test.py",
        Lang::Python,
        &["def $NAME", "class $NAME"],
        1,
        0,
    );

    assert_eq!(chunks.len(), 3);
}

#[test]
fn test_chunk_line_numbers_correct() {
    let content = r"
def first():
    line 2

def second():
    line 5
";

    let chunks = chunk_or_panic(content, "test.py", Lang::Python, &["def $NAME"], 1, 0);

    assert_eq!(chunks.len(), 2);

    assert_eq!(chunks[0].line_start, 2);
    assert_eq!(chunks[0].line_end, 3);
    assert_eq!(chunks[1].line_start, 5);
    assert_eq!(chunks[1].line_end, 6);
}
