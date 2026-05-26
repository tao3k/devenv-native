use super::{
    default_contract_feedback_storage_path_with, must_ok, resolve_prj_data_home_with,
    resolve_workspace_root,
};
use std::path::{Path, PathBuf};

#[test]
fn default_contract_feedback_storage_path_uses_workspace_data_root() {
    let workspace_root = Path::new("/repo/workspace");
    let resolved = default_contract_feedback_storage_path_with(workspace_root, None);
    assert_eq!(
        resolved,
        PathBuf::from("/repo/workspace/.data/xiuxian-qianji/contract_feedback")
    );
}

#[test]
fn resolve_prj_data_home_resolves_relative_override_against_workspace_root() {
    let resolved = resolve_prj_data_home_with(Path::new("/repo/workspace"), Some(".runtime/data"));
    assert_eq!(resolved, PathBuf::from("/repo/workspace/.runtime/data"));
}

#[test]
fn resolve_prj_data_home_accepts_absolute_override() {
    let resolved =
        resolve_prj_data_home_with(Path::new("/repo/workspace"), Some("/tmp/foreign-data"));
    assert_eq!(resolved, PathBuf::from("/tmp/foreign-data"));
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
