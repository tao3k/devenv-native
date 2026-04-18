use std::io;

use tempfile::TempDir;
use xiuxian_skills::{SkillScanner, ToolsScanner};

use super::{TestResult, create_scripts_dir, scan_scripts, write_script};

#[test]
fn test_enriched_tool_record_json_serialization() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "test.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="get_data", description="Get test data", category="test")
def get_data(param: str) -> str:
    '''Test docstring.'''
    return "ok"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);

    let json = serde_json::to_string(&tools[0])?;
    let deserialized: xiuxian_skills::ToolRecord = serde_json::from_str(&json)?;

    assert_eq!(deserialized.description, "Get test data");
    assert_eq!(deserialized.category, "test");
    assert_eq!(deserialized.parameters, vec!["param"]);
    assert!(deserialized.annotations.read_only);
    Ok(())
}

#[test]
fn test_input_schema_generation() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "query.py",
        r#"
@skill_command(name="query")
def query_data(user_id: str, limit: int = 10) -> list:
    '''Query data from database.'''
    return []
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);

    let schema: serde_json::Value = serde_json::from_str(&tools[0].input_schema)?;
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"].is_object());
    assert!(schema["required"].is_array());

    let props = schema["properties"]
        .as_object()
        .ok_or_else(|| io::Error::other("schema.properties should be object"))?;
    assert!(props.contains_key("user_id"));
    assert!(props.contains_key("limit"));

    let required = schema["required"]
        .as_array()
        .ok_or_else(|| io::Error::other("schema.required should be array"))?;
    assert!(required.contains(&serde_json::json!("user_id")));
    Ok(())
}

#[test]
fn test_input_schema_empty_for_no_params() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "status.py",
        r#"
@skill_command(name="status")
def get_status() -> dict:
    '''Get system status.'''
    return {}
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    let schema: serde_json::Value = serde_json::from_str(&tools[0].input_schema)?;

    assert_eq!(schema["type"], "object");
    assert_eq!(
        schema["properties"]
            .as_object()
            .ok_or_else(|| io::Error::other("schema.properties should be object"))?
            .len(),
        0
    );
    assert!(
        schema["required"]
            .as_array()
            .ok_or_else(|| io::Error::other("schema.required should be array"))?
            .is_empty()
    );
    Ok(())
}

#[test]
fn test_index_tool_entry_includes_category_and_schema() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("test_skill");
    let scripts_dir = create_scripts_dir(&temp_dir, "test_skill")?;

    std::fs::write(
        skill_path.join("SKILL.md"),
        r#"
---
name: "test_skill"
version: "1.0.0"
description: "Test skill"
routing_keywords: ["test"]
---
"#,
    )?;
    write_script(
        &scripts_dir,
        "process.py",
        r#"
@skill_command(name="process", category="processing")
def process_data(input: str) -> str:
    '''Process data.'''
    return input
"#,
    )?;

    let scanner = SkillScanner::new();
    let script_scanner = ToolsScanner::new();
    let metadata = scanner
        .scan_skill(&skill_path, None)?
        .ok_or_else(|| io::Error::other("expected test_skill metadata"))?;
    let scanned_tools = script_scanner.scan_scripts(&scripts_dir, "test_skill", &[], &[])?;

    let entry = scanner.build_index_entry(metadata, &scanned_tools, &skill_path);
    assert_eq!(entry.tools.len(), 1);
    let tool = &entry.tools[0];

    assert_eq!(tool.name, "test_skill.process");
    assert_eq!(tool.category, "processing");
    assert!(!tool.input_schema.is_empty());

    let schema: serde_json::Value = serde_json::from_str(&tool.input_schema)?;
    assert_eq!(schema["type"], "object");
    Ok(())
}
