use super::search_strategy_flow_evidence_edge_kinds;

#[test]
fn rust_uses_language_provider_boundary() {
    let kinds = search_strategy_flow_evidence_edge_kinds("src/lib.rs");

    assert!(kinds.contains(&"parser-priority:language-provider".to_owned()));
    assert!(kinds.contains(&"provider-boundary:agent-semantic-protocols/languages".to_owned()));
    assert!(kinds.contains(&"effective-parser:asp:rust".to_owned()));
    assert!(!kinds.iter().any(|kind| kind.contains("xiuxian-ast")));
}

#[test]
fn julia_uses_language_provider_boundary() {
    let kinds = search_strategy_flow_evidence_edge_kinds("src/SearchStrategyFlow.jl");

    assert!(kinds.contains(&"parser-priority:language-provider".to_owned()));
    assert!(kinds.contains(&"provider-boundary:agent-semantic-protocols/languages".to_owned()));
    assert!(kinds.contains(&"effective-parser:asp:julia".to_owned()));
    assert!(!kinds.contains(&"general-ast-baseline".to_owned()));
}

#[test]
fn typescript_uses_language_provider_boundary() {
    let kinds = search_strategy_flow_evidence_edge_kinds("src/app.tsx");

    assert!(kinds.contains(&"parser-priority:language-provider".to_owned()));
    assert!(kinds.contains(&"provider-boundary:agent-semantic-protocols/languages".to_owned()));
    assert!(kinds.contains(&"effective-parser:asp:typescript".to_owned()));
    assert!(!kinds.iter().any(|kind| kind.contains("xiuxian-ast")));
}
