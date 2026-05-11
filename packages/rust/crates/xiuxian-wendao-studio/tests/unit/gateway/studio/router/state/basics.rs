use crate::contracts::UiProjectConfig;
use crate::studio::router::state::lifecycle::{
    gateway_bootstrap_background_indexing_with_lookup,
    gateway_start_bootstrap_background_indexing_with_lookup,
};
use crate::studio::router::state::project_config::graph_include_dirs;
use crate::studio::router::state::{GatewayState, StudioState, supported_code_kinds};
use std::sync::Arc;

#[test]
fn supported_code_kinds_contains_reference_and_doc() {
    let kinds = supported_code_kinds();
    assert!(kinds.iter().any(|kind| kind == "reference"));
    assert!(kinds.iter().any(|kind| kind == "doc"));
}

#[test]
fn graph_include_dirs_deduplicates_normalized_paths() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().to_path_buf();
    let config_root = temp_dir.path().to_path_buf();
    std::fs::create_dir_all(temp_dir.path().join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::create_dir_all(temp_dir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src dir: {error}"));

    let projects = vec![UiProjectConfig {
        name: "kernel".to_string(),
        root: ".".to_string(),
        dirs: vec![
            "docs".to_string(),
            "./docs".to_string(),
            "src".to_string(),
            "src/".to_string(),
        ],
    }];

    let include_dirs = graph_include_dirs(
        project_root.as_path(),
        config_root.as_path(),
        projects.as_slice(),
    );

    assert_eq!(include_dirs, vec!["docs".to_string(), "src".to_string()]);
}

#[test]
fn studio_state_bootstrap_background_indexing_defaults_to_disabled() {
    assert!(!gateway_bootstrap_background_indexing_with_lookup(&|_| {
        None
    }));
    assert!(!gateway_bootstrap_background_indexing_with_lookup(&|_| {
        Some("false".to_string())
    }));
    assert!(!gateway_bootstrap_background_indexing_with_lookup(&|_| {
        Some("invalid".to_string())
    }));
}

#[test]
fn gateway_start_bootstrap_background_indexing_defaults_to_enabled() {
    assert!(gateway_start_bootstrap_background_indexing_with_lookup(
        &|_| None
    ));
    assert!(gateway_start_bootstrap_background_indexing_with_lookup(
        &|_| Some("invalid".to_string())
    ));
}

#[test]
fn gateway_bootstrap_background_indexing_accepts_truthy_env_values() {
    assert!(gateway_bootstrap_background_indexing_with_lookup(&|_| {
        Some("true".to_string())
    }));
    assert!(gateway_bootstrap_background_indexing_with_lookup(&|_| {
        Some(" YES ".to_string())
    }));
    assert!(gateway_bootstrap_background_indexing_with_lookup(&|_| {
        Some("1".to_string())
    }));
}

#[test]
fn gateway_start_bootstrap_background_indexing_accepts_falsy_env_values() {
    assert!(!gateway_start_bootstrap_background_indexing_with_lookup(
        &|_| Some("false".to_string())
    ));
    assert!(!gateway_start_bootstrap_background_indexing_with_lookup(
        &|_| Some(" OFF ".to_string())
    ));
    assert!(!gateway_start_bootstrap_background_indexing_with_lookup(
        &|_| Some("0".to_string())
    ));
}

#[test]
fn bootstrap_background_indexing_telemetry_reports_default_deferred_state() {
    let studio = StudioState::new();
    let telemetry = studio.bootstrap_background_indexing_telemetry();
    let cold_start = studio.search_cold_start_telemetry();

    assert!(!telemetry.enabled());
    assert_eq!(telemetry.mode(), "deferred");
    assert!(!telemetry.deferred_activation_observed());
    assert_eq!(telemetry.deferred_activation_at(), None);
    assert_eq!(telemetry.deferred_activation_source(), None);
    assert_eq!(cold_start.cold_start_window_ms, 60_000);
    assert!(cold_start.cold_start_window_open);
    assert_eq!(cold_start.corpora.len(), 4);
    assert_eq!(
        cold_start
            .diagnostics
            .repeat_work
            .summary
            .repeated_file_observations,
        0
    );
    assert!(
        cold_start
            .diagnostics
            .repeat_work
            .source_operations
            .is_empty()
    );
    assert!(cold_start.diagnostics.repeat_work.hot_paths.is_empty());
    assert!(cold_start.diagnostics.repeat_work.findings.is_empty());
    assert!(cold_start.corpora.iter().all(|corpus| {
        corpus.first_index_started.is_none()
            && corpus.first_partial_search_response.is_none()
            && corpus.first_ready_search_response.is_none()
            && corpus
                .first_ready_observed
                .as_ref()
                .is_none_or(|event| event.source.as_deref() == Some("search_plane_bootstrap"))
    }));
}

#[test]
fn gateway_start_state_defaults_to_enabled_bootstrap_background_indexing() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let config_path = temp_dir.path().join("wendao.toml");
    let state = GatewayState::new_for_gateway_start(
        None,
        None,
        None,
        Some(config_path.as_path()),
        Arc::new(xiuxian_wendao::analyzers::PluginRegistry::new()),
    );

    assert!(state.studio.bootstrap_background_indexing_enabled());
    assert_eq!(state.studio.bootstrap_background_indexing_mode(), "enabled");
}
