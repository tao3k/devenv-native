use super::{
    build_code_search_response, create_sample_html_repo, create_sample_rust_repo, test_studio_state,
};

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_supports_repo_scoped_ast_grep_without_published_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(xiuxian_wendao::search::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![xiuxian_wendao::search::contracts::UiRepoProjectConfig {
            id: "rust-live".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lang:rust ast:\"fn $NAME($$$ARGS) { $$$BODY }\"".to_string(),
        Some("rust-live"),
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("Rust ast-grep code search response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("ast_match") && hit.path == "src/lib.rs"),
        "expected ast-grep hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_supports_repo_scoped_generic_ast_analysis_without_pattern()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(xiuxian_wendao::search::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![xiuxian_wendao::search::contracts::UiRepoProjectConfig {
            id: "rust-live".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response =
        build_code_search_response(&studio, "scan lang:rust".to_string(), Some("rust-live"), 10)
            .await
            .unwrap_or_else(|error| panic!("Rust ast-grep analysis response: {error:?}"));

    assert!(
        response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "src/lib.rs"
                && hit.best_section.as_deref() == Some("fn scan_rows(dataset: &Dataset) {")
        }),
        "expected generic ast-grep analysis hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_treats_placeholder_ast_pattern_as_generic_analysis()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(xiuxian_wendao::search::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![xiuxian_wendao::search::contracts::UiRepoProjectConfig {
            id: "rust-live".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lang:rust ast:\"$PATTERN\"".to_string(),
        Some("rust-live"),
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("Rust ast-grep placeholder response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("ast_match") && hit.path == "src/lib.rs"),
        "expected placeholder ast-grep analysis hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_supports_repo_scoped_ast_grep_for_html_without_published_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_html_repo(temp.path(), "SearchHtml")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(xiuxian_wendao::search::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![xiuxian_wendao::search::contracts::UiRepoProjectConfig {
            id: "html-live".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lang:html ast:\"<title>$TEXT</title>\"".to_string(),
        Some("html-live"),
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("HTML ast-grep code search response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("ast_match") && hit.path == "index.html"),
        "expected HTML ast-grep hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}
