//! Scenario-based contract tests for xiuxian-types.

use std::error::Error;
use std::path::PathBuf;

use serde_json::{Value, json};
use xiuxian_types::SkillDefinition;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_skill_definition_scenarios() {
    let manifest = manifest_dir();
    let input_path = manifest
        .join("tests")
        .join("fixtures")
        .join("scenarios")
        .join("001_routing_keywords_merge")
        .join("input")
        .join("skill.json");

    let output = run_skill_definition_fixture(&input_path)
        .unwrap_or_else(|error| panic!("skill definition scenario should pass: {error}"));

    assert_eq!(
        output,
        json!({
            "description": "desc",
            "metadata": {
                "routing_keywords": ["alpha", "beta", "gamma"],
            },
            "name": "git",
        })
    );
}

fn run_skill_definition_fixture(input_path: &std::path::Path) -> Result<Value, Box<dyn Error>> {
    let raw = std::fs::read_to_string(input_path)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let def: SkillDefinition = serde_json::from_value(value)?;
    Ok(serde_json::to_value(def)?)
}
