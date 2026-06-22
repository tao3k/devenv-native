use std::fs;

use tempfile::tempdir;

use super::{
    ARTISAN_STATE_ROOT_DIR_NAME, ArtisanStateRootConfig, STATE_STORE_DIR_NAME,
    STATE_STORE_DUCKDB_FILE_NAME, artisan_state_root_from_config, discover_git_toplevel_from,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn artisan_state_root_uses_home_scoped_directory_by_default() -> TestResult {
    let tempdir = tempdir()?;
    let project_root = tempdir.path().join("xiuxian-artisan-workshop");
    let home_dir = tempdir.path().join("home");

    let root = artisan_state_root_from_config(ArtisanStateRootConfig {
        project_root: Some(project_root),
        state_root: None,
        home_dir: Some(home_dir.clone()),
    });

    assert_eq!(root, home_dir.join(ARTISAN_STATE_ROOT_DIR_NAME));
    Ok(())
}

#[test]
fn artisan_state_root_respects_absolute_explicit_root() -> TestResult {
    let tempdir = tempdir()?;
    let project_root = tempdir.path().join("xiuxian-artisan-workshop");
    let state_root = tempdir.path().join("state-root");

    let root = artisan_state_root_from_config(ArtisanStateRootConfig {
        project_root: Some(project_root),
        state_root: Some(state_root.clone()),
        home_dir: None,
    });

    assert_eq!(root, state_root);
    Ok(())
}

#[test]
fn artisan_state_root_resolves_relative_explicit_root_against_project_root() -> TestResult {
    let tempdir = tempdir()?;
    let project_root = tempdir.path().join("repo");

    let root = artisan_state_root_from_config(ArtisanStateRootConfig {
        project_root: Some(project_root.clone()),
        state_root: Some(".local-state".into()),
        home_dir: None,
    });

    assert_eq!(root, project_root.join(".local-state"));
    Ok(())
}

#[test]
fn state_store_duckdb_path_shape_is_stable() -> TestResult {
    let tempdir = tempdir()?;
    let project_root = tempdir.path().join("xiuxian-artisan-workshop");
    let home_dir = tempdir.path().join("home");

    let root = artisan_state_root_from_config(ArtisanStateRootConfig {
        project_root: Some(project_root),
        state_root: None,
        home_dir: Some(home_dir),
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
