use super::super::*;
use super::support::chunk_or_panic;

#[test]
fn test_chunk_python_functions() {
    let content = r#"
def hello(name: str) -> str:
    """Greet someone."""
    return f"Hello, {name}!"

def goodbye():
    """Say goodbye."""
    pass

class Greeter:
    """A greeting class."""
    def __init__(self, name: str):
        self.name = name
"#;

    let chunks = chunk_or_panic(
        content,
        "test.py",
        Lang::Python,
        &["def $NAME", "class $NAME"],
        2,
        0,
    );

    assert_eq!(chunks.len(), 3);

    let funcs: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == "function")
        .collect();
    let classes: Vec<_> = chunks.iter().filter(|c| c.chunk_type == "class").collect();
    assert_eq!(funcs.len(), 2);
    assert_eq!(classes.len(), 1);

    let Some(docstring) = funcs[0].docstring.as_ref() else {
        panic!("expected hello docstring to be present");
    };
    assert_eq!(docstring, "Greet someone.");
}

#[test]
fn test_chunk_id_generation() {
    let content = r"
def my_function():
    pass
";

    let chunks = chunk_or_panic(content, "test.py", Lang::Python, &["def $NAME"], 1, 0);

    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].id.contains("my_function"));
    assert!(chunks[0].id.contains("test"));
    assert!(chunks[0].id.contains("function"));
}

#[test]
fn test_min_lines_filter() {
    let content = r"
def short():
    x = 1
def normal():
    x = 1
    y = 2
    z = 3
";

    let chunks = chunk_or_panic(content, "test.py", Lang::Python, &["def $NAME"], 3, 0);

    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].content.contains("normal"));
}

#[test]
fn test_max_lines_split() {
    let mut lines: Vec<String> = vec!["def large_function():".to_string()];
    for i in 0..24 {
        lines.push(format!("    x_{i} = {i}"));
    }
    let content = lines.join("\n");

    let chunks = chunk_or_panic(&content, "test.py", Lang::Python, &["def $NAME"], 1, 10);

    assert_eq!(chunks.len(), 3);
    for (i, chunk) in chunks.iter().enumerate() {
        assert!(chunk.id.contains("large_function"));
        assert!(chunk.id.contains(&format!("_part_{i}")));
    }
}

#[test]
fn test_chunk_empty_content() {
    let chunks = chunk_or_panic("", "empty.py", Lang::Python, &["def $NAME"], 1, 0);
    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_chunk_no_matches() {
    let content = r"
x = 1
y = 2
";

    let chunks = chunk_or_panic(
        content,
        "test.py",
        Lang::Python,
        &["def $NAME", "class $NAME"],
        1,
        0,
    );
    assert_eq!(chunks.len(), 0);
}
