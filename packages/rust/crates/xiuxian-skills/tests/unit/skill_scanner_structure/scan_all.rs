use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;
use xiuxian_skills::SkillScanner;

use super::{TestResult, write_skill_md};

#[test]
fn test_scan_all_with_structure_validation() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let writer_path = skills_dir.join("writer");
    fs::create_dir_all(&writer_path)?;
    write_skill_md(
        &writer_path,
        r#"---
name: "writer"
metadata:
  version: "1.0"
  routing_keywords: ["write", "edit"]
---
# Writer
"#,
    )?;

    let git_path = skills_dir.join("git");
    fs::create_dir_all(&git_path)?;
    write_skill_md(
        &git_path,
        r#"---
name: "git"
metadata:
  version: "1.0"
  routing_keywords: ["commit", "branch"]
---
# Git
"#,
    )?;

    let no_md_path = skills_dir.join("no_md");
    fs::create_dir_all(&no_md_path)?;

    let scanner = SkillScanner::new();
    let structure = SkillScanner::default_structure();
    let metadatas = scanner.scan_all(&skills_dir, Some(&structure))?;
    assert_eq!(metadatas.len(), 2);
    assert!(metadatas.iter().any(|m| m.skill_name == "writer"));
    assert!(metadatas.iter().any(|m| m.skill_name == "git"));
    assert!(!metadatas.iter().any(|m| m.skill_name == "no_md"));
    Ok(())
}

#[test]
fn test_scan_all_without_structure() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let writer_path = skills_dir.join("writer");
    fs::create_dir_all(&writer_path)?;
    write_skill_md(
        &writer_path,
        r#"---
name: "writer"
metadata:
  version: "1.0"
---
# Writer
"#,
    )?;

    let scanner = SkillScanner::new();
    let metadatas = scanner.scan_all(&skills_dir, None)?;
    assert_eq!(metadatas.len(), 1);
    Ok(())
}

#[test]
fn test_scan_all_nonexistent_base_path() {
    let scanner = SkillScanner::new();
    let nonexistent_path = PathBuf::from("/nonexistent");
    let Err(error) = scanner.scan_all(&nonexistent_path, None) else {
        panic!("non-directory root should return an error");
    };
    assert!(
        error.to_string().contains("Root path is not a directory"),
        "unexpected error: {error}"
    );
}
