use crate::contracts::{AstSearchHit, StudioNavigationTarget, UiConfig, UiProjectConfig};
use crate::studio::router::state::StudioState;
use std::sync::Arc;
use xiuxian_wendao::search::{SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace};

use super::support::{
    assert_warm_started_cold_start_telemetry, search_plane_with_paths, wait_for_symbol_index_ready,
    warm_start_writer_corpora,
};

#[tokio::test]
async fn studio_state_records_bootstrap_ready_observation_for_warm_started_local_corpus() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::write(
        project_root.join("docs/intro.md"),
        "# Warm Start\n\nBootstrap should recover this corpus.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));
    let projects = vec![UiProjectConfig {
        name: "docs".to_string(),
        root: ".".to_string(),
        dirs: vec!["docs".to_string()],
    }];
    let writer = search_plane_with_paths(
        project_root.clone(),
        storage_root.clone(),
        "xiuxian:test:studio-state:warm-start-writer",
    );
    writer
        .publish_knowledge_sections_from_projects(
            project_root.as_path(),
            project_root.as_path(),
            &projects,
            "warm-start-bootstrap",
        )
        .await
        .unwrap_or_else(|error| panic!("publish local knowledge sections: {error}"));

    let plugin_registry = Arc::new(
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let reader = search_plane_with_paths(
        project_root.clone(),
        storage_root,
        "xiuxian:test:studio-state:warm-start-reader",
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry,
        project_root.clone(),
        project_root,
        reader,
    );

    let cold_start = studio.search_cold_start_telemetry();
    let knowledge = cold_start
        .corpora
        .iter()
        .find(|corpus| corpus.corpus == SearchCorpusKind::KnowledgeSection.as_str())
        .unwrap_or_else(|| panic!("knowledge_section telemetry should be present"));

    assert!(knowledge.first_index_started.is_none());
    assert_eq!(
        knowledge
            .first_ready_observed
            .as_ref()
            .and_then(|event| event.source.as_deref()),
        Some("search_plane_bootstrap")
    );
}

#[tokio::test]
async fn warm_started_local_corpora_do_not_record_spurious_index_started_events() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src dir: {error}"));
    std::fs::write(
        project_root.join("docs/intro.md"),
        "# Warm Start\n\nPreserve the restored local corpus.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "pub struct WarmStartSymbol;\npub fn warm_start_reference() {}\n",
    )
    .unwrap_or_else(|error| panic!("write source: {error}"));

    let projects = vec![UiProjectConfig {
        name: "kernel".to_string(),
        root: ".".to_string(),
        dirs: vec!["docs".to_string(), "src".to_string()],
    }];
    let writer = search_plane_with_paths(
        project_root.clone(),
        storage_root.clone(),
        "xiuxian:test:studio-state:warm-start-noop-writer",
    );
    warm_start_writer_corpora(&writer, project_root.as_path(), &projects).await;

    let plugin_registry = Arc::new(
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let reader = search_plane_with_paths(
        project_root.clone(),
        storage_root,
        "xiuxian:test:studio-state:warm-start-noop-reader",
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry,
        project_root.clone(),
        project_root,
        reader,
    );

    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: projects.clone(),
        repo_projects: Vec::new(),
    });
    studio
        .ensure_knowledge_section_index_started()
        .unwrap_or_else(|error| panic!("ensure knowledge section index started: {error:?}"));
    studio
        .ensure_local_symbol_index_started()
        .unwrap_or_else(|error| panic!("ensure local symbol index started: {error:?}"));
    studio
        .ensure_attachment_index_started()
        .unwrap_or_else(|error| panic!("ensure attachment index started: {error:?}"));
    studio
        .ensure_reference_occurrence_index_started()
        .unwrap_or_else(|error| panic!("ensure reference occurrence index started: {error:?}"));
    let _ = studio.search_index_status().await;
    assert_warm_started_cold_start_telemetry(&studio);
}

#[tokio::test]
async fn seed_eager_configured_owners_for_tests_warms_symbol_index_from_local_symbol_artifact() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src dir: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "pub fn ArtifactWarmSymbol() {}\n",
    )
    .unwrap_or_else(|error| panic!("write source: {error}"));

    let writer = xiuxian_wendao::search::SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root.clone(),
        SearchManifestKeyspace::new("xiuxian:test:studio-state:symbol-warm-writer"),
        SearchMaintenancePolicy::default(),
    );
    let hits = crate::contracts::domain_ast_hits_for_search_plane(vec![AstSearchHit {
        name: "ArtifactWarmSymbol".to_string(),
        signature: "fn ArtifactWarmSymbol()".to_string(),
        path: "src/lib.rs".to_string(),
        language: "rust".to_string(),
        crate_name: "kernel".to_string(),
        project_name: None,
        root_label: None,
        node_kind: Some("function".to_string()),
        owner_title: None,
        navigation_target: StudioNavigationTarget {
            path: "src/lib.rs".to_string(),
            category: "symbol".to_string(),
            project_name: None,
            root_label: None,
            line: Some(11),
            line_end: Some(11),
            column: Some(1),
        },
        line_start: 11,
        line_end: 11,
        score: 0.0,
    }]);
    writer
        .publish_local_symbol_hits("fp-studio-symbol-warm", hits.as_slice())
        .await
        .unwrap_or_else(|error| panic!("publish local symbol hits: {error}"));

    let plugin_registry = Arc::new(
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let reader = search_plane_with_paths(
        project_root.clone(),
        storage_root,
        "xiuxian:test:studio-state:symbol-warm-reader",
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry,
        project_root.clone(),
        project_root,
        reader,
    );

    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["src".to_string()],
        }],
        repo_projects: Vec::new(),
    });
    wait_for_symbol_index_ready(&studio).await;

    let symbol_index = studio
        .current_symbol_index()
        .unwrap_or_else(|| panic!("studio symbol index should warm from artifact"));
    let results = symbol_index.search_unified("ArtifactWarmSymbol", 10);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "ArtifactWarmSymbol");
    assert!(results[0].location.starts_with("src/lib.rs:"));
}
