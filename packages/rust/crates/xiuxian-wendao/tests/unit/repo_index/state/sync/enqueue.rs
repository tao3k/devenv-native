use super::support::{
    PathBuf, RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
    SearchPlaneService, new_coordinator, repo,
};

#[test]
fn sync_repositories_only_enqueues_new_or_changed_repositories() {
    let coordinator = new_coordinator(SearchPlaneService::new(PathBuf::from(".")));

    let first = coordinator.sync_repositories(vec![repo("sciml", "./sciml")]);
    let second = coordinator.sync_repositories(vec![repo("sciml", "./sciml")]);
    let third = coordinator.sync_repositories(vec![repo("sciml", "./sciml-next")]);

    assert_eq!(first, vec!["sciml".to_string()]);
    assert!(second.is_empty());
    assert_eq!(third, vec!["sciml".to_string()]);
}

#[test]
fn sync_repositories_enqueues_search_only_repositories_for_repo_backed_search() {
    let coordinator = new_coordinator(SearchPlaneService::new(PathBuf::from(".")));
    let repository = RegisteredRepository {
        id: "sciml".to_string(),
        path: Some(PathBuf::from("./sciml")),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("repo-content".to_string())],
    };

    let enqueued = coordinator.sync_repositories(vec![repository]);

    assert_eq!(enqueued, vec!["sciml".to_string()]);
    assert_eq!(
        coordinator.pending_repo_ids_for_test(),
        vec!["sciml".to_string()]
    );
    assert_eq!(coordinator.status_response(None).total, 1);
}

#[test]
fn sync_repositories_reenqueues_repositories_when_configured_plugin_set_changes() {
    let coordinator = new_coordinator(SearchPlaneService::new(PathBuf::from(".")));
    let repository = repo("sciml", "./sciml");
    let repository_with_repo_content = RegisteredRepository {
        plugins: vec![
            RepositoryPluginConfig::Id("julia-code-parser".to_string()),
            RepositoryPluginConfig::Id("repo-content".to_string()),
        ],
        ..repository.clone()
    };

    let first = coordinator.sync_repositories(vec![repository]);
    let second = coordinator.sync_repositories(vec![repository_with_repo_content]);

    assert_eq!(first, vec!["sciml".to_string()]);
    assert_eq!(second, vec!["sciml".to_string()]);
    assert_eq!(
        coordinator.pending_repo_ids_for_test(),
        vec!["sciml".to_string()]
    );
}

#[test]
fn sync_repositories_does_not_reenqueue_repositories_when_configured_plugin_order_only_changes() {
    let coordinator = new_coordinator(SearchPlaneService::new(PathBuf::from(".")));
    let repository = RegisteredRepository {
        plugins: vec![
            RepositoryPluginConfig::Id("julia-code-parser".to_string()),
            RepositoryPluginConfig::Id("repo-content".to_string()),
            RepositoryPluginConfig::Config {
                id: "modelica".to_string(),
                options: serde_json::json!({
                    "mode": "parser-summary"
                }),
            },
        ],
        ..repo("sciml", "./sciml")
    };
    let reordered_repository = RegisteredRepository {
        plugins: vec![
            RepositoryPluginConfig::Config {
                id: "modelica".to_string(),
                options: serde_json::json!({
                    "mode": "doc-surface"
                }),
            },
            RepositoryPluginConfig::Id("julia-code-parser".to_string()),
            RepositoryPluginConfig::Id("repo-content".to_string()),
            RepositoryPluginConfig::Id("repo-content".to_string()),
        ],
        ..repository.clone()
    };

    let first = coordinator.sync_repositories(vec![repository]);
    let second = coordinator.sync_repositories(vec![reordered_repository]);

    assert_eq!(first, vec!["sciml".to_string()]);
    assert!(second.is_empty());
}
