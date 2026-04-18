//! Matrix snapshot contracts for frontmatter diversity and strict validation behavior.
//!
//! Uses Insta for snapshot testing.

use std::fs;

use crate::json_support::canonicalize_json;
use crate::path_sanitization::sanitize_path;
use crate::read_fixture_support::read_fixture;
use crate::structure::default_structure;
use tempfile::TempDir;
use xiuxian_skills::SkillScanner;

// ============================================================================
// Snapshot: Skill Frontmatter Matrix
// ============================================================================

#[test]
fn snapshot_skill_frontmatter_matrix_contract() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let structure = default_structure();
    let temp_dir = TempDir::new()?;
    let matrix_root = temp_dir.path().join("skills");
    fs::create_dir_all(&matrix_root)?;

    let cases = [
        ("valid", true),
        ("missing_markers", false),
        ("missing_name", false),
        ("missing_metadata", false),
        ("metadata_not_mapping", false),
        ("name_empty", false),
        ("malformed_yaml", false),
    ];

    let mut outcomes = Vec::new();
    for (case_id, expected_ok) in cases {
        let skill_path = matrix_root.join(case_id);
        fs::create_dir_all(&skill_path)?;

        let fixture_content = read_fixture(&format!(
            "skill_scanner_matrix/skill_frontmatter/{case_id}/SKILL.md"
        ));
        fs::write(skill_path.join("SKILL.md"), fixture_content)?;

        let mut row = serde_json::Map::new();
        row.insert("case".to_string(), serde_json::json!(case_id));
        row.insert("expected_ok".to_string(), serde_json::json!(expected_ok));
        match scanner.scan_skill(&skill_path, Some(&structure)) {
            Ok(Some(metadata)) => {
                row.insert("actual_ok".to_string(), serde_json::json!(true));
                row.insert(
                    "skill_name".to_string(),
                    serde_json::json!(metadata.skill_name),
                );
            }
            Ok(None) => {
                row.insert("actual_ok".to_string(), serde_json::json!(false));
                row.insert("error".to_string(), serde_json::json!("scan returned None"));
            }
            Err(error) => {
                row.insert("actual_ok".to_string(), serde_json::json!(false));
                row.insert(
                    "error".to_string(),
                    serde_json::json!(sanitize_path(&error.to_string(), &skill_path)),
                );
            }
        }
        outcomes.push(serde_json::Value::Object(row));
    }

    let outcomes = canonicalize_json(serde_json::to_value(outcomes)?);
    insta::assert_json_snapshot!("skill_frontmatter_matrix", outcomes);
    Ok(())
}

// ============================================================================
// Snapshot: Reference Frontmatter Matrix
// ============================================================================

#[test]
fn snapshot_reference_frontmatter_matrix_contract() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let structure = default_structure();
    let temp_dir = TempDir::new()?;
    let matrix_root = temp_dir.path().join("skills");
    fs::create_dir_all(&matrix_root)?;

    let cases = [
        ("valid_knowledge", true),
        ("valid_persona", true),
        ("missing_type", false),
        ("invalid_type", false),
        ("persona_missing_role_class", false),
        ("missing_markers", false),
        ("malformed_yaml", false),
    ];

    let mut outcomes = Vec::new();
    for (case_id, expected_ok) in cases {
        let skill_path = matrix_root.join(case_id);
        fs::create_dir_all(skill_path.join("references"))?;

        let base_content = read_fixture("skill_scanner_matrix/reference_frontmatter/base/SKILL.md");
        fs::write(skill_path.join("SKILL.md"), base_content)?;

        let ref_content = read_fixture(&format!(
            "skill_scanner_matrix/reference_frontmatter/{case_id}/doc.md"
        ));
        fs::write(skill_path.join("references/doc.md"), ref_content)?;

        let mut row = serde_json::Map::new();
        row.insert("case".to_string(), serde_json::json!(case_id));
        row.insert("expected_ok".to_string(), serde_json::json!(expected_ok));
        match scanner.scan_skill(&skill_path, Some(&structure)) {
            Ok(Some(metadata)) => {
                row.insert("actual_ok".to_string(), serde_json::json!(true));
                row.insert(
                    "skill_name".to_string(),
                    serde_json::json!(metadata.skill_name),
                );
            }
            Ok(None) => {
                row.insert("actual_ok".to_string(), serde_json::json!(false));
                row.insert("error".to_string(), serde_json::json!("scan returned None"));
            }
            Err(error) => {
                row.insert("actual_ok".to_string(), serde_json::json!(false));
                row.insert(
                    "error".to_string(),
                    serde_json::json!(sanitize_path(&error.to_string(), &skill_path)),
                );
            }
        }
        outcomes.push(serde_json::Value::Object(row));
    }

    let outcomes = canonicalize_json(serde_json::to_value(outcomes)?);
    insta::assert_json_snapshot!("reference_frontmatter_matrix", outcomes);
    Ok(())
}

// ============================================================================
// Snapshot: Parse Skill MD Matrix
// ============================================================================

#[test]
fn snapshot_parse_skill_md_matrix_contract() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = SkillScanner::new();
    let temp_dir = TempDir::new()?;
    let matrix_root = temp_dir.path().join("skills");
    fs::create_dir_all(&matrix_root)?;

    let cases = [
        ("writer_full", "writer", true),
        ("researcher_spaces", "researcher", true),
        ("no_frontmatter", "minimal", true),
        ("malformed_yaml", "broken", false),
    ];

    let mut outcomes = Vec::new();
    for (case_id, skill_dir, expected_ok) in cases {
        let skill_path = matrix_root.join(skill_dir);
        fs::create_dir_all(&skill_path)?;
        let content = read_fixture(&format!(
            "skill_scanner_matrix/parse_skill_md/{case_id}/SKILL.md"
        ));

        let mut row = serde_json::Map::new();
        row.insert("case".to_string(), serde_json::json!(case_id));
        row.insert("expected_ok".to_string(), serde_json::json!(expected_ok));
        match scanner.parse_skill_md(content.as_str(), &skill_path) {
            Ok(metadata) => {
                row.insert("actual_ok".to_string(), serde_json::json!(true));
                row.insert("metadata".to_string(), serde_json::to_value(metadata)?);
            }
            Err(error) => {
                row.insert("actual_ok".to_string(), serde_json::json!(false));
                row.insert("error".to_string(), serde_json::json!(error.to_string()));
            }
        }
        outcomes.push(serde_json::Value::Object(row));
    }

    let outcomes = canonicalize_json(serde_json::to_value(outcomes)?);
    insta::assert_json_snapshot!("parse_skill_md_matrix", outcomes);
    Ok(())
}
