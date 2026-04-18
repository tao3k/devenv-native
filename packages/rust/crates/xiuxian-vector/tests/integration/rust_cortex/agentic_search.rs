use super::*;

#[tokio::test]
async fn test_agentic_search_delegates_to_hybrid() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("agentic_test");
    clean_test_db(&db_path);

    let store = VectorStore::new(db_path.to_string_lossy().as_ref(), Some(10)).await?;

    let tools = [
        (
            "git.commit",
            "Commit changes",
            r#"{"skill_name": "git", "tool_name": "commit", "type": "command", "command": "git.commit", "file_path": "skills/git/SKILL.md"}"#,
            vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "knowledge.recall",
            "Recall from knowledge",
            r#"{"skill_name": "knowledge", "tool_name": "recall", "type": "command", "command": "knowledge.recall", "file_path": "knowledge/recall.py"}"#,
            vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
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
    let config = AgenticSearchConfig::default();
    let limit = config.limit;
    let results = store.agentic_search("tools", &query, None, config).await?;
    assert!(!results.is_empty());
    assert!(results.len() <= limit);

    let config_with_intent = AgenticSearchConfig {
        intent: Some(QueryIntent::Hybrid),
        ..AgenticSearchConfig::default()
    };
    let results = store
        .agentic_search("tools", &query, None, config_with_intent)
        .await?;
    assert!(!results.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_agentic_search_semantic_vector_only() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("agentic_semantic");
    clean_test_db(&db_path);

    let store = VectorStore::new(db_path.to_string_lossy().as_ref(), Some(10)).await?;
    let tools = [(
        "a.cmd",
        "desc",
        r#"{"skill_name": "a", "tool_name": "cmd", "type": "command", "command": "a.cmd", "file_path": "a/cmd.py"}"#,
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    )];
    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();
    store
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    let config = AgenticSearchConfig {
        intent: Some(QueryIntent::Semantic),
        limit: 5,
        threshold: 0.0,
        ..AgenticSearchConfig::default()
    };
    let results = store
        .agentic_search(
            "tools",
            &[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            None,
            config,
        )
        .await?;
    assert!(!results.is_empty());
    assert_eq!(results[0].name, "a.cmd");
    Ok(())
}

#[tokio::test]
async fn test_agentic_search_exact_fallback_without_keyword_index() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("agentic_exact_fallback");
    clean_test_db(&db_path);

    let store = VectorStore::new(db_path.to_string_lossy().as_ref(), Some(10)).await?;
    let tools = [(
        "git.commit",
        "Commit",
        r#"{"skill_name": "git", "tool_name": "commit", "type": "command", "command": "git.commit", "file_path": "skills/git/SKILL.md"}"#,
        vec![1.0; 10],
    )];
    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();
    store
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    let config = AgenticSearchConfig {
        intent: Some(QueryIntent::Exact),
        limit: 5,
        threshold: 0.0,
        ..AgenticSearchConfig::default()
    };
    let query = vec![1.0f32; 10];
    let results = store
        .agentic_search("tools", &query, Some("commit"), config)
        .await?;
    assert!(
        results.len() <= 5,
        "Exact without keyword index falls back to hybrid"
    );
    Ok(())
}

#[tokio::test]
async fn test_agentic_search_skill_name_filter() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("agentic_filter_test");
    clean_test_db(&db_path);

    let store = VectorStore::new(db_path.to_string_lossy().as_ref(), Some(10)).await?;
    let tools = [
        (
            "git.commit",
            "Commit changes",
            r#"{"skill_name": "git", "tool_name": "commit", "category": "vcs", "type": "command", "command": "git.commit", "file_path": "skills/git/SKILL.md"}"#,
            vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "knowledge.recall",
            "Recall from knowledge",
            r#"{"skill_name": "knowledge", "tool_name": "recall", "category": "knowledge", "type": "command", "command": "knowledge.recall", "file_path": "knowledge/recall.py"}"#,
            vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    ];
    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();
    store
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    let config = AgenticSearchConfig {
        skill_name_filter: Some("knowledge".to_string()),
        ..AgenticSearchConfig::default()
    };
    let query = vec![0.95, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let results = store.agentic_search("tools", &query, None, config).await?;
    assert!(
        results
            .iter()
            .all(|result| result.skill_name == "knowledge"),
        "skill_name_filter should restrict to knowledge; got: {:?}",
        results
            .iter()
            .map(|result| &result.skill_name)
            .collect::<Vec<_>>()
    );
    assert!(!results.is_empty());
    Ok(())
}
