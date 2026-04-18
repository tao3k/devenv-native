use super::*;

#[tokio::test]
async fn test_search_tools_key_consistency() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("key_consistency_test");
    clean_test_db(&db_path);

    let store = VectorStore::new_with_keyword_index(
        temp_dir
            .path()
            .join("key_consistency_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
        true,
        None,
        None,
    )
    .await?;

    let tools = [
        (
            "skill.discover",
            "Discover available skills and commands",
            r#"{"skill_name": "skill", "tool_name": "discover", "type": "command", "command": "skill.discover", "file_path": "skill/scripts/discovery.py", "routing_keywords": ["skill", "discover", "find"], "input_schema": {}}"#,
            vec![0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "knowledge.status",
            "Check knowledge base status",
            r#"{"skill_name": "knowledge", "tool_name": "status", "type": "command", "command": "knowledge.status", "file_path": "knowledge/scripts/status.py", "routing_keywords": ["knowledge", "status", "check"], "input_schema": {}}"#,
            vec![0.1, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    ];

    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();

    store
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    let kw_docs: Vec<KeywordDoc> = vec![
        (
            "skill.discover".to_string(),
            "Discover available skills and commands".to_string(),
            "skill".to_string(),
            vec!["skill".to_string(), "discover".to_string()],
            vec![],
        ),
        (
            "knowledge.status".to_string(),
            "Check knowledge base status".to_string(),
            "knowledge".to_string(),
            vec!["knowledge".to_string(), "status".to_string()],
            vec![],
        ),
    ];
    store.bulk_index_keywords(kw_docs)?;

    let query = vec![0.95, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let results = store
        .search_tools("tools", &query, Some("discover"), 10, 0.0)
        .await?;

    assert!(!results.is_empty(), "Should find at least one tool");
    let discover_result = results
        .iter()
        .find(|result| result.name == "skill.discover");
    assert!(
        discover_result.is_some(),
        "skill.discover should be found when searching 'discover'. Got results: {:?}",
        results
            .iter()
            .map(|result| &result.name)
            .collect::<Vec<_>>()
    );
    if let Some(result) = discover_result {
        assert_eq!(result.tool_name, "skill.discover");
        assert_eq!(result.skill_name, "skill");
    }
    Ok(())
}

#[tokio::test]
async fn test_search_tools_exact_skill_command() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("exact_cmd_test");
    clean_test_db(&db_path);

    let store = VectorStore::new_with_keyword_index(
        temp_dir
            .path()
            .join("exact_cmd_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
        true,
        None,
        None,
    )
    .await?;

    let tools = [
        (
            "skill.discover",
            "Capability Discovery - Find available skills",
            r#"{"skill_name": "skill", "tool_name": "discover", "type": "command", "command": "skill.discover", "file_path": "skill/scripts/discovery.py", "routing_keywords": ["skill", "discover"], "input_schema": {}}"#,
            vec![0.5; 10],
        ),
        (
            "knowledge.status",
            "Knowledge base status",
            r#"{"skill_name": "knowledge", "tool_name": "status", "type": "command", "command": "knowledge.status", "file_path": "knowledge/scripts/status.py", "routing_keywords": ["knowledge", "status"], "input_schema": {}}"#,
            vec![0.5; 10],
        ),
    ];

    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();

    store
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    let kw_docs: Vec<KeywordDoc> = vec![
        (
            "skill.discover".to_string(),
            "Capability Discovery".to_string(),
            "skill".to_string(),
            vec!["skill".to_string(), "discover".to_string()],
            vec![],
        ),
        (
            "knowledge.status".to_string(),
            "Knowledge base status".to_string(),
            "knowledge".to_string(),
            vec!["knowledge".to_string(), "status".to_string()],
            vec![],
        ),
    ];
    store.bulk_index_keywords(kw_docs)?;

    let results = store
        .search_tools("tools", &[0.5; 10], Some("skill.discover"), 10, 0.0)
        .await?;

    assert!(!results.is_empty());
    assert_eq!(
        results[0].name,
        "skill.discover",
        "Searching 'skill.discover' should return it first. Got: {:?}",
        results
            .iter()
            .map(|result| &result.name)
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
async fn test_search_tools_same_tool_name_different_skills() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("same_name_test");
    clean_test_db(&db_path);

    let store = VectorStore::new_with_keyword_index(
        temp_dir
            .path()
            .join("same_name_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
        true,
        None,
        None,
    )
    .await?;

    let tools = [
        (
            "git.status",
            "Show git repository status",
            r#"{"skill_name": "git", "tool_name": "status", "type": "command", "command": "git.status", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git", "status"], "input_schema": {}}"#,
            vec![0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "filesystem.status",
            "Check filesystem status",
            r#"{"skill_name": "filesystem", "tool_name": "status", "type": "command", "command": "filesystem.status", "file_path": "fs/scripts/status.py", "routing_keywords": ["filesystem", "status", "disk"], "input_schema": {}}"#,
            vec![0.1, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    ];

    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();

    store
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    let kw_docs: Vec<KeywordDoc> = vec![
        (
            "git.status".to_string(),
            "Show git repository status".to_string(),
            "git".to_string(),
            vec!["git".to_string(), "status".to_string()],
            vec![],
        ),
        (
            "filesystem.status".to_string(),
            "Check filesystem status".to_string(),
            "filesystem".to_string(),
            vec!["filesystem".to_string(), "status".to_string()],
            vec![],
        ),
    ];
    store.bulk_index_keywords(kw_docs)?;

    let results = store
        .search_tools(
            "tools",
            &[0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Some("git.status"),
            10,
            0.0,
        )
        .await?;

    assert_eq!(results[0].name, "git.status");
    assert!(
        results
            .iter()
            .any(|result| result.name == "filesystem.status")
    );
    Ok(())
}
