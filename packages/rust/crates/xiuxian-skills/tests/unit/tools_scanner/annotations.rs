use std::io;

use tempfile::TempDir;

use super::{TestResult, create_scripts_dir, scan_scripts, write_script};

#[test]
fn test_annotation_heuristics_read_only() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "getter.py",
        r#"
@skill_command(name="get_data")
def get_data() -> dict:
    '''Get data from storage.'''
    return {}
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert!(tools[0].annotations.read_only);
    assert!(tools[0].annotations.is_idempotent());
    Ok(())
}

#[test]
fn test_annotation_heuristics_destructive() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "remover.py",
        r#"
@skill_command(name="remove_file")
def remove_file(path: str) -> bool:
    '''Remove a file.'''
    return true
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert!(tools[0].annotations.destructive);
    assert!(!tools[0].annotations.is_idempotent());
    Ok(())
}

#[test]
fn test_annotation_heuristics_open_world() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "fetcher.py",
        r#"
@skill_command(name="fetch_url")
def fetch_url(url: str) -> str:
    '''Fetch content from URL.'''
    return ""
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert!(tools[0].annotations.is_open_world());
    Ok(())
}

#[test]
fn test_explicit_annotation_overrides_heuristic() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "delete.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="delete", destructive=False)
def delete_data() -> str:
    '''Delete operation marked as non-destructive.'''
    return "deleted"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert!(!tools[0].annotations.destructive);
    Ok(())
}

#[test]
fn test_multiple_tools_different_annotations() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "db.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="query")
def query_data() -> list:
    '''Query database.'''
    return []

@skill_command(name="insert")
def insert_data(row: dict) -> bool:
    '''Insert into database.'''
    return true

@skill_command(name="delete")
def delete_data(id: str) -> bool:
    '''Delete from database.'''
    return true
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 3);

    let query_tool = tools
        .iter()
        .find(|t| t.tool_name == "test.query")
        .ok_or_else(|| io::Error::other("missing query tool"))?;
    assert!(query_tool.annotations.read_only);
    assert!(query_tool.annotations.is_idempotent());

    let insert_tool = tools
        .iter()
        .find(|t| t.tool_name == "test.insert")
        .ok_or_else(|| io::Error::other("missing insert tool"))?;
    assert!(insert_tool.annotations.destructive);

    let delete_tool = tools
        .iter()
        .find(|t| t.tool_name == "test.delete")
        .ok_or_else(|| io::Error::other("missing delete tool"))?;
    assert!(delete_tool.annotations.destructive);
    Ok(())
}
