use std::io;

use tempfile::TempDir;

use super::{TestResult, create_scripts_dir, scan_scripts, write_script};

#[test]
fn test_parameter_extraction_basic() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "params.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="example")
def example(a: str, b: int, c: bool) -> str:
    '''Example with multiple params.'''
    return "ok"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].parameters, vec!["a", "b", "c"]);
    Ok(())
}

#[test]
fn test_parameter_extraction_with_defaults() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "defaults.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="defaults")
def with_defaults(required: str, optional: str = "default", number: int = 42) -> str:
    '''Function with default values.'''
    return "ok"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].parameters, vec!["required", "optional", "number"]);
    Ok(())
}

#[test]
fn test_async_function_type_inference() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "skill")?;
    write_script(
        &scripts_dir,
        "discover.py",
        r#"
@skill_command(name="discover")
async def discover(intent: str, limit: int = 3) -> dict:
    '''Test async function.'''
    return {}
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "skill", &[])?;
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    let schema: serde_json::Value = serde_json::from_str(&tool.input_schema)?;
    let props = schema["properties"]
        .as_object()
        .ok_or_else(|| io::Error::other("schema.properties should be object"))?;

    assert_eq!(props["intent"]["type"], "string");
    assert_eq!(props["limit"]["type"], "integer");

    let required: Vec<&str> = schema["required"]
        .as_array()
        .ok_or_else(|| io::Error::other("schema.required should be array"))?
        .iter()
        .map(|value| value.as_str())
        .collect::<Option<Vec<&str>>>()
        .ok_or_else(|| io::Error::other("required entries should be strings"))?;

    assert!(required.contains(&"intent"));
    assert!(!required.contains(&"limit"));
    Ok(())
}

#[test]
fn test_input_schema_type_inference() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "types.py",
        r#"
@skill_command(name="types")
def test_types(
    name: str,
    count: int = 10,
    enabled: bool,
    tags: list[str],
    metadata: dict[str, str] | None = None,
) -> str:
    '''Test type inference.'''
    return name
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    let schema: serde_json::Value = serde_json::from_str(&tool.input_schema)?;

    assert_eq!(schema["type"], "object");
    assert!(schema["properties"].is_object());
    assert!(schema["required"].is_array());

    let props = schema["properties"]
        .as_object()
        .ok_or_else(|| io::Error::other("schema.properties should be object"))?;
    assert_eq!(props["name"]["type"], "string");
    assert_eq!(props["count"]["type"], "integer");
    assert_eq!(props["enabled"]["type"], "boolean");

    let tags_type = &props["tags"]["type"];
    assert_eq!(tags_type["type"], "array");
    assert_eq!(tags_type["items"]["type"], "string");

    let metadata_type = &props["metadata"]["type"];
    assert_eq!(metadata_type["type"], "object");
    assert_eq!(metadata_type["additionalProperties"], true);

    let required: Vec<&str> = schema["required"]
        .as_array()
        .ok_or_else(|| io::Error::other("schema.required should be array"))?
        .iter()
        .map(|value| value.as_str())
        .collect::<Option<Vec<&str>>>()
        .ok_or_else(|| io::Error::other("required entries should be strings"))?;

    assert!(required.contains(&"name"));
    assert!(required.contains(&"enabled"));
    assert!(required.contains(&"tags"));
    assert!(!required.contains(&"count"));
    assert!(!required.contains(&"metadata"));
    Ok(())
}

#[test]
fn test_input_schema_with_param_descriptions() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "described.py",
        r#"
@skill_command(name="described")
def described_tool(
    message: str,
    count: int = 5,
) -> str:
    '''Tool with described parameters.

    Args:
        - message: str - The message to process (required)
        - count: int - Number of times to repeat (default: 5)

    Returns:
        Processed result.
    '''
    return message
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    let schema: serde_json::Value = serde_json::from_str(&tool.input_schema)?;
    let props = schema["properties"]
        .as_object()
        .ok_or_else(|| io::Error::other("schema.properties should be object"))?;

    assert!(props["message"]["description"].is_string());
    assert!(props["count"]["description"].is_string());

    let msg_desc = props["message"]["description"]
        .as_str()
        .ok_or_else(|| io::Error::other("message.description should be a string"))?;
    assert!(msg_desc.contains("message"));

    let count_desc = props["count"]["description"]
        .as_str()
        .ok_or_else(|| io::Error::other("count.description should be a string"))?;
    assert!(count_desc.contains("repeat"));
    Ok(())
}

#[test]
fn test_input_schema_literal_types() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "literal_test.py",
        r#"
@skill_command(name="literal_test")
def literal_tool(mode: Literal["fast", "slow", "normal"] = "normal") -> str:
    '''Test Literal type.'''
    return mode
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    let schema: serde_json::Value = serde_json::from_str(&tool.input_schema)?;
    let props = schema["properties"]
        .as_object()
        .ok_or_else(|| io::Error::other("schema.properties should be object"))?;

    let mode_type = &props["mode"]["type"];
    assert_eq!(mode_type["type"], "string");
    assert!(mode_type["enum"].is_array());
    Ok(())
}

#[test]
fn test_parameter_extraction_skips_varargs() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "varargs.py",
        r#"
from agent.skills.decorators import skill_command

@skill_command(name="varargs")
def with_varargs(a: str, *args, b: int, **kwargs) -> str:
    '''Function with *args and **kwargs.'''
    return "ok"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].parameters, vec!["a", "b"]);
    Ok(())
}

#[test]
fn test_parameter_extraction_empty() -> TestResult {
    let temp_dir = TempDir::new()?;
    let scripts_dir = create_scripts_dir(&temp_dir, "test")?;
    write_script(
        &scripts_dir,
        "no_args.py",
        r#"
@skill_command(name="no_args")
def no_args() -> str:
    '''Function with no arguments.'''
    return "ok"
"#,
    )?;

    let tools = scan_scripts(&scripts_dir, "test", &[])?;
    assert_eq!(tools.len(), 1);
    assert!(tools[0].parameters.is_empty());
    Ok(())
}
