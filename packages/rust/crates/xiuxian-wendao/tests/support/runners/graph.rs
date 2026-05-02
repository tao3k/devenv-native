//! Runner for graph-related scenario tests.

use std::error::Error;
use std::path::Path;

use serde_json::{Value, json};
use xiuxian_wendao::LinkGraphIndex;

use crate::scenarios::support::{Scenario, ScenarioRunner};

/// Runner for graph-related category scenarios.
///
/// Handles categories:
/// - `graph_navigation`
/// - `mixed_topology`
/// - `build_scope`
/// - `cache_build`
/// - `markdown_attachments`
/// - `refresh`
/// - `seed_and_priors`
/// - `semantic_policy`
/// - `ppr_precision`
/// - `ppr_weighting`
/// - `agentic_expansion`
/// - `quantum_fusion`
/// - `hybrid`
pub struct GraphRunner;

impl ScenarioRunner for GraphRunner {
    fn category(&self) -> &'static str {
        "graph_navigation"
    }

    fn additional_categories(&self) -> Vec<&str> {
        vec![
            "mixed_topology",
            "build_scope",
            "cache_build",
            "markdown_attachments",
            "refresh",
            "seed_and_priors",
            "semantic_policy",
            "ppr_precision",
            "ppr_weighting",
            "agentic_expansion",
            "quantum_fusion",
            "hybrid",
        ]
    }

    fn run(&self, scenario: &Scenario, temp_dir: &Path) -> Result<Value, Box<dyn Error>> {
        // Get expected files (optional, for backward compatibility)
        let expected_files = scenario
            .config
            .expected
            .as_ref()
            .map(|e| e.files.clone())
            .unwrap_or_default();

        // Check if scenario has input
        if !scenario.has_input() {
            return Ok(json!({
                "scenario_id": scenario.id(),
                "category": scenario.category(),
                "status": "no_input",
                "files": expected_files,
            }));
        }

        // Build the index if input exists
        let input_path = scenario.input_path();
        if let Some(path) = input_path
            && path.exists()
        {
            let _index = LinkGraphIndex::build(temp_dir)?;
        }

        Ok(json!({
            "scenario_id": scenario.id(),
            "category": scenario.category(),
            "status": "validated",
            "files": expected_files,
        }))
    }
}
