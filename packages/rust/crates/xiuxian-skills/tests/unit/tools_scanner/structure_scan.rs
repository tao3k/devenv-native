use tempfile::TempDir;

use super::{TestResult, create_scripts_dir, scan_with_structure, write_script};

#[test]
fn test_scan_with_structure_single_directory() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("writer");
    let scripts_dir = create_scripts_dir(&temp_dir, "writer")?;
    write_script(
        &scripts_dir,
        "text.py",
        r#"
@skill_command(name="write_text")
def write_text(content: str) -> str:
    '''Write text to a file.'''
    return "written"
"#,
    )?;

    let tools = scan_with_structure(
        &skill_path,
        "writer",
        &["write".to_string(), "edit".to_string()],
    )?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "writer.write_text");
    Ok(())
}

#[test]
fn test_scan_with_structure_skips_missing_directories() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("empty_skill");

    let tools = scan_with_structure(&skill_path, "empty_skill", &[])?;
    assert!(tools.is_empty());
    Ok(())
}

#[test]
fn test_scan_with_structure_nonexistent_skill_path() -> TestResult {
    let temp_dir = TempDir::new()?;
    let nonexistent_path = temp_dir.path().join("does_not_exist");

    let tools = scan_with_structure(&nonexistent_path, "ghost", &[])?;
    assert!(tools.is_empty());
    Ok(())
}

#[test]
fn test_scan_with_structure_includes_routing_keywords() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("git");
    let scripts_dir = create_scripts_dir(&temp_dir, "git")?;
    write_script(
        &scripts_dir,
        "main.py",
        r#"
@skill_command(name="commit")
def commit(message: str) -> str:
    '''Create a commit.'''
    return f"Committed: {message}"
"#,
    )?;

    let routing_keywords = vec!["git".to_string(), "version_control".to_string()];
    let tools = scan_with_structure(&skill_path, "git", &routing_keywords)?;
    assert_eq!(tools.len(), 1);
    let keywords = &tools[0].keywords;
    assert!(keywords.contains(&"git".to_string()));
    assert!(keywords.contains(&"commit".to_string()));
    assert!(keywords.contains(&"version_control".to_string()));
    Ok(())
}

#[test]
fn test_tool_record_contains_file_metadata() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "example.py",
        r#"
@skill_command(name="example")
def example():
    '''Example tool.'''
    pass
"#,
    )?;

    let tools = super::scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert!(!tools[0].file_path.is_empty());
    assert!(!tools[0].file_hash.is_empty());
    assert_eq!(tools[0].file_hash.len(), 64);
    Ok(())
}
