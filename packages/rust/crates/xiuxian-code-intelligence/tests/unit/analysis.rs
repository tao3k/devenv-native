use xiuxian_ast::Lang;
use xiuxian_code_intelligence::{
    CodeLanguageId, SymbolKind, code_semantic_fingerprint,
    code_semantic_fingerprint_language_id_from_identifier,
    code_semantic_fingerprint_language_id_from_path, count_code_pattern_matches,
    count_code_pattern_matches_for_language_id, extract_code_dependency_symbols,
    extract_code_pattern_matches, extract_code_structure_symbols,
    extract_code_structure_symbols_for_language_id, resolve_code_source_files,
    resolve_code_source_files_for_language_id, supports_code_semantic_fingerprint,
};

#[test]
fn extracts_rust_code_structure_symbols_from_skeleton_patterns() {
    let symbols = extract_code_structure_symbols(
        r"
pub struct SearchPlan {
    limit: usize,
}

pub fn build_plan() -> SearchPlan {
    SearchPlan { limit: 10 }
}
",
        Lang::Rust,
    );

    assert!(symbols.iter().any(|symbol| {
        symbol.name == "SearchPlan" && symbol.signature == "pub struct SearchPlan {"
    }));
    assert!(symbols.iter().any(|symbol| {
        symbol.name == "build_plan" && symbol.signature == "pub fn build_plan() -> SearchPlan {"
    }));
}

#[test]
fn extracts_toml_code_structure_symbols_with_name_capture() {
    let symbols = extract_code_structure_symbols(
        r#"
[repo.alpha]
path = "docs"
"#,
        Lang::Toml,
    );

    assert!(symbols.iter().any(|symbol| symbol.name == "repo.alpha"));
}

#[test]
fn extracts_code_pattern_matches_with_captures() {
    let matches =
        extract_code_pattern_matches("pub fn search() {}\n", "pub fn $NAME", Lang::Rust, None);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].text, "pub fn search() {}");
    assert_eq!(matches[0].line_start, 1);
    assert_eq!(
        matches[0].captures.get("NAME").map(String::as_str),
        Some("search")
    );
}

#[test]
fn counts_code_pattern_matches_for_observation_checks() {
    let count = count_code_pattern_matches(
        "pub fn search() {}\npub fn index() {}\n",
        "pub fn $NAME",
        Lang::Rust,
    )
    .unwrap_or_else(|error| panic!("pattern should scan: {error}"));

    assert_eq!(count, 2);
}

#[test]
fn count_code_pattern_matches_returns_zero_for_misses() {
    let count = count_code_pattern_matches("pub fn search() {}\n", "pub struct $NAME", Lang::Rust)
        .unwrap_or_else(|error| panic!("pattern should scan: {error}"));

    assert_eq!(count, 0);
}

#[test]
fn extracts_dependency_symbols_with_rust_indexing_kinds() {
    let symbols = extract_code_dependency_symbols(
        r#"
pub struct SearchPlan;
pub enum SearchMode {}
pub trait SearchStage {}
pub fn build_plan() {}
impl SearchPlan {}
pub mod planner;
pub type SearchId = String;
pub const DEFAULT_LIMIT: usize = 10;
pub static CACHE_NAME: &str = "search";
"#,
        &CodeLanguageId::from("rust"),
    );

    let expected = [
        ("SearchPlan", SymbolKind::Struct),
        ("SearchMode", SymbolKind::Enum),
        ("SearchStage", SymbolKind::Trait),
        ("build_plan", SymbolKind::Function),
        ("SearchPlan", SymbolKind::Impl),
        ("planner", SymbolKind::Module),
        ("SearchId", SymbolKind::TypeAlias),
        ("DEFAULT_LIMIT", SymbolKind::Const),
        ("CACHE_NAME", SymbolKind::Static),
    ];

    for (name, kind) in expected {
        assert!(
            symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.kind == kind),
            "missing {kind:?} {name}"
        );
    }
}

#[test]
fn extracts_structure_and_counts_matches_with_language_ids() {
    let language_id = CodeLanguageId::from("rust");
    let content = "pub fn search() {}\npub fn index() {}\n";

    let symbols = extract_code_structure_symbols_for_language_id(content, &language_id);
    let count = count_code_pattern_matches_for_language_id(content, "pub fn $NAME", &language_id)
        .unwrap_or_else(|error| panic!("pattern should scan: {error}"));

    assert_eq!(symbols.len(), 2);
    assert_eq!(count, 2);
}

#[test]
fn extracts_dependency_symbols_with_python_indexing_kinds() {
    let symbols = extract_code_dependency_symbols(
        r"
class SearchPlan:
    pass

async def build_async():
    pass

def build_sync():
    pass
",
        &CodeLanguageId::from("python"),
    );

    assert!(
        symbols
            .iter()
            .any(|symbol| symbol.name == "SearchPlan" && symbol.kind == SymbolKind::Struct)
    );
    assert!(
        symbols
            .iter()
            .any(|symbol| symbol.name == "build_async" && symbol.kind == SymbolKind::Function)
    );
    assert!(
        symbols
            .iter()
            .any(|symbol| symbol.name == "build_sync" && symbol.kind == SymbolKind::Function)
    );
}

#[test]
fn resolves_shallow_source_files_for_language_extensions() {
    let tempdir =
        tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    let rust_path = tempdir.path().join("lib.rs");
    let text_path = tempdir.path().join("notes.txt");
    let nested_dir = tempdir.path().join("nested");
    let nested_path = nested_dir.join("mod.rs");
    std::fs::write(&rust_path, "pub fn search() {}\n")
        .unwrap_or_else(|error| panic!("rust fixture should write: {error}"));
    std::fs::write(&text_path, "ignore\n")
        .unwrap_or_else(|error| panic!("text fixture should write: {error}"));
    std::fs::create_dir(&nested_dir)
        .unwrap_or_else(|error| panic!("nested fixture dir should write: {error}"));
    std::fs::write(&nested_path, "pub fn nested() {}\n")
        .unwrap_or_else(|error| panic!("nested fixture should write: {error}"));

    let sources = resolve_code_source_files(&[tempdir.path()], Lang::Rust);
    let language_id_sources =
        resolve_code_source_files_for_language_id(&[tempdir.path()], &CodeLanguageId::from("rust"));

    assert_eq!(sources.len(), 1);
    assert_eq!(sources, language_id_sources);
    assert_eq!(sources[0].path, rust_path.display().to_string());
    assert_eq!(sources[0].content, "pub fn search() {}\n");
}

#[test]
fn resolves_and_builds_generic_code_semantic_fingerprints() {
    let path_language =
        code_semantic_fingerprint_language_id_from_path(std::path::Path::new("src/lib.rs"))
            .unwrap_or_else(|| panic!("rust path should support semantic fingerprints"));
    let parser_language = code_semantic_fingerprint_language_id_from_identifier("rust-lang-parser")
        .unwrap_or_else(|| panic!("rust parser id should support semantic fingerprints"));

    assert_eq!(path_language, parser_language);
    assert!(supports_code_semantic_fingerprint(&path_language));

    let first = code_semantic_fingerprint("pub fn search() {}\n", &path_language)
        .unwrap_or_else(|| panic!("fingerprint should be produced"));
    let second = code_semantic_fingerprint(
        "pub fn search() { println!(\"ignored\"); }\n",
        &path_language,
    )
    .unwrap_or_else(|| panic!("fingerprint should be produced"));

    assert_eq!(first, second);
}
