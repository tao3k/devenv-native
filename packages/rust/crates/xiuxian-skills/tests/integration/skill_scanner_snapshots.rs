//! Snapshot contracts for `SkillScanner` metadata and strict validation outputs.
//!
//! Uses Insta for snapshot testing.

use std::fs;

use crate::json_support::{canonicalize_json, sanitize_json_paths};
use crate::path_sanitization::sanitize_path;
use crate::read_fixture_support::read_fixture;
use crate::structure::default_structure;
use crate::write_fixture_support::write_fixture_file;
use tempfile::TempDir;
use xiuxian_skills::{SkillMetadata, SkillScanner, ToolAnnotations, ToolRecord};

// ============================================================================
// Snapshot: Skill Metadata Parse
// ============================================================================

#[test]
fn snapshot_skill_metadata_parse_contract() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("auditor_neuron");
    fs::create_dir_all(&skill_path)?;
    let content = read_fixture("skill_scanner_snapshots/auditor_neuron_parse/SKILL.md");

    let metadata = scanner.parse_skill_md(content.as_str(), &skill_path)?;
    let metadata = canonicalize_json(serde_json::to_value(metadata)?);
    insta::assert_json_snapshot!("parsed_metadata", metadata);
    Ok(())
}

// ============================================================================
// Snapshot: Structure Validation Summary
// ============================================================================

#[test]
fn snapshot_structure_validation_summary_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("auditor_neuron");
    fs::create_dir_all(skill_path.join("scripts"))?;
    fs::create_dir_all(skill_path.join("references"))?;
    fs::create_dir_all(skill_path.join("scratch"))?;
    write_fixture_file(
        skill_path.join("SKILL.md").as_path(),
        "skill_scanner_snapshots/auditor_neuron_base/SKILL.md",
    )?;

    let structure = default_structure();
    let report = SkillScanner::validate_structure_report(&skill_path, &structure);
    let sanitized_issues: Vec<String> = report
        .issues
        .iter()
        .map(|issue| sanitize_path(issue, &skill_path))
        .collect();

    let summary = serde_json::json!({
        "valid": report.valid,
        "issues": sanitized_issues,
    });

    insta::assert_json_snapshot!("structure_report_summary", canonicalize_json(summary));
    Ok(())
}

// ============================================================================
// Snapshot: Missing Type Error
// ============================================================================

#[test]
fn snapshot_missing_type_error_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("auditor_neuron");
    fs::create_dir_all(skill_path.join("references"))?;
    write_fixture_file(
        skill_path.join("SKILL.md").as_path(),
        "skill_scanner_snapshots/auditor_neuron_base/SKILL.md",
    )?;
    write_fixture_file(
        skill_path.join("references/teacher.md").as_path(),
        "skill_scanner_snapshots/missing_type/teacher.md",
    )?;

    let scanner = SkillScanner::new();
    let structure = default_structure();
    let Err(error) = scanner.scan_skill(&skill_path, Some(&structure)) else {
        return Err(std::io::Error::other("expected error").into());
    };

    let normalized = sanitize_path(&error.to_string(), &skill_path);
    insta::assert_snapshot!("missing_type_error", normalized);
    Ok(())
}

// ============================================================================
// Snapshot: Structure Validation Issues
// ============================================================================

#[test]
fn snapshot_structure_validation_issues_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("auditor_neuron");
    fs::create_dir_all(skill_path.join("scripts"))?;
    fs::create_dir_all(skill_path.join("references"))?;
    fs::create_dir_all(skill_path.join("scratch"))?;
    write_fixture_file(
        skill_path.join("SKILL.md").as_path(),
        "skill_scanner_snapshots/auditor_neuron_base/SKILL.md",
    )?;

    let structure = default_structure();
    let report = SkillScanner::validate_structure_report(&skill_path, &structure);

    let sanitized_issues: Vec<String> = report
        .issues
        .iter()
        .map(|issue| sanitize_path(issue, &skill_path))
        .collect();

    let sanitized_issues = canonicalize_json(serde_json::to_value(sanitized_issues)?);
    insta::assert_json_snapshot!("structure_report_issues", sanitized_issues);
    Ok(())
}

// ============================================================================
// Snapshot: Scan All Multiple Skills
// ============================================================================

#[test]
fn snapshot_scan_all_multiple_skills_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let writer_path = skills_dir.join("writer");
    fs::create_dir_all(&writer_path)?;
    write_fixture_file(
        writer_path.join("SKILL.md").as_path(),
        "skill_scanner_snapshots/scan_all/writer/SKILL.md",
    )?;

    let git_path = skills_dir.join("git");
    fs::create_dir_all(&git_path)?;
    write_fixture_file(
        git_path.join("SKILL.md").as_path(),
        "skill_scanner_snapshots/scan_all/git/SKILL.md",
    )?;

    let scanner = SkillScanner::new();
    let mut metadatas = scanner.scan_all(&skills_dir, None)?;
    metadatas.sort_by(|left, right| left.skill_name.cmp(&right.skill_name));

    let metadatas = canonicalize_json(serde_json::to_value(metadatas)?);
    insta::assert_json_snapshot!("scan_all_multiple_skills", metadatas);
    Ok(())
}

// ============================================================================
// Snapshot: Canonical Payload Tool Reference
// ============================================================================

#[test]
fn snapshot_canonical_payload_tool_reference_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("researcher");
    fs::create_dir_all(&skill_path)?;
    fs::create_dir_all(skill_path.join("references"))?;
    write_fixture_file(
        skill_path.join("SKILL.md").as_path(),
        "skill_scanner_snapshots/canonical_payload/researcher/SKILL.md",
    )?;
    write_fixture_file(
        skill_path
            .join("references/run_research_graph.md")
            .as_path(),
        "skill_scanner_snapshots/canonical_payload/researcher/references/run_research_graph.md",
    )?;

    let metadata = SkillMetadata {
        skill_name: "researcher".to_string(),
        version: "1.0.0".to_string(),
        description: "Research skill".to_string(),
        routing_keywords: vec!["research".to_string()],
        authors: vec![],
        intents: vec![],
        require_refs: vec![],
        repository: String::new(),
        permissions: vec![],
    };

    let tools = vec![ToolRecord {
        tool_name: "researcher.run_research_graph".to_string(),
        description: "Run the graph".to_string(),
        skill_name: "researcher".to_string(),
        file_path: "researcher/scripts/commands.py".to_string(),
        function_name: "run_research_graph".to_string(),
        execution_mode: "async".to_string(),
        keywords: vec!["research".to_string()],
        intents: vec![],
        file_hash: "abc".to_string(),
        input_schema: "{}".to_string(),
        docstring: String::new(),
        category: "research".to_string(),
        annotations: ToolAnnotations::default(),
        parameters: vec![],
        skill_tools_refers: vec![],
        resource_uri: String::new(),
    }];

    let scanner = SkillScanner::new();
    let payload = scanner.build_canonical_payload(metadata, &tools, &skill_path);
    let mut json_value = serde_json::to_value(payload)?;
    sanitize_json_paths(&mut json_value, &skill_path);
    let canonical = canonicalize_json(json_value);

    insta::assert_json_snapshot!("canonical_payload_tool_reference", canonical);
    Ok(())
}
