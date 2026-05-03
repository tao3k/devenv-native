use super::{build_code_search_response, create_sample_rust_repo, test_studio_state};

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_treats_repo_alias_search_term_as_scope_for_placeholder_analysis()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(xiuxian_wendao::search::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![xiuxian_wendao::search::contracts::UiRepoProjectConfig {
            id: "lancd".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lance lang:rust ast:\"$PATTERN\"".to_string(),
        Some("lancd"),
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("Rust ast-grep alias placeholder response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("ast_match") && hit.path == "src/lib.rs"),
        "expected alias-scoped placeholder ast-grep analysis hit in code search response: {:?}",
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
async fn build_code_search_response_infers_repo_scope_from_repository_url_alias()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(xiuxian_wendao::search::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![xiuxian_wendao::search::contracts::UiRepoProjectConfig {
            id: "lancd".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lance lang:rust ast:\"$PATTERN\"".to_string(),
        None,
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("Rust ast-grep inferred alias response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("ast_match") && hit.path == "src/lib.rs"),
        "expected inferred alias ast-grep analysis hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}
