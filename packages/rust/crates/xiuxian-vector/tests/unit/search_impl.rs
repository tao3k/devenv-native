//! Unit coverage for search-result IPC conversion helpers.

use anyhow::{Result, anyhow};
use xiuxian_types::VectorSearchResult;
use xiuxian_vector::test_support::search_results_to_ipc;

#[test]
fn test_search_results_to_ipc_empty() -> Result<()> {
    let bytes = search_results_to_ipc(&[], None).map_err(|error| anyhow!(error))?;
    assert!(!bytes.is_empty(), "IPC stream should contain schema");
    Ok(())
}

#[test]
fn test_search_results_to_ipc_one_row() -> Result<()> {
    let result = VectorSearchResult {
        id: "tool.a".to_string(),
        content: "Does something".to_string(),
        tool_name: "tool.a".to_string(),
        file_path: "/path/to/file".to_string(),
        routing_keywords: "kw1 kw2".to_string(),
        intents: "intent1 | intent2".to_string(),
        metadata: serde_json::json!({"x": 1}),
        distance: 0.5,
    };
    let bytes = search_results_to_ipc(&[result], None).map_err(|error| anyhow!(error))?;
    assert!(!bytes.is_empty());
    Ok(())
}

#[test]
fn test_search_results_to_ipc_projection() -> Result<()> {
    let result = VectorSearchResult {
        id: "a".to_string(),
        content: "text".to_string(),
        tool_name: "t".to_string(),
        file_path: "p".to_string(),
        routing_keywords: String::new(),
        intents: String::new(),
        metadata: serde_json::json!({}),
        distance: 0.1,
    };
    let projection = vec![
        "id".to_string(),
        "content".to_string(),
        "_distance".to_string(),
    ];
    let projected =
        search_results_to_ipc(std::slice::from_ref(&result), Some(projection.as_slice()))
            .map_err(|error| anyhow!(error))?;
    assert!(!projected.is_empty());
    let full = search_results_to_ipc(&[result], None).map_err(|error| anyhow!(error))?;
    assert!(
        projected.len() < full.len(),
        "projected IPC should be smaller"
    );
    Ok(())
}

#[test]
fn test_search_results_to_ipc_invalid_projection() {
    let result = VectorSearchResult {
        id: "a".to_string(),
        content: "b".to_string(),
        tool_name: "t".to_string(),
        file_path: "p".to_string(),
        routing_keywords: String::new(),
        intents: String::new(),
        metadata: serde_json::json!({}),
        distance: 0.0,
    };
    let projection = vec!["id".to_string(), "no_such_column".to_string()];
    let Err(error) = search_results_to_ipc(&[result], Some(projection.as_slice())) else {
        panic!("invalid projection should fail")
    };
    assert!(error.contains("invalid ipc_projection"));
}
