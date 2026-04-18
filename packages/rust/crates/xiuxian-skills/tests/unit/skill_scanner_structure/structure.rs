use std::fs;
use std::io;
use std::path::PathBuf;

use tempfile::TempDir;
use xiuxian_skills::SkillScanner;

use super::{TestResult, create_skill_dir, write_skill_md};

#[test]
fn test_default_structure_required_files() {
    let structure = SkillScanner::default_structure();

    assert!(!structure.required.is_empty());
    assert!(structure.required.iter().any(|i| i.path == "SKILL.md"));
    assert!(structure.required.iter().any(|i| i.item_type == "file"));
}

#[test]
fn test_default_structure_default_directories() {
    let structure = SkillScanner::default_structure();

    assert!(!structure.default.is_empty());
    assert!(structure.default.iter().any(|i| i.path == "scripts/"));
    assert!(structure.default.iter().any(|i| i.path == "references/"));
    assert!(
        structure.optional.is_empty(),
        "embedded skills.toml currently defines no optional entries"
    );
}

#[test]
fn test_validate_structure_valid_skill() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = create_skill_dir(&temp_dir, "writer");
    fs::create_dir_all(skill_path.join("scripts"))?;
    fs::create_dir_all(skill_path.join("references"))?;
    fs::create_dir_all(skill_path.join("assets"))?;
    fs::create_dir_all(skill_path.join("tests"))?;

    write_skill_md(
        &skill_path,
        r#"---
name: "writer"
metadata:
  version: "1.0"
  routing_keywords: ["write", "edit"]
---
# Writer Skill
"#,
    )?;

    let structure = SkillScanner::default_structure();
    assert!(SkillScanner::validate_structure(&skill_path, &structure));
    Ok(())
}

#[test]
fn test_validate_structure_missing_skill_md() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = create_skill_dir(&temp_dir, "empty_skill");

    let structure = SkillScanner::default_structure();
    assert!(!SkillScanner::validate_structure(&skill_path, &structure));
    Ok(())
}

#[test]
fn test_validate_structure_ignores_out_of_scope_entries() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = create_skill_dir(&temp_dir, "writer");
    fs::create_dir_all(skill_path.join("scripts"))?;
    fs::create_dir_all(skill_path.join("references"))?;
    fs::create_dir_all(skill_path.join("assets"))?;
    fs::create_dir_all(skill_path.join("tests"))?;
    fs::create_dir_all(skill_path.join("temp_junk"))?;
    write_skill_md(
        &skill_path,
        r#"---
name: writer
description: Use when writing.
metadata:
  version: "1.0.0"
---
# Writer
"#,
    )?;

    let structure = SkillScanner::default_structure();
    let report = SkillScanner::validate_structure_report(&skill_path, &structure);
    assert!(
        report.valid,
        "out-of-scope entries should not invalidate a skill"
    );
    assert!(
        report.issues.is_empty(),
        "unexpected validation issues: {report:?}"
    );
    Ok(())
}

#[test]
fn test_validate_structure_nonexistent_path() {
    let structure = SkillScanner::default_structure();
    let nonexistent = PathBuf::from("/nonexistent/path");
    assert!(!SkillScanner::validate_structure(&nonexistent, &structure));
}

#[test]
fn test_skill_name_from_directory() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = create_skill_dir(&temp_dir, "custom_skill_name");

    write_skill_md(
        &skill_path,
        r#"---
version: "1.0"
---
# Content
"#,
    )?;

    let scanner = SkillScanner::new();
    let result = scanner
        .scan_skill(&skill_path, None)?
        .ok_or_else(|| io::Error::other("expected custom_skill_name metadata"))?;

    assert_eq!(result.skill_name, "custom_skill_name");
    Ok(())
}
