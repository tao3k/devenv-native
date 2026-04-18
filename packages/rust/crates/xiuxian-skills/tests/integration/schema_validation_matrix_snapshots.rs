//! Snapshot contracts for schema validation and tool name formatting.
//!
//! Uses Insta for snapshot testing.

use std::fs;

use crate::json_support::canonicalize_json;
use crate::read_fixture_support::read_fixture;
use tempfile::TempDir;
use xiuxian_skills::{IndexToolEntry, SkillIndexEntry, ToolsScanner};

// ============================================================================
// Snapshot: Tool Name Format Matrix
// ============================================================================

#[test]
fn snapshot_tool_name_format_matrix_contract() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = ToolsScanner::new();
    let temp_dir = TempDir::new()?;
    let matrix_root = temp_dir.path().join("scripts");
    fs::create_dir_all(&matrix_root)?;

    let cases = [
        ("named", "git", "scripts/named/commit.py"),
        ("multiple", "git", "scripts/multiple/commit.py"),
        ("underscore", "my_skill", "scripts/underscore/utils.py"),
        (
            "function_fallback",
            "test",
            "scripts/function_fallback/hello.py",
        ),
        ("path_like", "my_skill", "scripts/path_like/cmd.py"),
    ];

    let mut rows = Vec::new();
    for (case_id, skill_name, fixture_rel) in cases {
        let scripts_dir = matrix_root.join(case_id);
        fs::create_dir_all(&scripts_dir)?;

        let filename = fixture_rel
            .rsplit('/')
            .next()
            .ok_or_else(|| std::io::Error::other("fixture file name missing"))?;

        let content = read_fixture(&format!("schema_validation_matrix/{fixture_rel}"));
        fs::write(scripts_dir.join(filename), content)?;

        let mut tool_names = scanner
            .scan_scripts(&scripts_dir, skill_name, &[], &[])?
            .into_iter()
            .map(|tool| tool.tool_name)
            .collect::<Vec<_>>();
        tool_names.sort();

        let repeated_prefix = format!("{skill_name}.{skill_name}.");
        let invalid_part_count = tool_names
            .iter()
            .filter(|name| name.split('.').count() != 2)
            .count();
        let repeated_prefix_count = tool_names
            .iter()
            .filter(|name| name.starts_with(repeated_prefix.as_str()))
            .count();
        let first_part_mismatch_count = tool_names
            .iter()
            .filter(|name| name.split('.').next() != Some(skill_name))
            .count();

        rows.push(serde_json::json!({
            "case": case_id,
            "skill_name": skill_name,
            "tool_count": tool_names.len(),
            "tool_names": tool_names,
            "invalid_part_count": invalid_part_count,
            "repeated_prefix_count": repeated_prefix_count,
            "first_part_mismatch_count": first_part_mismatch_count
        }));
    }

    let rows = canonicalize_json(serde_json::to_value(rows)?);
    insta::assert_json_snapshot!("tool_name_format_matrix", rows);
    Ok(())
}

// ============================================================================
// Snapshot: Skill Index JSON Schema
// ============================================================================

#[test]
fn snapshot_skill_index_json_schema_contract() {
    let mut entry = SkillIndexEntry::new(
        "git".to_string(),
        "Git skill".to_string(),
        "1.0.0".to_string(),
        "skills/git".to_string(),
    );

    entry.add_tool(IndexToolEntry {
        name: "git.commit".to_string(),
        description: "Create commit".to_string(),
        category: String::new(),
        input_schema: String::new(),
        file_hash: String::new(),
    });

    let tool_names = entry
        .tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();

    let snapshot = serde_json::json!({
        "name": entry.name,
        "version": entry.version,
        "tool_count": tool_names.len(),
        "tool_names": tool_names
    });

    insta::assert_json_snapshot!("skill_index_schema", canonicalize_json(snapshot));
}
