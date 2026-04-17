use super::*;

#[tokio::test]
async fn test_search_tools_file_discovery_intent_boost_without_keyword_backend() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("file_discovery_intent_test");
    clean_test_db(&db_path);

    let store = VectorStore::new(
        temp_dir
            .path()
            .join("file_discovery_intent_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
    )
    .await?;

    let tools = [
        (
            "knowledge.search",
            "Search knowledge notes and documents",
            r#"{"skill_name": "knowledge", "tool_name": "knowledge.search", "type": "command", "command": "knowledge.search", "file_path": "knowledge/scripts/search.py", "routing_keywords": ["knowledge","search","notes"], "intents": ["find related notes"], "category": "knowledge", "input_schema": {}}"#,
            vec![0.95, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "advanced_tools.smart_find",
            "Fast recursive file and directory discovery powered by fd",
            r#"{"skill_name": "advanced_tools", "tool_name": "advanced_tools.smart_find", "type": "command", "command": "advanced_tools.smart_find", "file_path": "advanced_tools/scripts/search.py", "routing_keywords": ["find","files","directory","path","fd"], "intents": ["locate files"], "category": "file_discovery", "input_schema": {}}"#,
            vec![0.55, 0.45, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    ];

    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();

    store
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    let query = vec![0.95, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let results = store
        .search_tools(
            "tools",
            &query,
            Some("Search for Python files in current directory"),
            10,
            0.0,
        )
        .await?;

    assert!(!results.is_empty());
    assert_eq!(
        results[0].name,
        "advanced_tools.smart_find",
        "File discovery intent should prioritize smart_find. Got: {:?}",
        results
            .iter()
            .map(|result| &result.name)
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
async fn test_search_tools_with_options_can_disable_rerank_boost() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("file_discovery_rerank_toggle_test");
    clean_test_db(&db_path);

    let store = VectorStore::new(
        temp_dir
            .path()
            .join("file_discovery_rerank_toggle_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
    )
    .await?;

    let tools = [
        (
            "knowledge.search",
            "Search knowledge notes and documents",
            r#"{"skill_name": "knowledge", "tool_name": "knowledge.search", "type": "command", "command": "knowledge.search", "file_path": "knowledge/scripts/search.py", "routing_keywords": ["knowledge","search","notes"], "intents": ["find related notes"], "category": "knowledge", "input_schema": {}}"#,
            vec![0.95, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "advanced_tools.smart_find",
            "Fast recursive file and directory discovery powered by fd",
            r#"{"skill_name": "advanced_tools", "tool_name": "advanced_tools.smart_find", "type": "command", "command": "advanced_tools.smart_find", "file_path": "advanced_tools/scripts/search.py", "routing_keywords": ["find","files","directory","path","fd"], "intents": ["locate files"], "category": "file_discovery", "input_schema": {}}"#,
            vec![0.55, 0.45, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    ];

    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();

    store
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    let query = vec![0.95, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let with_rerank = store
        .search_tools(
            "tools",
            &query,
            Some("Search for Python files in current directory"),
            10,
            0.0,
        )
        .await?;
    let without_rerank = store
        .search_tools_with_options(ToolSearchRequest {
            table_name: "tools",
            query_vector: &query,
            query_text: Some("Search for Python files in current directory"),
            limit: 10,
            threshold: 0.0,
            options: ToolSearchOptions {
                rerank: false,
                semantic_weight: None,
                keyword_weight: None,
            },
            where_filter: None,
        })
        .await?;

    assert_eq!(with_rerank[0].name, "advanced_tools.smart_find");
    assert_eq!(without_rerank[0].name, "knowledge.search");
    Ok(())
}
