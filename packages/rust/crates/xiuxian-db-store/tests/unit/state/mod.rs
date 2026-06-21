use std::fs;

use tempfile::tempdir;

use super::{
    ProjectCacheRootConfig, STATE_STORE_DIR_NAME, STATE_STORE_DUCKDB_FILE_NAME,
    discover_git_toplevel_from, project_cache_root_from_config, project_namespace_from_root,
    sanitize_project_namespace,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn project_cache_root_adds_repository_namespace_under_cache_home() -> TestResult {
    let tempdir = tempdir()?;
    let project_root = tempdir.path().join("xiuxian-artisan-workshop");
    let cache_home = project_root.join(".cache");
    fs::create_dir_all(&cache_home)?;

    let root = project_cache_root_from_config(ProjectCacheRootConfig {
        project_root: Some(project_root),
        cache_home: Some(cache_home.clone()),
        project_namespace: None,
    });

    assert_eq!(root, cache_home.join("xiuxian-artisan-workshop"));
    Ok(())
}

#[test]
fn project_cache_root_does_not_duplicate_existing_namespace() -> TestResult {
    let tempdir = tempdir()?;
    let project_root = tempdir.path().join("xiuxian-artisan-workshop");
    let cache_home = project_root.join(".cache").join("xiuxian-artisan-workshop");

    let root = project_cache_root_from_config(ProjectCacheRootConfig {
        project_root: Some(project_root),
        cache_home: Some(cache_home.clone()),
        project_namespace: None,
    });

    assert_eq!(root, cache_home);
    Ok(())
}

#[test]
fn project_cache_root_uses_explicit_namespace_when_provided() -> TestResult {
    let tempdir = tempdir()?;
    let project_root = tempdir.path().join("repo");
    let cache_home = project_root.join(".cache");

    let root = project_cache_root_from_config(ProjectCacheRootConfig {
        project_root: Some(project_root),
        cache_home: Some(cache_home.clone()),
        project_namespace: Some("CyberXiuXian Artisan workshop".to_owned()),
    });

    assert_eq!(root, cache_home.join("CyberXiuXian-Artisan-workshop"));
    Ok(())
}

#[test]
fn state_store_duckdb_path_shape_is_stable() -> TestResult {
    let tempdir = tempdir()?;
    let project_root = tempdir.path().join("xiuxian-artisan-workshop");
    let cache_home = project_root.join(".cache");

    let root = project_cache_root_from_config(ProjectCacheRootConfig {
        project_root: Some(project_root),
        cache_home: Some(cache_home),
        project_namespace: None,
    });

    assert_eq!(
        root.join(STATE_STORE_DIR_NAME)
            .join(STATE_STORE_DUCKDB_FILE_NAME),
        root.join("state").join("state.duckdb")
    );
    Ok(())
}

#[test]
fn git_toplevel_discovery_accepts_git_directory_marker() -> TestResult {
    let tempdir = tempdir()?;
    let repo = tempdir.path().join("xiuxian-artisan-workshop");
    let nested = repo.join("packages/rust/crates");
    fs::create_dir_all(repo.join(".git"))?;
    fs::create_dir_all(&nested)?;

    assert_eq!(discover_git_toplevel_from(&nested), Some(repo));
    Ok(())
}

#[test]
fn git_toplevel_discovery_accepts_git_file_marker() -> TestResult {
    let tempdir = tempdir()?;
    let repo = tempdir.path().join("xiuxian-artisan-workshop");
    let nested = repo.join("packages/rust/crates");
    fs::create_dir_all(&nested)?;
    fs::write(repo.join(".git"), "gitdir: ../worktrees/current\n")?;

    assert_eq!(discover_git_toplevel_from(&nested), Some(repo));
    Ok(())
}

#[test]
fn project_namespace_sanitizes_repository_root_name() {
    let namespace = project_namespace_from_root("/tmp/CyberXiuXian Artisan workshop");
    assert_eq!(namespace, "CyberXiuXian-Artisan-workshop");
}

#[test]
fn empty_project_namespace_is_rejected() {
    assert_eq!(sanitize_project_namespace(" @@@ "), None);
}
