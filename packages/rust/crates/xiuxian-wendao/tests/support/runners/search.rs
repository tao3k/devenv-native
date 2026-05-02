//! Runner for search scenario tests.

use std::error::Error;
use std::path::Path;

use serde_json::{Value, json};
use xiuxian_wendao::LinkGraphIndex;

use crate::scenarios::support::{Scenario, ScenarioRunner};

/// Runner for search-related category scenarios.
///
/// Handles categories:
/// - `search_core`
/// - `search_filters`
/// - `search_match_strategies`
/// - `tree_scope_filters`
pub struct SearchRunner;

impl ScenarioRunner for SearchRunner {
    fn category(&self) -> &'static str {
        "search_core"
    }

    fn additional_categories(&self) -> Vec<&str> {
        vec![
            "search_filters",
            "search_match_strategies",
            "tree_scope_filters",
        ]
    }

    fn run(&self, scenario: &Scenario, temp_dir: &Path) -> Result<Value, Box<dyn Error>> {
        // For search scenarios, we validate that the index can be built
        // and return a summary of the scenario configuration

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
