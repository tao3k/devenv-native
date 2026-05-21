use xiuxian_code_intelligence::{
    CodeParserEvidenceRegistry, CodeParserPriority, code_language_id_from_path,
    normalize_code_language_identifier, supported_code_language_id_from_path,
};

#[test]
fn rust_uses_native_effective_parser_above_general_ast_baseline() {
    let evidence = CodeParserEvidenceRegistry::agent_search_defaults().resolve_path("src/lib.rs");

    assert_eq!(evidence.priority, CodeParserPriority::LocalOverride);
    assert_eq!(evidence.effective_parser, "rust-lang-parser");
    assert_eq!(
        evidence.baseline_parser.as_deref(),
        Some("xiuxian-ast:rust")
    );
    assert!(
        evidence
            .edge_kinds
            .contains(&"native-parser-override".to_owned())
    );
    assert!(
        evidence
            .edge_kinds
            .contains(&"baseline-parser:xiuxian-ast:rust".to_owned())
    );
}

#[test]
fn julia_uses_plugin_effective_parser_without_general_ast_baseline() {
    let evidence = CodeParserEvidenceRegistry::agent_search_defaults()
        .resolve_path("src/SearchStrategyFlow.jl");

    assert_eq!(evidence.priority, CodeParserPriority::LocalOverride);
    assert_eq!(evidence.effective_parser, "julia-lang-parser");
    assert_eq!(evidence.baseline_parser, None);
    assert!(
        evidence
            .edge_kinds
            .contains(&"plugin-parser-override".to_owned())
    );
    assert!(
        !evidence
            .edge_kinds
            .contains(&"general-ast-baseline".to_owned())
    );
}

#[test]
fn typescript_keeps_general_ast_baseline_when_no_override_exists() {
    let evidence = CodeParserEvidenceRegistry::agent_search_defaults().resolve_path("src/app.tsx");

    assert_eq!(evidence.priority, CodeParserPriority::GeneralBaseline);
    assert_eq!(evidence.effective_parser, "xiuxian-ast:typescript");
    assert_eq!(
        evidence.baseline_parser.as_deref(),
        Some("xiuxian-ast:typescript")
    );
    assert!(
        evidence
            .edge_kinds
            .contains(&"parser-priority:general-baseline".to_owned())
    );
}

#[test]
fn normalizes_parser_identifiers_into_language_ids() {
    assert_eq!(normalize_code_language_identifier("TS"), "typescript");
    assert_eq!(normalize_code_language_identifier("md"), "markdown");
    assert_eq!(
        normalize_code_language_identifier("julia-code-parser"),
        "julia"
    );
    assert_eq!(
        normalize_code_language_identifier("markdown-lang-parser"),
        "markdown"
    );
    assert_eq!(
        normalize_code_language_identifier("custom-parser"),
        "custom-parser"
    );
}

#[test]
fn resolves_code_language_ids_from_paths() {
    assert_eq!(
        code_language_id_from_path(std::path::Path::new("src/lib.rs")),
        Some("rust")
    );
    assert_eq!(
        supported_code_language_id_from_path(std::path::Path::new("src/lib.rs")),
        Some("rust")
    );
}
