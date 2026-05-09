use crate::contracts::{UiConfig, UiRepoProjectConfig};
use crate::studio::router::StudioState;

pub(crate) fn studio_with_repo_projects(repo_projects: Vec<UiRepoProjectConfig>) -> StudioState {
    let studio = StudioState::new();
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: Vec::new(),
        repo_projects,
    });
    studio
}

pub(crate) fn repo_project(id: &str) -> UiRepoProjectConfig {
    UiRepoProjectConfig {
        id: id.to_string(),
        root: Some(".".to_string()),
        url: None,
        git_ref: None,
        refresh: None,
        plugins: vec!["julia-code-parser".to_string()],
    }
}
