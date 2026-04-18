use tempfile::TempDir;
use xiuxian_skills::ToolsScanner;

use super::{TestResult, create_scripts_dir, scan_scripts, write_script};

#[test]
fn test_scan_scripts_single_tool() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "writer")?;
    write_script(
        &scripts_dir,
        "text.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="write_text")
def write_text(content: str) -> str:
    '''Write text to a file.'''
    return "written"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "writer", &["write".to_string()])?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "writer.write_text");
    assert_eq!(tools[0].function_name, "write_text");
    assert_eq!(tools[0].skill_name, "writer");
    Ok(())
}

#[test]
fn test_scan_scripts_multiple_tools() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "git")?;
    write_script(
        &scripts_dir,
        "main.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="commit")
def commit(message: str) -> str:
    '''Create a commit.'''
    return f"Committed: {message}"

@skill_command(name="status")
def status() -> str:
    '''Show working tree status.'''
    return "status output"

@skill_command(name="branch")
def branch(name: str) -> str:
    '''Create a new branch.'''
    return f"Created branch: {name}"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "git", &["git".to_string()])?;
    assert_eq!(tools.len(), 3);
    assert!(tools.iter().any(|t| t.tool_name == "git.commit"));
    assert!(tools.iter().any(|t| t.tool_name == "git.status"));
    assert!(tools.iter().any(|t| t.tool_name == "git.branch"));
    Ok(())
}

#[test]
fn test_scan_scripts_no_scripts_dir() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = temp_dir.path().join("empty/scripts");

    let tools = scan_scripts(&scripts_dir, "empty", &[])?;
    assert!(tools.is_empty());
    Ok(())
}

#[test]
fn test_scan_scripts_empty_dir() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "empty")?;

    let tools = scan_scripts(&scripts_dir, "empty", &[])?;
    assert!(tools.is_empty());
    Ok(())
}

#[test]
fn test_parse_script_skips_init() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "__init__.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="init_tool")
def init_tool():
    '''This should be skipped.'''
    pass
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert!(tools.is_empty());
    Ok(())
}

#[test]
fn test_scan_skill_scripts() -> TestResult {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("test_skill");
    let scripts_dir = create_scripts_dir(&temp_dir, "test_skill")?;
    write_script(
        &scripts_dir,
        "test.py",
        r#"
@skill_command(name="test")
def test_tool():
    '''A test tool.'''
    pass
"#,
    )?;

    let tools = ToolsScanner::new().scan_skill_scripts(&skill_path, "test_skill", &[], &[])?;
    assert_eq!(tools.len(), 1);
    assert!(tools[0].tool_name.starts_with("test_skill."));
    Ok(())
}

#[test]
fn test_scan_nested_directories() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "writer")?;
    let nested_dir = scripts_dir.join("subcommands");
    std::fs::create_dir_all(&nested_dir)?;
    write_script(
        &nested_dir,
        "nested.py",
        r#"
@skill_command(name="nested_tool")
def nested_tool():
    '''Tool in nested directory.'''
    pass
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "writer", &[])?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "writer.nested_tool");
    Ok(())
}

#[test]
fn test_scan_only_python_files() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "tool.py",
        r#"
@skill_command(name="py_tool")
def py_tool():
    pass
"#,
    )?;
    std::fs::write(scripts_dir.join("notes.txt"), "Some notes")?;
    std::fs::write(scripts_dir.join("data.json"), "{}")?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_name, "test.py_tool");
    Ok(())
}
