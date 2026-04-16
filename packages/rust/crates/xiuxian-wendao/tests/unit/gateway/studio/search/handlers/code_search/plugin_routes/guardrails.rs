use super::*;

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_excludes_language_owned_by_non_ast_plugin_from_generic_ast_grep()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_toml_repo(temp.path(), "OwnedToml")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "toml-owned".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["toml".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lang:toml ast:\"name = $VALUE\"".to_string(),
        Some("toml-owned"),
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("owned-language ast-grep code search response: {error:?}"));

    assert!(
        response.hits.is_empty(),
        "generic ast-grep should skip files owned by a dedicated plugin window: {:?}",
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
async fn build_code_search_response_rejects_ast_grep_without_explicit_repository_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "rust-live".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let error = match build_code_search_response(
        &studio,
        "ast:\"fn $NAME($$$ARGS) { $$$BODY }\"".to_string(),
        None,
        10,
    )
    .await
    {
        Ok(response) => panic!("repo-scopeless ast-grep query should fail: {response:?}"),
        Err(error) => error,
    };

    assert_eq!(error.code(), "MISSING_REPOSITORY");
    assert_eq!(
        error.error.message,
        "ast-grep code search requires repo:<id> or an explicit repository hint"
    );
    Ok(())
}
