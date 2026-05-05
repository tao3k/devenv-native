use xiuxian_ast::{Lang, Match, extract, scan};

fn scan_or_panic(content: &str, pattern: &str, lang: Lang) -> Vec<Match> {
    match scan(content, pattern, lang) {
        Ok(matches) => matches,
        Err(error) => panic!("scan failed: {error}"),
    }
}

#[test]
fn test_scan_python() {
    let content = r#"
def hello(name: str) -> str:
    '''Greet someone.'''
    return f"Hello, {name}!"

def goodbye():
    pass
"#;

    let funcs = scan_or_panic(content, "def $NAME", Lang::Python);
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
fn test_extract() {
    let content = "def hello(name: str): pass";
    let name = extract(content, "def $NAME($ARGS)", "NAME", Lang::Python);
    assert_eq!(name, Some("hello".to_string()));
}
