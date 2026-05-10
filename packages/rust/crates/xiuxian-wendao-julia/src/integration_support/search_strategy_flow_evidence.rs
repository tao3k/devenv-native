//! Evidence parser ownership for Rust-owned `SearchStrategyFlow` probes.

use xiuxian_code_intelligence::CodeParserEvidenceRegistry;

pub(crate) fn search_strategy_flow_evidence_edge_kinds(path: &str) -> Vec<String> {
    CodeParserEvidenceRegistry::agent_search_defaults().resolve_path_edge_kinds(path)
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/search_strategy_flow_evidence.rs"]
mod tests;
