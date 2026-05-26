use super::{
    resolve_prj_config_home, resolve_prj_data_home, resolve_project_root,
    resolve_project_root_from_value,
};
use crate::runtime_config::model::QianjiRuntimeEnv;
use std::path::Path;

#[test]
fn resolve_project_root_ignores_blank_override_values() {
    let runtime_env = QianjiRuntimeEnv {
        extra_env: vec![("PRJ_ROOT".to_string(), "   ".to_string())],
        ..QianjiRuntimeEnv::default()
    };
    let resolved = resolve_project_root(&runtime_env);
    assert!(!resolved.as_os_str().is_empty());
}

#[test]
fn resolve_prj_config_home_resolves_relative_override_against_project_root() {
    let runtime_env = QianjiRuntimeEnv {
        extra_env: vec![("PRJ_CONFIG_HOME".to_string(), ".config/custom".to_string())],
        ..QianjiRuntimeEnv::default()
    };
    let resolved = resolve_prj_config_home(&runtime_env, Path::new("/repo/project"));
    assert_eq!(resolved, Path::new("/repo/project/.config/custom"));
}

#[test]
fn resolve_prj_data_home_resolves_relative_override_against_project_root() {
    let runtime_env = QianjiRuntimeEnv {
        extra_env: vec![("PRJ_DATA_HOME".to_string(), ".data/custom".to_string())],
        ..QianjiRuntimeEnv::default()
    };
    let resolved = resolve_prj_data_home(&runtime_env, Path::new("/repo/project"));
    assert_eq!(resolved, Path::new("/repo/project/.data/custom"));
}

#[test]
fn resolve_project_root_from_value_resolves_relative_values_against_current_dir() {
    let resolved =
        resolve_project_root_from_value(Some("workspace/root"), Some(Path::new("/repo")));
    assert_eq!(resolved, Path::new("/repo/workspace/root"));
}

#[test]
fn resolve_project_root_from_value_ignores_blank_values() {
    let resolved = resolve_project_root_from_value(Some("   "), Some(Path::new("/repo")));
    assert_eq!(resolved, Path::new("/repo"));
}
