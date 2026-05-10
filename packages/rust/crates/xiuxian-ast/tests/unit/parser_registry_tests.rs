use xiuxian_ast::{AstParserPriority, AstParserRegistry};

#[test]
fn registry_uses_general_ast_baseline_without_override() {
    let registry = AstParserRegistry::new();

    let resolution = registry.resolve_path("src/app.tsx");

    assert_eq!(resolution.priority, AstParserPriority::GeneralBaseline);
    assert_eq!(resolution.effective_parser, "xiuxian-ast:typescript");
    assert_eq!(
        resolution.baseline_parser.as_deref(),
        Some("xiuxian-ast:typescript")
    );
}

#[test]
fn registry_uses_local_override_above_general_baseline() {
    let registry = AstParserRegistry::new().with_extension_override("rs", "rust-lang-parser");

    let resolution = registry.resolve_path("src/lib.rs");

    assert_eq!(resolution.priority, AstParserPriority::LocalOverride);
    assert_eq!(resolution.effective_parser, "rust-lang-parser");
    assert_eq!(
        resolution.baseline_parser.as_deref(),
        Some("xiuxian-ast:rust")
    );
}

#[test]
fn registry_supports_plugin_override_without_general_baseline() {
    let registry = AstParserRegistry::new().with_extension_override(".jl", "julia-lang-parser");

    let resolution = registry.resolve_path("src/SearchStrategyFlow.jl");

    assert_eq!(resolution.priority, AstParserPriority::LocalOverride);
    assert_eq!(resolution.effective_parser, "julia-lang-parser");
    assert_eq!(resolution.baseline_parser, None);
}

#[test]
fn registry_uses_plain_text_when_no_parser_matches() {
    let registry = AstParserRegistry::new();

    let resolution = registry.resolve_path("notes/source.unknown");

    assert_eq!(resolution.priority, AstParserPriority::PlainText);
    assert_eq!(resolution.effective_parser, "plain-text-parser");
    assert_eq!(resolution.baseline_parser, None);
}
