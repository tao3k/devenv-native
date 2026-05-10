//! Test coverage for xiuxian-daochang behavior.

//! Unit tests for multi-server tool name qualification and parsing (no network).

use xiuxian_daochang::{QualifiedToolName, parse_qualified_tool_name, qualify_tool_name};

#[test]
fn qualify_tool_name_format() {
    assert_eq!(
        qualify_tool_name("omniAgent", "run_terminal_cmd"),
        "tool__omniAgent__run_terminal_cmd"
    );
    assert_eq!(qualify_tool_name("s1", "tool_a"), "tool__s1__tool_a");
}

#[test]
fn parse_qualified_tool_name_valid() {
    assert_eq!(
        parse_qualified_tool_name("tool__omniAgent__run_terminal_cmd"),
        Some(QualifiedToolName {
            server: "omniAgent".to_string(),
            tool: "run_terminal_cmd".to_string(),
        })
    );
    assert_eq!(
        parse_qualified_tool_name("tool__s1__tool_a"),
        Some(QualifiedToolName {
            server: "s1".to_string(),
            tool: "tool_a".to_string(),
        })
    );
}

#[test]
fn parse_qualified_tool_name_invalid_returns_none() {
    assert!(parse_qualified_tool_name("run_terminal_cmd").is_none());
    assert!(parse_qualified_tool_name("tool__").is_none());
    assert!(parse_qualified_tool_name("tool__server_only").is_none());
    assert!(parse_qualified_tool_name("").is_none());
}

#[test]
fn qualify_and_parse_roundtrip() {
    let server = "myServer";
    let tool = "my_tool";
    let qualified = qualify_tool_name(server, tool);
    let Some(parsed) = parse_qualified_tool_name(&qualified) else {
        panic!("qualified tool name should parse");
    };
    assert_eq!(parsed.server, server);
    assert_eq!(parsed.tool, tool);
}
