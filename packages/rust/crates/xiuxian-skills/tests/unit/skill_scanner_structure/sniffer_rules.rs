use std::fs;

use tempfile::TempDir;
use xiuxian_skills::{SkillScanner, ToolsScanner};

use super::{TestResult, create_skill_dir, write_skill_md};

#[test]
fn test_parse_rules_toml_valid() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = create_skill_dir(&temp_dir, "python");

    write_skill_md(
        &skill_path,
        r#"---
name: "python"
metadata:
  version: "1.0"
  routing_keywords: ["python", "py"]
---
# Python Skill
"#,
    )?;

    let rules_path = skill_path.join("extensions/sniffer");
    fs::create_dir_all(&rules_path)?;
    fs::write(
        rules_path.join("rules.toml"),
        r#"
[[match]]
type = "file_exists"
pattern = "pyproject.toml"

[[match]]
type = "file_pattern"
pattern = "*.py"
"#,
    )?;

    let scanner = SkillScanner::new();
    let structure = SkillScanner::default_structure();
    let result = scanner
        .scan_skill(&skill_path, Some(&structure))?
        .ok_or_else(|| std::io::Error::other("expected python metadata"))?;
    assert_eq!(result.skill_name, "python");

    let rules = scanner.scan_skill(&skill_path, Some(&structure))?;
    assert!(rules.is_some());
    Ok(())
}

#[test]
fn test_parse_rules_toml_missing() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = create_skill_dir(&temp_dir, "test_skill");

    let scanner = SkillScanner::new();
    let result = scanner.scan_skill(&skill_path, None)?;
    assert!(result.is_none());
    Ok(())
}

#[test]
fn test_build_index_entry_with_sniffer_rules() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skills_dir = temp_dir.path().join("skills");
    fs::create_dir_all(&skills_dir)?;

    let python_path = skills_dir.join("python");
    fs::create_dir_all(&python_path)?;
    write_skill_md(
        &python_path,
        r#"---
name: "python"
metadata:
  version: "1.0"
  routing_keywords: ["python", "py"]
---
# Python Skill
"#,
    )?;

    let rules_path = python_path.join("extensions/sniffer");
    fs::create_dir_all(&rules_path)?;
    fs::write(
        rules_path.join("rules.toml"),
        r#"
[[match]]
type = "file_exists"
pattern = "pyproject.toml"
"#,
    )?;

    let scanner = SkillScanner::new();
    let tools_scanner = ToolsScanner::new();
    let metadatas = scanner.scan_all(&skills_dir, None)?;
    assert_eq!(metadatas.len(), 1);

    let metadata = &metadatas[0];
    let scripts_path = python_path.join("scripts");
    let tools = if scripts_path.exists() {
        tools_scanner.scan_scripts(
            &scripts_path,
            &metadata.skill_name,
            &metadata.routing_keywords,
            &[],
        )?
    } else {
        Vec::new()
    };

    let entry = scanner.build_index_entry(metadata.clone(), &tools, &python_path);
    assert!(!entry.sniffing_rules.is_empty());
    assert_eq!(entry.sniffing_rules[0].pattern, "pyproject.toml");
    assert_eq!(entry.sniffing_rules[0].rule_type, "file_exists");
    Ok(())
}
