use super::{
    ARTISAN_STATE_ROOT_DIR_NAME, default_contract_feedback_storage_path_with, must_ok,
    resolve_artisan_state_root_with, resolve_workspace_root,
};
use std::path::{Path, PathBuf};

#[test]
fn default_contract_feedback_storage_path_uses_artisan_state_root() {
    let workspace_root = Path::new("/repo/workspace");
    let resolved = default_contract_feedback_storage_path_with(workspace_root, None);
    assert_eq!(
        resolved,
        PathBuf::from("/repo/workspace/home")
            .join(ARTISAN_STATE_ROOT_DIR_NAME)
            .join("state")
            .join("xiuxian-qianji")
            .join("contract_feedback")
    );
}

#[test]
fn resolve_artisan_state_root_resolves_relative_override_against_workspace_root() {
    let resolved =
        resolve_artisan_state_root_with(Path::new("/repo/workspace"), Some(".runtime/state"));
    assert_eq!(
        resolved,
        PathBuf::from("/repo/workspace/.runtime/state/state")
    );
}

#[test]
fn resolve_artisan_state_root_accepts_absolute_override() {
    let resolved =
        resolve_artisan_state_root_with(Path::new("/repo/workspace"), Some("/tmp/foreign-state"));
    assert_eq!(resolved, PathBuf::from("/tmp/foreign-state/state"));
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
