use tempfile::TempDir;

use super::{TestResult, create_scripts_dir, scan_scripts, write_script};

#[test]
fn test_decorator_kwargs_extracts_description() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "api.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="fetch_data", description="Fetch data from API endpoint")
def fetch_data(url: str) -> dict:
    '''This docstring should be overridden by decorator description.'''
    return {}
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].description, "Fetch data from API endpoint");
    Ok(())
}

#[test]
fn test_decorator_kwargs_extracts_category() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "process.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="process", category="data_processing")
def process_data(input: str) -> str:
    '''Process input data.'''
    return input
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].category, "data_processing");
    Ok(())
}

#[test]
fn test_decorator_kwargs_extracts_destructive() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "danger.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="delete_all", destructive=True)
def delete_all() -> str:
    '''Delete all data.'''
    return "deleted"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert!(tools[0].annotations.destructive);
    assert!(!tools[0].annotations.is_idempotent());
    Ok(())
}

#[test]
fn test_decorator_kwargs_extracts_read_only() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "status.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="get_status", read_only=True)
def get_status() -> dict:
    '''Get system status.'''
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
fn test_decorator_kwargs_combined() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "complex.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(
    name="complex_op",
    description="Perform a complex operation",
    category="operations",
    destructive=False,
    read_only=True
)
def complex_operation(param1: str, param2: int, optional: bool = True) -> str:
    '''Complex operation with all kwargs.'''
    return "done"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    assert_eq!(tool.description, "Perform a complex operation");
    assert_eq!(tool.category, "operations");
    assert!(tool.annotations.read_only);
    assert!(!tool.annotations.destructive);
    Ok(())
}

#[test]
fn test_triple_quoted_description() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "memory")?;
    write_script(
        &scripts_dir,
        "load.py",
        r#"
@skill_command(
    name="load_skill",
    description="""Load a skill's manifest into semantic memory for LLM recall.

    Usage:
    - load_skill("git")
    - load_skill("writer")

    Args:
        skill_name: Name of the skill to load
    """
)
def load_skill(skill_name: str) -> str:
    '''Load skill into memory.'''
    return "loaded"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "memory", &[])?;
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    assert!(tool.description.contains("Load a skill's manifest"));
    assert!(tool.description.contains("Usage:"));
    assert!(tool.description.contains("load_skill"));
    assert!(tool.description.contains("Args:"));
    assert!(tool.description.contains("skill_name"));
    Ok(())
}

#[test]
fn test_category_extraction() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "process.py",
        r#"
@skill_command(name="process", category="data_processing")
def process_data(input: str) -> str:
    '''Process input data.'''
    return input
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].category, "data_processing");
    Ok(())
}

#[test]
fn test_single_line_description() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "fetch.py",
        r#"
@skill_command(name="fetch", description="Fetch data from API")
def fetch_data(url: str) -> str:
    '''Fetch data.'''
    return ""
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].description, "Fetch data from API");
    Ok(())
}
