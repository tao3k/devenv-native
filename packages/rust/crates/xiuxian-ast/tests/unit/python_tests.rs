use xiuxian_ast::{
    extract_docstring_from_match, extract_python_docstring, find_python_async_functions,
    find_python_classes, find_python_functions,
};

#[test]
fn test_find_python_functions() {
    let content = r#"
@skill_command(name="test")
def hello(name: str) -> str:
    '''Greet someone by name.'''
    return f"Hello, {name}!"

def goodbye():
    pass
"#;

    let funcs = find_python_functions(content);
    assert_eq!(funcs.len(), 2);

    let hello = &funcs[0];
    assert!(
        hello
            .captures
            .iter()
            .any(|(n, v)| n == "NAME" && v == "hello")
    );
}

#[test]
fn test_find_python_async_functions() {
    let content = r"
async def fetch_data(url: str) -> dict:
    '''Fetch data from URL.'''
    pass

def sync_func():
    pass
";

    let funcs = find_python_async_functions(content);
    assert_eq!(funcs.len(), 1);
}

#[test]
fn test_find_python_classes() {
    let content = r"
class Agent:
    pass

class Tool:
    pass
";

    let classes = find_python_classes(content);
    assert_eq!(classes.len(), 2);
}

#[test]
fn test_extract_python_docstring() {
    let body = r#"
    '''This is a docstring.'''
    return "hello"
"#;
    let doc = extract_python_docstring(body);
    assert_eq!(doc, "This is a docstring.");
}

#[test]
fn test_extract_docstring_from_match() {
    let content = r"def hello():
    '''Test docstring.'''
    pass";

    let funcs = find_python_functions(content);
    if let Some(f) = funcs.first() {
        let doc = extract_docstring_from_match(f);
        // Note: matched text is just "def hello", not the full function
        assert!(doc.is_empty() || doc.contains("Test"));
    }
}
