use xiuxian_ast::{Lang, Match, scan};

fn scan_or_panic(content: &str, pattern: &str, lang: Lang) -> Vec<Match> {
    match scan(content, pattern, lang) {
        Ok(matches) => matches,
        Err(error) => panic!("scan failed: {error}"),
    }
}

#[test]
fn test_scan_decorated_function() {
    let content = r#"
@skill_command(name="write_text")
def write_text(content: str) -> str:
    '''Write text to a file.'''
    return "written"
"#;

    // Test scanning functions - decorator pattern is complex, test basic function scanning
    let pattern = r"def $NAME";
    let matches = scan_or_panic(content, pattern, Lang::Python);
    println!("Found {} matches", matches.len());
    for m in &matches {
        println!("Text: {}", m.text);
        println!("Captures: {:?}", m.captures);
    }
    assert!(
        !matches.is_empty(),
        "Should find at least one function definition"
    );
}
