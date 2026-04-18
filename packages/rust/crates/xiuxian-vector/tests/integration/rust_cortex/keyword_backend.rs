use super::*;

#[tokio::test]
async fn test_search_tools_weighted_rrf() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("wrrf_test");
    clean_test_db(&db_path);

    let store = VectorStore::new_with_keyword_index(
        temp_dir.path().join("wrrf_test").to_string_lossy().as_ref(),
        Some(10),
        true,
        None,
        None,
    )
    .await?;

    let tools = [
        (
            "git.commit",
            "Commit changes to repository",
            r#"{"skill_name": "git", "tool_name": "commit", "type": "command", "command": "git.commit", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git", "commit", "vcs"], "input_schema": {}}"#,
            vec![0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "git.status",
            "Show working tree status",
            r#"{"skill_name": "git", "tool_name": "status", "type": "command", "command": "git.status", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git", "status", "vcs"], "input_schema": {}}"#,
            vec![0.8, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "git.branch",
            "Create or list branches",
            r#"{"skill_name": "git", "tool_name": "branch", "type": "command", "command": "git.branch", "file_path": "git/branch.py", "routing_keywords": ["git", "branch", "vcs"], "input_schema": {}}"#,
            vec![0.7, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "python.run",
            "Run Python code",
            r#"{"skill_name": "python", "tool_name": "run", "type": "command", "command": "python.run", "file_path": "python/run.py", "routing_keywords": ["python", "run"], "input_schema": {}}"#,
            vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
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
            "git.commit".to_string(),
            "Commit changes to repository".to_string(),
            "git".to_string(),
            vec!["git".to_string(), "commit".to_string()],
            vec![],
        ),
        (
            "git.status".to_string(),
            "Show working tree status".to_string(),
            "git".to_string(),
            vec!["git".to_string(), "status".to_string()],
            vec![],
        ),
        (
            "git.branch".to_string(),
            "Create or list branches".to_string(),
            "git".to_string(),
            vec!["git".to_string(), "branch".to_string()],
            vec![],
        ),
        (
            "python.run".to_string(),
            "Run Python code".to_string(),
            "python".to_string(),
            vec!["python".to_string(), "run".to_string()],
            vec![],
        ),
    ];
    store.bulk_index_keywords(kw_docs)?;

    let query = vec![0.85, 0.15, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let results = store
        .search_tools("tools", &query, Some("commit"), 10, 0.0)
        .await?;

    assert!(!results.is_empty());
    assert_eq!(results[0].name, "git.commit");
    Ok(())
}

#[tokio::test]
async fn test_search_tools_field_boosting() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("field_boost_test");
    clean_test_db(&db_path);

    let store = VectorStore::new_with_keyword_index(
        temp_dir
            .path()
            .join("field_boost_test")
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
            "git.commit",
            "Commit changes to repository",
            r#"{"skill_name": "git", "tool_name": "commit", "type": "command", "command": "git.commit", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git", "commit"], "input_schema": {}}"#,
            vec![0.5; 10],
        ),
        (
            "git.status",
            "Show git status",
            r#"{"skill_name": "git", "tool_name": "status", "type": "command", "command": "git.status", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git", "status"], "input_schema": {}}"#,
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
            "git.commit".to_string(),
            "Commit changes to repository".to_string(),
            "git".to_string(),
            vec!["git".to_string(), "commit".to_string()],
            vec![],
        ),
        (
            "git.status".to_string(),
            "Show git status".to_string(),
            "git".to_string(),
            vec!["git".to_string(), "status".to_string()],
            vec![],
        ),
    ];
    store.bulk_index_keywords(kw_docs)?;

    let query = vec![0.5; 10];
    let results = store
        .search_tools("tools", &query, Some("git commit"), 10, 0.0)
        .await?;

    assert!(
        results.len() >= 2,
        "Expected at least 2 results, got {}",
        results.len()
    );
    assert_eq!(results[0].name, "git.commit");
    assert!(
        results[0].score > results[1].score,
        "Field boosting should give git.commit higher score"
    );
    Ok(())
}

#[tokio::test]
async fn test_search_tools_keyword_rescue() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("rescue_test");
    clean_test_db(&db_path);

    let store = VectorStore::new_with_keyword_index(
        temp_dir
            .path()
            .join("rescue_test")
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
            "git.commit",
            "Commit changes",
            r#"{"skill_name": "git", "tool_name": "commit", "type": "command", "command": "git.commit", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git", "commit"], "input_schema": {}}"#,
            vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "filesystem.read",
            "Read file contents",
            r#"{"skill_name": "filesystem", "tool_name": "read", "type": "command", "command": "filesystem.read", "file_path": "fs/read.py", "routing_keywords": ["file", "read"], "input_schema": {}}"#,
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

    let kw_docs: Vec<KeywordDoc> = vec![
        (
            "git.commit".to_string(),
            "Commit changes".to_string(),
            "git".to_string(),
            vec!["git".to_string(), "commit".to_string()],
            vec![],
        ),
        (
            "filesystem.read".to_string(),
            "Read file contents".to_string(),
            "filesystem".to_string(),
            vec!["file".to_string(), "read".to_string()],
            vec![],
        ),
    ];
    store.bulk_index_keywords(kw_docs)?;

    let query = vec![0.0, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let results = store
        .search_tools("tools", &query, Some("git commit"), 10, 0.0)
        .await?;

    assert!(results.iter().any(|result| result.name == "git.commit"));
    Ok(())
}

#[tokio::test]
async fn test_search_tools_keyword_only_rescue() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("kw_rescue_test");
    clean_test_db(&db_path);

    let store = VectorStore::new_with_keyword_index(
        temp_dir
            .path()
            .join("kw_rescue_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
        true,
        None,
        None,
    )
    .await?;

    let tools = [(
        "database.query",
        "Execute database query",
        r#"{"skill_name": "database", "tool_name": "query", "type": "command", "command": "database.query", "file_path": "db/scripts/query.py", "routing_keywords": ["database", "query", "sql"], "input_schema": {}}"#,
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    )];

    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();

    store
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    let kw_docs = vec![(
        "database.query".to_string(),
        "Execute database query".to_string(),
        "database".to_string(),
        vec!["database".to_string(), "query".to_string()],
        vec![],
    )];
    store.bulk_index_keywords(kw_docs)?;

    let results = store
        .search_tools(
            "tools",
            &[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Some("sql query"),
            10,
            0.0,
        )
        .await?;

    assert!(results.iter().any(|result| result.name == "database.query"));
    Ok(())
}
