//! Evidence parser ownership for Rust-owned `SearchStrategyFlow` probes.

pub(crate) fn search_strategy_flow_evidence_edge_kinds(path: &str) -> Vec<String> {
    let provider = language_provider_for_path(path);
    vec![
        "parser-priority:language-provider".to_owned(),
        "provider-boundary:agent-semantic-protocols/languages".to_owned(),
        format!("effective-parser:{provider}"),
    ]
}

fn language_provider_for_path(path: &str) -> &'static str {
    if path.ends_with(".jl") {
        "asp:julia"
    } else if path.ends_with(".ts") || path.ends_with(".tsx") {
        "asp:typescript"
    } else if path.ends_with(".py") {
        "asp:python"
    } else {
        "asp:rust"
    }
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/search_strategy_flow_evidence.rs"]
mod tests;
