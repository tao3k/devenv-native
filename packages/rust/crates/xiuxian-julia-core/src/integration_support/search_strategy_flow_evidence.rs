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
    match path.rsplit_once('.').map(|(_, extension)| extension) {
        Some("jl") => "asp:julia",
        Some("md" | "markdown") => "asp:markdown",
        Some("py") => "asp:python",
        Some("toml") => "asp:toml",
        Some("ts" | "tsx") => "asp:typescript",
        _ => "asp:rust",
    }
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/search_strategy_flow_evidence.rs"]
mod tests;
