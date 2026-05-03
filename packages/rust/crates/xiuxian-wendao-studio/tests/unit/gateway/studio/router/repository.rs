use crate::studio::router::{StudioState, configured_repository, resolve_registered_repository_id};
use xiuxian_wendao::analyzers::RegisteredRepository;
use xiuxian_wendao::search::contracts::{UiConfig, UiRepoProjectConfig};

#[test]
fn resolve_registered_repository_id_prefers_toml_repository_id() {
    let repositories = vec![RegisteredRepository {
        id: "lance".to_string(),
        url: Some("https://github.com/lance-format/lance".to_string()),
        ..RegisteredRepository::default()
    }];

    assert_eq!(
        resolve_registered_repository_id(repositories.as_slice(), "lance").as_deref(),
        Some("lance")
    );
}

#[test]
fn configured_repository_accepts_toml_repository_id() {
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

    let repository = configured_repository(&state, "lance")
        .unwrap_or_else(|error| panic!("configured repository should resolve: {error}"));

    assert_eq!(repository.id, "lance");
}
