use crate::contracts::UiRepoProjectConfig;

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
