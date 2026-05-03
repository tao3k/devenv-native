use crate::studio::router::{sanitize_projects, sanitize_repo_projects};
use xiuxian_wendao::search::contracts::{UiProjectConfig, UiRepoProjectConfig};

#[test]
fn sanitize_projects_removes_empty_names() {
    let input = vec![
        UiProjectConfig {
            name: String::new(),
            root: ".".to_string(),
            dirs: vec!["src".to_string()],
        },
        UiProjectConfig {
            name: "valid".to_string(),
            root: ".".to_string(),
            dirs: vec!["src".to_string()],
        },
    ];
    let result = sanitize_projects(input);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "valid");
}

#[test]
fn sanitize_projects_removes_duplicates() {
    let input = vec![
        UiProjectConfig {
            name: "dup".to_string(),
            root: ".".to_string(),
            dirs: vec!["src".to_string()],
        },
        UiProjectConfig {
            name: "dup".to_string(),
            root: "./other".to_string(),
            dirs: vec!["lib".to_string()],
        },
    ];
    let result = sanitize_projects(input);
    assert_eq!(result.len(), 1);
}

#[test]
fn sanitize_repo_projects_requires_plugins() {
    let input = vec![UiRepoProjectConfig {
        id: "test".to_string(),
        root: Some(".".to_string()),
        url: None,
        git_ref: None,
        refresh: None,
        plugins: vec![],
    }];
    let result = sanitize_repo_projects(input);
    assert!(result.is_empty());
}

#[test]
fn sanitize_repo_projects_requires_source() {
    let input = vec![UiRepoProjectConfig {
        id: "test".to_string(),
        root: None,
        url: None,
        git_ref: None,
        refresh: None,
        plugins: vec!["julia".to_string()],
    }];
    let result = sanitize_repo_projects(input);
    assert!(result.is_empty());
}
