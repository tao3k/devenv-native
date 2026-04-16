use super::super::*;
use super::support::chunk_or_panic;

#[test]
fn test_chunk_rust_functions() {
    let content = r#"
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn goodbye() {
    println!("Goodbye");
}

struct Greeter {
    name: String,
}

impl Greeter {
    fn new(name: String) -> Self {
        Self { name }
    }
}
"#;

    let chunks = chunk_or_panic(
        content,
        "lib.rs",
        Lang::Rust,
        &["fn $NAME", "struct $NAME"],
        1,
        0,
    );

    assert_eq!(chunks.len(), 4);

    let funcs: Vec<_> = chunks
        .iter()
        .filter(|c| c.chunk_type == "function")
        .collect();
    let structs: Vec<_> = chunks.iter().filter(|c| c.chunk_type == "struct").collect();
    assert_eq!(funcs.len(), 3);
    assert_eq!(structs.len(), 1);
}

#[test]
fn test_chunk_javascript_functions() {
    let content = r#"
function hello(name) {
    return `Hello, ${name}!`;
}

const goodbye = () => {
    console.log("Goodbye");
};

class Greeter {
    constructor(name) {
        this.name = name;
    }
}
"#;

    let chunks = chunk_or_panic(
        content,
        "app.js",
        Lang::JavaScript,
        &["function $NAME", "const $NAME"],
        1,
        0,
    );

    assert_eq!(chunks.len(), 2);
}

#[test]
fn test_chunk_python_async_functions() {
    let content = r#"
async def fetch_data(url: str) -> dict:
    """Fetch data from URL."""
    response = await http_get(url)
    return response.json()

async def process_items():
    """Process all items concurrently."""
    results = []
    for item in items:
        result = await process(item)
        results.append(result)
    return results
"#;

    let chunks = chunk_or_panic(content, "api.py", Lang::Python, &["async def $NAME"], 1, 0);

    assert_eq!(chunks.len(), 2);
    assert!(chunks[0].content.contains("fetch_data"));
    assert!(chunks[1].content.contains("process_items"));
}
