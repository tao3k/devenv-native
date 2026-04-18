//! Snapshot-backed full workflow integration contracts for `xiuxian-skills`.
//!
//! Uses Insta for snapshot testing.

use std::fs;

use crate::json_support::canonicalize_json;
use crate::write_fixture_support::write_fixture_file;
use tempfile::TempDir;
use xiuxian_skills::VERSION;
use xiuxian_skills::{SkillScanner, ToolsScanner};

#[test]
fn test_version_constant() {
    assert!(!VERSION.is_empty());
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
}

#[test]
fn snapshot_full_scan_workflow_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let writer_path = skills_dir.join("writer");
    fs::create_dir_all(writer_path.join("scripts"))?;
    write_fixture_file(
        writer_path.join("SKILL.md").as_path(),
        "full_workflow_snapshots/full_scan/writer/SKILL.md",
    )?;
    write_fixture_file(
        writer_path.join("scripts/text.py").as_path(),
        "full_workflow_snapshots/full_scan/writer/scripts/text.py",
    )?;

    let git_path = skills_dir.join("git");
    fs::create_dir_all(&git_path)?;
    write_fixture_file(
        git_path.join("SKILL.md").as_path(),
        "full_workflow_snapshots/full_scan/git/SKILL.md",
    )?;

    let skill_scanner = SkillScanner::new();
    let mut metadatas = skill_scanner.scan_all(&skills_dir, None)?;
    metadatas.sort_by(|left, right| left.skill_name.cmp(&right.skill_name));

    let tools_scanner = ToolsScanner::new();
    let writer_metadata = metadatas
        .iter()
        .find(|metadata| metadata.skill_name == "writer")
        .ok_or_else(|| std::io::Error::other("writer metadata should exist"))?;
    let writer_tools = tools_scanner.scan_scripts(
        &writer_path.join("scripts"),
        "writer",
        &writer_metadata.routing_keywords,
        &writer_metadata.intents,
    )?;

    let mut tool_names = writer_tools
        .iter()
        .map(|tool| tool.tool_name.clone())
        .collect::<Vec<_>>();
    tool_names.sort();
    let mut writer_keywords = writer_tools
        .iter()
        .flat_map(|tool| tool.keywords.iter().cloned())
        .collect::<Vec<_>>();
    writer_keywords.sort();
    writer_keywords.dedup();

    let metadata_projection = metadatas
        .iter()
        .map(|metadata| {
            serde_json::json!({
                "skill_name": metadata.skill_name,
                "version": metadata.version,
                "description": metadata.description,
                "routing_keywords": metadata.routing_keywords,
                "intents": metadata.intents
            })
        })
        .collect::<Vec<_>>();

    let snapshot = serde_json::json!({
        "metadata_projection": metadata_projection,
        "writer_tool_count": writer_tools.len(),
        "writer_tool_names": tool_names,
        "writer_keyword_checks": {
            "contains_write": writer_keywords.contains(&"write".to_string()),
            "contains_edit": writer_keywords.contains(&"edit".to_string()),
            "contains_polish": writer_keywords.contains(&"polish".to_string())
        }
    });

    insta::assert_json_snapshot!("full_scan_workflow", canonicalize_json(snapshot));
    Ok(())
}

#[test]
fn snapshot_scanner_reports_duplicate_tools_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("skills/test");
    fs::create_dir_all(skill_path.join("scripts"))?;
    write_fixture_file(
        skill_path.join("SKILL.md").as_path(),
        "full_workflow_snapshots/duplicate_tools/test/SKILL.md",
    )?;
    write_fixture_file(
        skill_path.join("scripts/commands.py").as_path(),
        "full_workflow_snapshots/duplicate_tools/test/scripts/commands.py",
    )?;
    write_fixture_file(
        skill_path.join("scripts/more_commands.py").as_path(),
        "full_workflow_snapshots/duplicate_tools/test/scripts/more_commands.py",
    )?;

    let tools_scanner = ToolsScanner::new();
    let tools = tools_scanner.scan_scripts(
        &skill_path.join("scripts"),
        "test",
        &["test".to_string()],
        &[],
    )?;

    let mut tool_names = tools
        .iter()
        .map(|tool| tool.tool_name.clone())
        .collect::<Vec<_>>();
    tool_names.sort();
    let unique_count = tool_names
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let file_hashes_unique = tools
        .iter()
        .map(|tool| tool.file_hash.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    let snapshot = serde_json::json!({
        "tool_count": tools.len(),
        "tool_names": tool_names,
        "unique_tool_name_count": unique_count,
        "unique_file_hash_count": file_hashes_unique
    });

    insta::assert_json_snapshot!("duplicate_tools", canonicalize_json(snapshot));
    Ok(())
}

#[test]
fn snapshot_same_function_name_different_skills_contract() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let skill1_path = skills_dir.join("skill1");
    fs::create_dir_all(skill1_path.join("scripts"))?;
    write_fixture_file(
        skill1_path.join("SKILL.md").as_path(),
        "full_workflow_snapshots/cross_skill/skill1/SKILL.md",
    )?;
    write_fixture_file(
        skill1_path.join("scripts/main.py").as_path(),
        "full_workflow_snapshots/cross_skill/skill1/scripts/main.py",
    )?;

    let skill2_path = skills_dir.join("skill2");
    fs::create_dir_all(skill2_path.join("scripts"))?;
    write_fixture_file(
        skill2_path.join("SKILL.md").as_path(),
        "full_workflow_snapshots/cross_skill/skill2/SKILL.md",
    )?;
    write_fixture_file(
        skill2_path.join("scripts/main.py").as_path(),
        "full_workflow_snapshots/cross_skill/skill2/scripts/main.py",
    )?;

    let tools_scanner = ToolsScanner::new();
    let mut skill1_tools = tools_scanner
        .scan_scripts(
            &skill1_path.join("scripts"),
            "skill1",
            &["s1".to_string()],
            &[],
        )?
        .into_iter()
        .map(|tool| tool.tool_name)
        .collect::<Vec<_>>();
    skill1_tools.sort();
    let mut skill2_tools = tools_scanner
        .scan_scripts(
            &skill2_path.join("scripts"),
            "skill2",
            &["s2".to_string()],
            &[],
        )?
        .into_iter()
        .map(|tool| tool.tool_name)
        .collect::<Vec<_>>();
    skill2_tools.sort();

    let snapshot = serde_json::json!({
        "skill1_tools": skill1_tools,
        "skill2_tools": skill2_tools
    });

    insta::assert_json_snapshot!("cross_skill_same_function", canonicalize_json(snapshot));
    Ok(())
}
