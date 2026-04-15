use crate::gateway::studio::router::StudioState;
use crate::gateway::studio::router::handlers::repo::parse::{
    required_import_search_filters, required_registered_repo_id,
};
use crate::gateway::studio::types::{UiConfig, UiRepoProjectConfig};

#[test]
fn import_search_filters_require_package_or_module() {
    let Err(error) = required_import_search_filters(None, None) else {
        panic!("missing import filters should fail");
    };
    assert_eq!(error.code(), "MISSING_IMPORT_FILTER");
}

#[test]
fn required_registered_repo_id_uses_toml_configured_repository_seed() {
    let state = StudioState::new();
    state.seed_eager_configured_owners_for_tests(UiConfig {
        projects: Vec::new(),
        repo_projects: vec![UiRepoProjectConfig {
            id: "lance".to_string(),
            root: None,
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: Some("manual".to_string()),
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let repo_id = required_registered_repo_id(&state, Some("lance"))
        .unwrap_or_else(|error| panic!("configured repo seed should resolve: {error:?}"));

    assert_eq!(repo_id, "lance");
}
