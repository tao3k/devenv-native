use super::{
    default_contract_feedback_storage_path_with, must_ok, resolve_project_state_root_with,
    resolve_workspace_root,
};
use std::path::{Path, PathBuf};

#[test]
fn default_contract_feedback_storage_path_uses_workspace_state_root() {
    let workspace_root = Path::new("/repo/workspace");
    let resolved = default_contract_feedback_storage_path_with(workspace_root, None);
    assert_eq!(
        resolved,
        PathBuf::from("/repo/workspace/.cache/workspace/state/xiuxian-qianji/contract_feedback")
    );
}

#[test]
fn resolve_project_state_root_resolves_relative_cache_override_against_workspace_root() {
    let resolved =
        resolve_project_state_root_with(Path::new("/repo/workspace"), Some(".runtime/cache"));
    assert_eq!(
        resolved,
        PathBuf::from("/repo/workspace/.runtime/cache/workspace/state")
    );
}

#[test]
fn resolve_project_state_root_accepts_absolute_cache_override() {
    let resolved =
        resolve_project_state_root_with(Path::new("/repo/workspace"), Some("/tmp/foreign-cache"));
    assert_eq!(
        resolved,
        PathBuf::from("/tmp/foreign-cache/workspace/state")
    );
}

#[test]
fn resolve_workspace_root_prefers_explicit_path() {
    let explicit = Path::new("/tmp/explicit-workspace");
    let resolved = must_ok(
        resolve_workspace_root(Some(explicit)),
        "resolve workspace root",
    );
    assert_eq!(resolved, explicit);
}
