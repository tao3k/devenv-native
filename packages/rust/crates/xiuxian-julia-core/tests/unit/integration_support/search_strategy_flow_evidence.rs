use super::search_strategy_flow_evidence_edge_kinds;

#[test]
fn rust_uses_native_effective_parser_above_general_ast_baseline() {
    let kinds = search_strategy_flow_evidence_edge_kinds("src/lib.rs");

    assert!(kinds.contains(&"parser-priority:local-override".to_owned()));
    assert!(kinds.contains(&"native-parser-override".to_owned()));
    assert!(kinds.contains(&"effective-parser:rust-lang-parser".to_owned()));
    assert!(kinds.contains(&"general-ast-baseline".to_owned()));
    assert!(kinds.contains(&"baseline-parser:xiuxian-ast:rust".to_owned()));
}

#[test]
fn julia_uses_plugin_effective_parser_when_general_ast_has_no_language() {
    let kinds = search_strategy_flow_evidence_edge_kinds("src/SearchStrategyFlow.jl");

    assert!(kinds.contains(&"parser-priority:local-override".to_owned()));
    assert!(kinds.contains(&"plugin-parser-override".to_owned()));
    assert!(kinds.contains(&"effective-parser:julia-lang-parser".to_owned()));
    assert!(!kinds.contains(&"general-ast-baseline".to_owned()));
}

#[test]
fn general_ast_baseline_is_effective_for_supported_non_override_languages() {
    let kinds = search_strategy_flow_evidence_edge_kinds("src/app.tsx");

    assert!(kinds.contains(&"parser-priority:general-baseline".to_owned()));
    assert!(kinds.contains(&"effective-parser:xiuxian-ast:typescript".to_owned()));
    assert!(kinds.contains(&"general-ast-baseline".to_owned()));
    assert!(kinds.contains(&"baseline-parser:xiuxian-ast:typescript".to_owned()));
}
