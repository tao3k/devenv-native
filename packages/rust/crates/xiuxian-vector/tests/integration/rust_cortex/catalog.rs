use super::*;

#[tokio::test]
async fn test_search_tools_basic() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("cortex_test");
    clean_test_db(&db_path);

    let store = VectorStore::new(
        temp_dir
            .path()
            .join("cortex_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
    )
    .await?;

    let tools = [
        (
            "git.commit",
            "Commit changes to repository",
            r#"{"skill_name": "git", "tool_name": "commit", "type": "command", "command": "git.commit", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git", "commit", "vcs"], "input_schema": {"type": "object", "properties": {"message": {"type": "string"}}}}"#,
            vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "git.branch",
            "Create or list branches",
            r#"{"skill_name": "git", "tool_name": "branch", "type": "command", "command": "git.branch", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git", "branch", "vcs"], "input_schema": {"type": "object", "properties": {}}}"#,
            vec![0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "python.run",
            "Execute Python code",
            r#"{"skill_name": "python", "tool_name": "run", "type": "command", "command": "python.run", "file_path": "python/scripts/run.py", "routing_keywords": ["python", "execute", "code"], "input_schema": {"type": "object", "properties": {"code": {"type": "string"}}}}"#,
            vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    ];

    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();

    store
        .add_documents("tools", ids.clone(), vectors, contents, metadatas)
        .await?;

    let query = vec![0.95, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let results = store.search_tools("tools", &query, None, 5, 0.0).await?;

    assert!(!results.is_empty(), "Should find some tools");
    assert!(results.len() <= 5);
    Ok(())
}

#[tokio::test]
async fn test_search_tools_skips_uuid_like_tool_rows() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("uuid_filter_test");
    clean_test_db(&db_path);

    let store = VectorStore::new_with_keyword_index(
        temp_dir
            .path()
            .join("uuid_filter_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
        true,
        None,
        None,
    )
    .await?;

    store
        .add_documents(
            "tools",
            vec![
                "6f9619ff-8b86-d011-b42d-00cf4fc964ff".to_string(),
                "advanced_tools.smart_find".to_string(),
            ],
            vec![vec![0.2; 10], vec![0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]],
            vec![
                "bad uuid payload".to_string(),
                "Find files by name".to_string(),
            ],
            vec![
                r#"{"type":"command","skill_name":"unknown","tool_name":"6f9619ff-8b86-d011-b42d-00cf4fc964ff","command":"6f9619ff-8b86-d011-b42d-00cf4fc964ff","routing_keywords":["uuid"]}"#.to_string(),
                r#"{"type":"command","skill_name":"advanced_tools","tool_name":"smart_find","command":"smart_find","routing_keywords":["find","files"],"category":"file_discovery"}"#.to_string(),
            ],
        )
        .await?;

    let results = store
        .search_tools(
            "tools",
            &[0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Some("find python files"),
            10,
            0.0,
        )
        .await?;

    assert!(
        results
            .iter()
            .all(|result| !result.name.contains("6f9619ff"))
    );
    assert!(
        results
            .iter()
            .any(|result| result.name == "advanced_tools.smart_find")
    );
    Ok(())
}

#[tokio::test]
async fn test_search_tools_with_threshold() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("threshold_test");
    clean_test_db(&db_path);

    let store = VectorStore::new(
        temp_dir
            .path()
            .join("threshold_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
    )
    .await?;

    let tools = [
        (
            "python.run",
            "Run Python code",
            r#"{"skill_name": "python", "tool_name": "run", "type": "command", "command": "python.run", "file_path": "python/run.py", "routing_keywords": ["python"], "input_schema": {}}"#,
            vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "rust.compile",
            "Compile Rust code",
            r#"{"skill_name": "rust", "tool_name": "compile", "type": "command", "command": "rust.compile", "file_path": "rust/compile.py", "routing_keywords": ["rust"], "input_schema": {}}"#,
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

    let query = vec![0.95, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let results = store.search_tools("tools", &query, None, 5, 0.9).await?;

    assert!(results.len() <= 1);
    Ok(())
}

#[tokio::test]
async fn test_load_tool_registry() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("registry_test");
    clean_test_db(&db_path);

    let store = VectorStore::new(
        temp_dir
            .path()
            .join("registry_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
    )
    .await?;

    let tools = [
        (
            "git.commit",
            "Commit changes",
            r#"{"skill_name": "git", "tool_name": "commit", "type": "command", "command": "git.commit", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git"], "input_schema": {"type": "object"}}"#,
            vec![0.0; 10],
        ),
        (
            "git.branch",
            "List branches",
            r#"{"skill_name": "git", "tool_name": "branch", "type": "command", "command": "git.branch", "file_path": "git/branch.py", "routing_keywords": ["git"], "input_schema": {"type": "object"}}"#,
            vec![0.0; 10],
        ),
        (
            "python.run",
            "Run code",
            r#"{"skill_name": "python", "tool_name": "run", "type": "command", "command": "python.run", "file_path": "python/run.py", "routing_keywords": ["python"], "input_schema": {"type": "object"}}"#,
            vec![0.0; 10],
        ),
    ];

    let ids: Vec<String> = tools.iter().map(|tool| tool.0.to_string()).collect();
    let contents: Vec<String> = tools.iter().map(|tool| tool.1.to_string()).collect();
    let metadatas: Vec<String> = tools.iter().map(|tool| tool.2.to_string()).collect();
    let vectors: Vec<Vec<f32>> = tools.iter().map(|tool| tool.3.clone()).collect();

    store
        .add_documents("tools", ids, vectors, contents, metadatas)
        .await?;

    let registry = store.load_tool_registry("tools").await?;

    assert_eq!(registry.len(), 3);
    for tool in registry {
        assert!((tool.score - 1.0).abs() < f32::EPSILON);
        assert!(!tool.name.is_empty());
        assert!(!tool.skill_name.is_empty());
        assert!(!tool.tool_name.is_empty());
        assert!(!tool.file_path.is_empty());
    }
    Ok(())
}

#[tokio::test]
async fn test_tool_search_result_structure() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("struct_test");
    clean_test_db(&db_path);

    let store = VectorStore::new(
        temp_dir
            .path()
            .join("struct_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
    )
    .await?;

    store
        .add_documents(
            "tools",
            vec!["test.tool".to_string()],
            vec![vec![0.0; 10]],
            vec!["Test tool description".to_string()],
            vec![r#"{"skill_name": "test", "tool_name": "tool", "type": "command", "command": "test.tool", "file_path": "test.py", "routing_keywords": ["test"], "input_schema": {"type": "object", "properties": {"arg": {"type": "string"}}}}"#.to_string()],
        )
        .await?;

    let results = store
        .search_tools("tools", &[0.0; 10], None, 1, 0.0)
        .await?;

    assert_eq!(results.len(), 1);
    let result = &results[0];
    assert_eq!(result.name, "test.tool");
    assert_eq!(result.skill_name, "test");
    assert_eq!(result.tool_name, "test.tool");
    assert_eq!(result.file_path, "test.py");
    assert_eq!(result.routing_keywords, vec!["test"]);
    assert!(result.score > 0.0);
    assert!(result.description.contains("Test"));
    Ok(())
}

#[tokio::test]
async fn test_search_tools_vector_only() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("vector_only_test");
    clean_test_db(&db_path);

    let store = VectorStore::new(
        temp_dir
            .path()
            .join("vector_only_test")
            .to_string_lossy()
            .as_ref(),
        Some(10),
    )
    .await?;

    let tools = [
        (
            "git.commit",
            "Commit changes to git",
            r#"{"skill_name": "git", "tool_name": "commit", "type": "command", "command": "git.commit", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git", "commit"], "input_schema": {}}"#,
            vec![0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        (
            "git.status",
            "Show git status",
            r#"{"skill_name": "git", "tool_name": "status", "type": "command", "command": "git.status", "file_path": "skills/git/SKILL.md", "routing_keywords": ["git", "status"], "input_schema": {}}"#,
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

    let results = store
        .search_tools(
            "tools",
            &[0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            None,
            10,
            0.0,
        )
        .await?;

    assert!(!results.is_empty());
    assert_eq!(results[0].name, "git.commit");
    Ok(())
}
