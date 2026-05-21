use xiuxian_ast::Lang;
use xiuxian_code_intelligence::{
    code_pattern_signature_line, first_code_signature_line, score_code_structure_query,
};

#[test]
fn first_code_signature_line_trims_the_first_line() {
    assert_eq!(
        first_code_signature_line("  pub fn search_documents() {}\n// body"),
        "pub fn search_documents() {}"
    );
}

#[test]
fn code_pattern_signature_line_preserves_c_like_body_placeholder() {
    assert_eq!(
        code_pattern_signature_line("pub fn search_documents() { body(); }\n", Lang::Rust),
        "pub fn search_documents() { $$$BODY }"
    );
}

#[test]
fn code_pattern_signature_line_preserves_python_like_body_placeholder() {
    assert_eq!(
        code_pattern_signature_line("def search_documents():\n    pass\n", Lang::Python),
        "def search_documents(): $$$BODY"
    );
}

#[test]
fn score_code_structure_query_prefers_exact_symbol_names() {
    assert_eq!(
        score_code_structure_query(
            Some("search_documents"),
            "src/search.rs",
            "search_documents",
            "pub fn search_documents() {}",
        ),
        Some(1.0)
    );
}

#[test]
fn score_code_structure_query_keeps_path_matches_and_rejects_misses() {
    assert_eq!(
        score_code_structure_query(
            Some("repo"),
            "src/repo_search/ast.rs",
            "build_index",
            "fn build_index()",
        ),
        Some(0.84)
    );
    assert_eq!(
        score_code_structure_query(
            Some("missing"),
            "src/repo_search/ast.rs",
            "build_index",
            "fn build_index()",
        ),
        None
    );
}
