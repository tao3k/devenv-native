use super::{
    assert_studio_json_snapshot, build_code_search_response, create_sample_rust_repo,
    load_code_search_response, search_response_snapshot, test_studio_state,
};

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_treats_search_only_repo_seed_as_generic_analysis()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::contracts::UiRepoProjectConfig {
            id: "lance".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(&studio, "lance".to_string(), None, 10)
        .await
        .unwrap_or_else(|error| panic!("search-only repo seed response: {error:?}"));

    assert!(
        response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "src/lib.rs"
                && hit.best_section.as_deref() == Some("fn scan_rows(dataset: &Dataset) {")
        }),
        "expected search-only repo seed to trigger generic ast analysis: {:?}",
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
async fn build_code_search_response_returns_rust_hits_for_search_only_repo_seed_with_lang_filter()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::contracts::UiRepoProjectConfig {
            id: "lance".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(&studio, "lance lang:rust".to_string(), None, 10)
        .await
        .unwrap_or_else(|error| panic!("search-only repo rust response: {error:?}"));

    assert!(
        response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "src/lib.rs"
                && hit.tags.iter().any(|tag| tag == "lang:rust")
        }),
        "expected search-only repo rust query to return src/lib.rs: {:?}",
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
async fn build_code_search_response_returns_toml_hits_for_search_only_repo_seed_with_lang_filter()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::contracts::UiRepoProjectConfig {
            id: "lance".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(&studio, "lance lang:toml".to_string(), None, 10)
        .await
        .unwrap_or_else(|error| panic!("search-only repo toml response: {error:?}"));

    assert!(
        response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "Cargo.toml"
                && hit.tags.iter().any(|tag| tag == "lang:toml")
        }),
        "expected search-only repo toml query to return Cargo.toml: {:?}",
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
async fn build_code_search_response_snapshots_search_only_ast_grep_query_payloads()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::contracts::UiRepoProjectConfig {
            id: "lance".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let plain_seed = load_code_search_response(&studio, "lance").await;
    let rust_filter = load_code_search_response(&studio, "lance lang:rust").await;
    let toml_filter = load_code_search_response(&studio, "lance lang:toml").await;
    let placeholder_rust =
        load_code_search_response(&studio, "lance ast:\"$PATTERN\" lang:rust").await;
    let placeholder_toml =
        load_code_search_response(&studio, "lance ast:\"$PATTERN\" lang:toml").await;

    assert_studio_json_snapshot(
        "search_code_search_response_search_only_ast_grep_query_payloads",
        serde_json::json!({
            "plainSeed": search_response_snapshot(&plain_seed),
            "rustFilter": search_response_snapshot(&rust_filter),
            "tomlFilter": search_response_snapshot(&toml_filter),
            "placeholderRust": search_response_snapshot(&placeholder_rust),
            "placeholderToml": search_response_snapshot(&placeholder_toml),
        }),
    );
    Ok(())
}
