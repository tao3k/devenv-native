//! Tests for extractor module - code outline extraction.

use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

use xiuxian_code_intelligence::{CodeIntelligenceExtractor, SearchConfig, SymbolKind};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_python_outline() -> TestResult {
    let dir = TempDir::new()?;
    let path = dir.path().join("test.py");
    let content = r"
class Agent:
    def __init__(self, name: str):
        pass

    async def run(self, task: str) -> None:
        pass

def helper_function(x: int) -> int:
    return x * 2

class AnotherClass:
    pass
";
    File::create(&path)?.write_all(content.as_bytes())?;

    let outline = CodeIntelligenceExtractor::outline_file(&path, Some("python"))?;

    assert!(outline.contains("class Agent"));
    assert!(outline.contains("def helper_function"));
    Ok(())
}

#[test]
fn test_rust_outline() -> TestResult {
    let dir = TempDir::new()?;
    let path = dir.path().join("test.rs");
    let content = r"
pub struct ContextLoader {
    root: PathBuf,
}

impl ContextLoader {
    pub fn new() -> Self {
        Self { root: PathBuf::new() }
    }

    fn load_file(&self, path: &str) -> String {
        String::new()
    }
}

trait Printable {
    fn print(&self);
}
";
    File::create(&path)?.write_all(content.as_bytes())?;

    let outline = CodeIntelligenceExtractor::outline_file(&path, Some("rust"))?;

    assert!(outline.contains("ContextLoader"));
    assert!(outline.contains("impl"));
    assert!(outline.contains("Printable"));
    Ok(())
}

#[test]
fn typed_python_outline_symbols_include_source_language_and_kind() -> TestResult {
    let dir = TempDir::new()?;
    let path = dir.path().join("agent.py");
    File::create(&path)?.write_all(
        b"
class Agent:
    pass

def run():
    pass
",
    )?;

    let symbols = CodeIntelligenceExtractor::outline_file_symbols(&path, Some("python"))?;

    assert!(symbols.iter().any(|symbol| {
        symbol.source_path == path.to_string_lossy()
            && symbol.language == "python"
            && symbol.name == "Agent"
            && symbol.kind == SymbolKind::Class
    }));
    assert!(
        symbols
            .iter()
            .any(|symbol| { symbol.name == "run" && symbol.kind == SymbolKind::Function })
    );
    Ok(())
}

#[test]
fn typed_structural_search_hits_include_captures() -> TestResult {
    let dir = TempDir::new()?;
    let path = dir.path().join("lib.rs");
    File::create(&path)?.write_all(
        b"
pub fn index_documents() {}
pub fn search_documents() {}
",
    )?;

    let hits = CodeIntelligenceExtractor::search_file_hits(&path, "pub fn $NAME", Some("rust"))?;

    assert_eq!(hits.len(), 2);
    assert!(hits.iter().any(|hit| {
        hit.source_path == path.to_string_lossy()
            && hit.language == "rust"
            && hit
                .captures
                .get("NAME")
                .is_some_and(|name| name == "index_documents")
    }));
    Ok(())
}

#[test]
fn typed_directory_search_hits_keep_source_paths() -> TestResult {
    let dir = TempDir::new()?;
    let first = dir.path().join("first.rs");
    let second = dir.path().join("second.rs");
    File::create(&first)?.write_all(b"pub fn first() {}\n")?;
    File::create(&second)?.write_all(b"pub fn second() {}\n")?;

    let hits = CodeIntelligenceExtractor::search_directory_hits(
        dir.path(),
        "pub fn $NAME",
        SearchConfig::default(),
    )?;

    assert_eq!(hits.len(), 2);
    assert!(
        hits.iter()
            .any(|hit| hit.source_path == first.to_string_lossy())
    );
    assert!(
        hits.iter()
            .any(|hit| hit.source_path == second.to_string_lossy())
    );
    Ok(())
}
