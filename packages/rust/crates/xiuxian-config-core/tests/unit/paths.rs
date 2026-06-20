//! Path-resolution helper regression tests.

use std::path::Path;

use xiuxian_config_core::{
    ProjectDirs, resolve_cache_home_from_value, resolve_home_from_value, resolve_path_from_value,
    resolve_project_root_or_cwd_from_value, resolve_runtime_dir_from_value,
};

#[test]
fn resolve_data_home_uses_project_default_when_env_missing() {
    let root = Path::new("/repo/project");
    let resolved = resolve_home_from_value(Some(root), None, ".data");
    assert_eq!(resolved.as_deref(), Some(Path::new("/repo/project/.data")));
}

#[test]
fn resolve_data_home_resolves_relative_env_against_project_root() {
    let root = Path::new("/repo/project");
    let resolved = resolve_home_from_value(Some(root), Some(".state/data"), ".data");
    assert_eq!(
        resolved.as_deref(),
        Some(Path::new("/repo/project/.state/data"))
    );
}

#[test]
fn resolve_cache_home_respects_absolute_env_value() {
    let root = Path::new("/repo/project");
    let resolved = resolve_home_from_value(Some(root), Some("/tmp/cache-root"), ".cache");
    assert_eq!(resolved.as_deref(), Some(Path::new("/tmp/cache-root")));
}

#[test]
fn resolve_cache_home_from_value_uses_project_default_when_env_missing() {
    let root = Path::new("/repo/project");
    let resolved = resolve_cache_home_from_value(Some(root), None);
    assert_eq!(resolved.as_deref(), Some(Path::new("/repo/project/.cache")));
}

#[test]
fn resolve_cache_home_from_value_resolves_relative_env_against_project_root() {
    let root = Path::new("/repo/project");
    let resolved = resolve_cache_home_from_value(Some(root), Some(".runtime/cache"));
    assert_eq!(
        resolved.as_deref(),
        Some(Path::new("/repo/project/.runtime/cache"))
    );
}

#[test]
fn resolve_runtime_dir_from_value_uses_project_default_when_env_missing() {
    let root = Path::new("/repo/project");
    let resolved = resolve_runtime_dir_from_value(Some(root), None);
    assert_eq!(resolved.as_deref(), Some(Path::new("/repo/project/.run")));
}

#[test]
fn resolve_runtime_dir_from_value_resolves_relative_env_against_project_root() {
    let root = Path::new("/repo/project");
    let resolved = resolve_runtime_dir_from_value(Some(root), Some(".state/run"));
    assert_eq!(
        resolved.as_deref(),
        Some(Path::new("/repo/project/.state/run"))
    );
}

#[test]
fn resolve_config_home_uses_project_default_when_env_missing() {
    let root = Path::new("/repo/project");
    let resolved = resolve_home_from_value(Some(root), None, ".config");
    assert_eq!(
        resolved.as_deref(),
        Some(Path::new("/repo/project/.config"))
    );
}

#[test]
fn resolve_path_from_value_resolves_relative_against_project_root() {
    let root = Path::new("/repo/project");
    let resolved = resolve_path_from_value(Some(root), Some(" .cache/state "));
    assert_eq!(
        resolved.as_deref(),
        Some(Path::new("/repo/project/.cache/state"))
    );
}

#[test]
fn resolve_path_from_value_preserves_absolute_input() {
    let root = Path::new("/repo/project");
    let resolved = resolve_path_from_value(Some(root), Some(" /tmp/cache-root "));
    assert_eq!(resolved.as_deref(), Some(Path::new("/tmp/cache-root")));
}

#[test]
fn resolve_project_root_or_cwd_from_value_uses_relative_env_against_cwd() {
    let cwd = Path::new("/repo/project");
    let resolved = resolve_project_root_or_cwd_from_value(Some("workspace"), Some(cwd));
    assert_eq!(resolved, Path::new("/repo/project/workspace"));
}

#[test]
fn resolve_project_root_or_cwd_from_value_falls_back_to_cwd_when_env_is_missing() {
    let cwd = Path::new("/repo/project");
    let resolved = resolve_project_root_or_cwd_from_value(None, Some(cwd));
    assert_eq!(resolved, Path::new("/repo/project"));
}

#[test]
fn resolve_project_root_or_cwd_from_value_falls_back_to_dot_without_cwd() {
    let resolved = resolve_project_root_or_cwd_from_value(Some("   "), None);
    assert_eq!(resolved, Path::new("."));
}

#[test]
fn project_dirs_from_values_preserves_prj_defaults() {
    let dirs = ProjectDirs::from_values(
        Path::new("/repo/project").to_path_buf(),
        None,
        None,
        None,
        None,
    );

    assert_eq!(dirs.project_root_path(), Path::new("/repo/project"));
    assert_eq!(dirs.config_home_path(), Path::new("/repo/project/.config"));
    assert_eq!(dirs.data_home_path(), Path::new("/repo/project/.data"));
    assert_eq!(dirs.cache_home_path(), Path::new("/repo/project/.cache"));
    assert_eq!(dirs.runtime_dir_path(), Path::new("/repo/project/.run"));
}

#[test]
fn project_dirs_from_values_resolves_relative_prj_values() {
    let dirs = ProjectDirs::from_values(
        Path::new("/repo/project").to_path_buf(),
        Some(".cfg"),
        Some(".state/data"),
        Some(".state/cache"),
        Some(".state/run"),
    );

    assert_eq!(dirs.config_home_path(), Path::new("/repo/project/.cfg"));
    assert_eq!(
        dirs.data_home_path(),
        Path::new("/repo/project/.state/data")
    );
    assert_eq!(
        dirs.cache_home_path(),
        Path::new("/repo/project/.state/cache")
    );
    assert_eq!(
        dirs.runtime_dir_path(),
        Path::new("/repo/project/.state/run")
    );
}

#[test]
fn project_dirs_from_values_preserves_absolute_prj_values() {
    let dirs = ProjectDirs::from_values(
        Path::new("/repo/project").to_path_buf(),
        Some("/tmp/config"),
        Some("/tmp/data"),
        Some("/tmp/cache"),
        Some("/tmp/run"),
    );

    assert_eq!(dirs.config_home_path(), Path::new("/tmp/config"));
    assert_eq!(dirs.data_home_path(), Path::new("/tmp/data"));
    assert_eq!(dirs.cache_home_path(), Path::new("/tmp/cache"));
    assert_eq!(dirs.runtime_dir_path(), Path::new("/tmp/run"));
}
