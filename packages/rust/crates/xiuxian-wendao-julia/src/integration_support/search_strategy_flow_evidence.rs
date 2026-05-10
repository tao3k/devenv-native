//! Evidence parser ownership for Rust-owned `SearchStrategyFlow` probes.

use std::sync::OnceLock;

use xiuxian_ast::{AstParserPriority, AstParserRegistry, AstParserResolution};

pub(crate) fn search_strategy_flow_evidence_edge_kinds(path: &str) -> Vec<String> {
    let resolution = search_strategy_flow_parser_registry().resolve_path(path);
    let mut kinds = edge_kinds_for_resolution(&resolution);
    kinds.sort();
    kinds.dedup();
    kinds
}

fn search_strategy_flow_parser_registry() -> &'static AstParserRegistry {
    static REGISTRY: OnceLock<AstParserRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        AstParserRegistry::new()
            .with_extension_override("md", "markdown-lang-parser")
            .with_extension_override("markdown", "markdown-lang-parser")
            .with_extension_override("rs", "rust-lang-parser")
            .with_extension_override("jl", "julia-lang-parser")
            .with_extension_override("mo", "modelica-lang-parser")
    })
}

fn edge_kinds_for_resolution(resolution: &AstParserResolution) -> Vec<String> {
    let mut kinds = match resolution.priority {
        AstParserPriority::LocalOverride => vec![
            "parser-priority:local-override".to_owned(),
            format!("effective-parser:{}", resolution.effective_parser),
        ],
        AstParserPriority::GeneralBaseline => vec![
            "parser-priority:general-baseline".to_owned(),
            format!("effective-parser:{}", resolution.effective_parser),
        ],
        AstParserPriority::PlainText => vec![
            "parser-priority:plain-text".to_owned(),
            format!("effective-parser:{}", resolution.effective_parser),
        ],
    };

    if let Some(baseline) = resolution.baseline_parser.as_deref() {
        kinds.push("general-ast-baseline".to_owned());
        kinds.push(format!("baseline-parser:{baseline}"));
    }

    if resolution.effective_parser == "markdown-lang-parser"
        || resolution.effective_parser == "rust-lang-parser"
    {
        kinds.push("native-parser-override".to_owned());
    }
    if resolution.effective_parser == "julia-lang-parser"
        || resolution.effective_parser == "modelica-lang-parser"
    {
        kinds.push("plugin-parser-override".to_owned());
    }

    kinds
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/search_strategy_flow_evidence.rs"]
mod tests;
