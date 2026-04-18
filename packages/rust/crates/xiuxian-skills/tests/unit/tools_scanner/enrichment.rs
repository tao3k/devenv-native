use tempfile::TempDir;

use super::{TestResult, create_scripts_dir, scan_scripts, write_script};

#[test]
fn test_tool_record_keywords_includes_skill_keywords() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "writer")?;
    write_script(
        &scripts_dir,
        "text.py",
        r#"
@skill_command(name="polish_text")
def polish_text(text: str) -> str:
    '''Polish text using writing guidelines.'''
    return text
"#,
    )?;

    let routing_keywords = vec![
        "write".to_string(),
        "edit".to_string(),
        "polish".to_string(),
    ];
    let tools = scan_scripts(&scripts_dir, "writer", &routing_keywords)?;

    assert_eq!(tools.len(), 1);
    let keywords = &tools[0].keywords;
    assert!(keywords.contains(&"writer".to_string()));
    assert!(keywords.contains(&"polish_text".to_string()));
    assert!(keywords.contains(&"polish".to_string()));
    assert!(keywords.contains(&"write".to_string()));
    assert!(keywords.contains(&"edit".to_string()));
    Ok(())
}

#[test]
fn test_enrich_tool_record_with_routing_keywords() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "database")?;
    write_script(
        &scripts_dir,
        "db.py",
        r#"
@skill_command(name="query")
def query(sql: str) -> str:
    '''Execute a SQL query.'''
    return "results"
"#,
    )?;

    let routing_keywords = vec![
        "database".to_string(),
        "query".to_string(),
        "sql".to_string(),
        "postgres".to_string(),
    ];
    let tools = scan_scripts(&scripts_dir, "database", &routing_keywords)?;

    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    assert!(tool.keywords.contains(&"database".to_string()));
    assert!(tool.keywords.contains(&"query".to_string()));
    assert!(tool.keywords.contains(&"sql".to_string()));
    assert!(tool.keywords.contains(&"postgres".to_string()));
    Ok(())
}

#[test]
fn test_enrich_multiple_tools_with_same_keywords() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "api")?;
    write_script(
        &scripts_dir,
        "users.py",
        r#"
@skill_command(name="get_user")
def get_user(user_id: str) -> dict:
    '''Get user by ID.'''
    return {}

@skill_command(name="create_user")
def create_user(name: str, email: str) -> dict:
    '''Create a new user.'''
    return {}

@skill_command(name="delete_user")
def delete_user(user_id: str) -> bool:
    '''Delete a user.'''
    return true
"#,
    )?;

    let routing_keywords = vec!["api".to_string(), "rest".to_string(), "user".to_string()];
    let tools = scan_scripts(&scripts_dir, "api", &routing_keywords)?;

    assert_eq!(tools.len(), 3);
    for tool in &tools {
        assert!(tool.keywords.contains(&"api".to_string()));
        assert!(tool.keywords.contains(&"rest".to_string()));
        assert!(tool.keywords.contains(&"user".to_string()));
        assert_eq!(tool.skill_name, "api");
    }
    Ok(())
}

#[test]
fn test_enrich_with_empty_routing_keywords() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "hello.py",
        r#"
@skill_command(name="hello")
def hello() -> str:
    '''Say hello.'''
    return "Hello!"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    assert!(tool.keywords.contains(&"test".to_string()));
    assert!(tool.keywords.contains(&"hello".to_string()));
    Ok(())
}

#[test]
fn test_enrich_metadata_structure_for_hybrid_search() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "search")?;
    write_script(
        &scripts_dir,
        "search.py",
        r#"
@skill_command(name="semantic_search")
def semantic_search(query: str, limit: int = 10) -> list:
    '''Perform semantic search.'''
    return []
"#,
    )?;

    let routing_keywords = vec![
        "search".to_string(),
        "semantic".to_string(),
        "vector".to_string(),
    ];
    let tools = scan_scripts(&scripts_dir, "search", &routing_keywords)?;

    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    assert!(!tool.skill_name.is_empty());
    assert!(!tool.tool_name.is_empty());
    assert!(!tool.function_name.is_empty());
    assert!(!tool.file_path.is_empty());
    assert!(!tool.file_hash.is_empty());
    assert!(!tool.description.is_empty());
    assert!(!tool.keywords.is_empty());
    assert!(tool.keywords.contains(&"search".to_string()));
    assert!(tool.keywords.contains(&"semantic".to_string()));
    assert!(tool.keywords.contains(&"vector".to_string()));
    Ok(())
}

#[test]
fn test_enrich_preserves_docstring() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "docs")?;
    write_script(
        &scripts_dir,
        "docs.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="generate_docs")
def generate_docs(source_path: str, output_format: str = "markdown") -> str:
    '''Generate documentation from source code.

    Args:
        source_path: Path to source files
        output_format: Output format (markdown, html, rst)

    Returns:
        Generated documentation content
    '''
    return "docs"
"#,
    )?;

    let routing_keywords = vec!["documentation".to_string(), "docs".to_string()];
    let tools = scan_scripts(&scripts_dir, "docs", &routing_keywords)?;

    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    assert!(tool.docstring.contains("Generate documentation"));
    assert!(tool.docstring.contains("source_path"));
    assert!(tool.docstring.contains("output_format"));
    Ok(())
}

#[test]
fn test_enrich_with_intent_keywords() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "planner")?;
    write_script(
        &scripts_dir,
        "plan.py",
        r#"
@skill_command(name="create_plan")
def create_plan(goal: str, constraints: list[str] = None) -> dict:
    '''Create an execution plan for a goal.'''
    return {}
"#,
    )?;

    let routing_keywords = vec![
        "plan".to_string(),
        "goal".to_string(),
        "execute".to_string(),
        "strategy".to_string(),
    ];
    let tools = scan_scripts(&scripts_dir, "planner", &routing_keywords)?;

    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    assert!(tool.keywords.contains(&"planner".to_string()));
    assert!(tool.keywords.contains(&"plan".to_string()));
    assert!(tool.keywords.contains(&"goal".to_string()));
    assert!(tool.keywords.contains(&"execute".to_string()));
    assert!(tool.keywords.contains(&"strategy".to_string()));
    Ok(())
}
