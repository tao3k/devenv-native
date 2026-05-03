use super::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
    analyze_registered_repository_with_registry, bootstrap_builtin_registry,
    build_code_search_response, create_sample_julia_repo, create_sample_modelica_repo,
    ensure_linked_modelica_parser_summary_service, ensure_linked_parser_summary_service,
    publish_repository_snapshot, repo_code_document, test_studio_state,
};

#[tokio::test]
#[serial_test::serial(julia_live)]
async fn build_code_search_response_returns_hits_for_plain_julia_plugin_repository()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_parser_summary_service()?;
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "SearchJulia")?;
    let repository = RegisteredRepository {
        id: "julia-live".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    };
    let registry = bootstrap_builtin_registry()?;
    let analysis =
        analyze_registered_repository_with_registry(&repository, temp.path(), &registry)?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(xiuxian_wendao::search::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![xiuxian_wendao::search::contracts::UiRepoProjectConfig {
            id: repository.id.clone(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia".to_string()],
        }],
    });
    publish_repository_snapshot(
        &studio,
        &repository.id,
        analysis,
        vec![repo_code_document(
            &repo_dir,
            repo_dir.join("src/SearchJulia.jl"),
            "julia",
        )?],
    )
    .await;

    let response = build_code_search_response(&studio, "solve".to_string(), Some("julia-live"), 10)
        .await
        .unwrap_or_else(|error| panic!("Julia code search response: {error:?}"));

    assert!(
        response.hits.iter().any(
            |hit| hit.doc_type.as_deref() == Some("symbol") && hit.path == "src/SearchJulia.jl"
        ),
        "expected Julia symbol hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(julia_live)]
async fn build_code_search_response_returns_hits_for_plain_modelica_plugin_repository()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "SearchModelica")?;
    let repository = RegisteredRepository {
        id: "modelica-live".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
    };
    let registry = bootstrap_builtin_registry()?;
    let analysis =
        analyze_registered_repository_with_registry(&repository, temp.path(), &registry)?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(xiuxian_wendao::search::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![xiuxian_wendao::search::contracts::UiRepoProjectConfig {
            id: repository.id.clone(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["modelica".to_string()],
        }],
    });
    publish_repository_snapshot(
        &studio,
        &repository.id,
        analysis,
        vec![
            repo_code_document(&repo_dir, repo_dir.join("package.mo"), "modelica")?,
            repo_code_document(&repo_dir, repo_dir.join("Controllers/PI.mo"), "modelica")?,
        ],
    )
    .await;

    let response = build_code_search_response(&studio, "PI".to_string(), Some("modelica-live"), 10)
        .await
        .unwrap_or_else(|error| panic!("Modelica code search response: {error:?}"));

    assert!(
        response.hits.iter().any(
            |hit| hit.doc_type.as_deref() == Some("symbol") && hit.path == "Controllers/PI.mo"
        ),
        "expected Modelica symbol hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
    Ok(())
}
