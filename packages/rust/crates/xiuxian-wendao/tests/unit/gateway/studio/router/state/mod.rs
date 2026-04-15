use crate::gateway::studio::router::state::helpers::graph_include_dirs;
use crate::gateway::studio::router::state::lifecycle::{
    gateway_bootstrap_background_indexing_with_lookup,
    gateway_start_bootstrap_background_indexing_with_lookup,
};
use crate::gateway::studio::router::state::{GatewayState, StudioState, supported_code_kinds};
use crate::gateway::studio::types::{UiConfig, UiProjectConfig};
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlanePhase,
    SearchPlaneService,
};
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
        &|_| { Some("invalid".to_string()) }
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
        &|_| { Some("false".to_string()) }
    ));
    assert!(!gateway_start_bootstrap_background_indexing_with_lookup(
        &|_| { Some(" OFF ".to_string()) }
    ));
    assert!(!gateway_start_bootstrap_background_indexing_with_lookup(
        &|_| { Some("0".to_string()) }
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
    assert!(
        cold_start
            .corpora
            .iter()
            .all(|corpus| corpus.first_index_started.is_none()
                && corpus.first_ready_observed.is_none()
                && corpus.first_partial_search_response.is_none()
                && corpus.first_ready_search_response.is_none())
    );
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
        Arc::new(crate::analyzers::PluginRegistry::new()),
    );

    assert!(state.studio.bootstrap_background_indexing_enabled());
    assert_eq!(state.studio.bootstrap_background_indexing_mode(), "enabled");
}

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
    let writer = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root.clone(),
        SearchManifestKeyspace::new("xiuxian:test:studio-state:warm-start-writer"),
        SearchMaintenancePolicy::default(),
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
        crate::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let reader = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:studio-state:warm-start-reader"),
        SearchMaintenancePolicy::default(),
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
    let writer = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root.clone(),
        SearchManifestKeyspace::new("xiuxian:test:studio-state:warm-start-noop-writer"),
        SearchMaintenancePolicy::default(),
    );
    assert!(writer.ensure_knowledge_section_index_started(
        project_root.as_path(),
        project_root.as_path(),
        &projects
    ));
    assert!(writer.ensure_attachment_index_started(
        project_root.as_path(),
        project_root.as_path(),
        &projects
    ));
    assert!(writer.ensure_local_symbol_index_started(
        project_root.as_path(),
        project_root.as_path(),
        &projects
    ));
    assert!(writer.ensure_reference_occurrence_index_started(
        project_root.as_path(),
        project_root.as_path(),
        &projects
    ));
    wait_for_search_plane_corpus_ready(&writer, SearchCorpusKind::KnowledgeSection).await;
    wait_for_search_plane_corpus_ready(&writer, SearchCorpusKind::Attachment).await;
    wait_for_search_plane_corpus_ready(&writer, SearchCorpusKind::LocalSymbol).await;
    wait_for_search_plane_corpus_ready(&writer, SearchCorpusKind::ReferenceOccurrence).await;

    let plugin_registry = Arc::new(
        crate::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let reader = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:studio-state:warm-start-noop-reader"),
        SearchMaintenancePolicy::default(),
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry,
        project_root.clone(),
        project_root,
        reader,
    );

    studio.apply_eager_ui_config(UiConfig {
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

    let cold_start = studio.search_cold_start_telemetry();
    for corpus in &cold_start.corpora {
        assert!(
            corpus.first_index_started.is_none(),
            "warm-started corpus `{}` should not record a no-op start",
            corpus.corpus
        );
        assert_eq!(
            corpus
                .first_ready_observed
                .as_ref()
                .and_then(|event| event.source.as_deref()),
            Some("search_plane_bootstrap"),
            "warm-started corpus `{}` should keep bootstrap ready telemetry",
            corpus.corpus
        );
    }
}

#[tokio::test]
async fn config_apply_starts_local_search_plane_indexes_for_configured_projects() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src dir: {error}"));
    std::fs::write(
        project_root.join("docs/intro.md"),
        "# Studio Search\n\nWarm local corpora before the first query.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));
    std::fs::write(project_root.join("src/lib.rs"), "pub fn warmup() {}\n")
        .unwrap_or_else(|error| panic!("write source: {error}"));

    let plugin_registry = Arc::new(
        crate::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let search_plane = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:studio-state:background-local-corpora"),
        SearchMaintenancePolicy::default(),
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry,
        project_root.clone(),
        project_root.clone(),
        search_plane,
    );

    studio.apply_eager_ui_config(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string(), "src".to_string()],
        }],
        repo_projects: Vec::new(),
    });

    wait_for_local_corpus_ready(&studio, SearchCorpusKind::KnowledgeSection).await;
    wait_for_local_corpus_ready(&studio, SearchCorpusKind::LocalSymbol).await;
    wait_for_local_corpus_ready(&studio, SearchCorpusKind::Attachment).await;
    wait_for_local_corpus_ready(&studio, SearchCorpusKind::ReferenceOccurrence).await;
    let expected_ready_observed = [
        SearchCorpusKind::KnowledgeSection,
        SearchCorpusKind::LocalSymbol,
        SearchCorpusKind::Attachment,
        SearchCorpusKind::ReferenceOccurrence,
    ]
    .into_iter()
    .map(|corpus| {
        let status = studio.search_plane.coordinator().status_for(corpus);
        (
            corpus.as_str().to_string(),
            status.build_finished_at.unwrap_or_else(|| {
                panic!("ready corpus `{corpus}` should carry build_finished_at")
            }),
        )
    })
    .collect::<std::collections::BTreeMap<_, _>>();
    let _ = studio.search_index_status().await;

    let telemetry = studio.bootstrap_background_indexing_telemetry();
    let cold_start = studio.search_cold_start_telemetry();
    assert!(telemetry.deferred_activation_observed());
    assert_eq!(telemetry.deferred_activation_source(), Some("config_apply"));
    for corpus in &cold_start.corpora {
        assert_eq!(
            corpus
                .first_index_started
                .as_ref()
                .and_then(|event| event.source.as_deref()),
            Some("config_apply")
        );
        assert_eq!(
            corpus
                .first_ready_observed
                .as_ref()
                .and_then(|event| event.source.as_deref()),
            Some("search_index_status")
        );
        assert_eq!(
            corpus
                .first_ready_observed
                .as_ref()
                .map(|event| event.recorded_at.as_str()),
            expected_ready_observed
                .get(&corpus.corpus)
                .map(String::as_str)
        );
    }
}

#[tokio::test]
async fn config_apply_uses_shared_scan_bundle_for_local_corpus_startup() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src dir: {error}"));
    std::fs::write(
        project_root.join("docs/intro.md"),
        "# Shared Scan\n\nStartup should share one scan inventory.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));
    std::fs::write(project_root.join("src/lib.rs"), "pub fn shared_scan() {}\n")
        .unwrap_or_else(|error| panic!("write source: {error}"));

    let plugin_registry = Arc::new(
        crate::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let search_plane = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:studio-state:shared-startup-scan-bundle"),
        SearchMaintenancePolicy::default(),
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry,
        project_root.clone(),
        project_root.clone(),
        search_plane,
    );

    studio.apply_eager_ui_config(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string(), "src".to_string()],
        }],
        repo_projects: Vec::new(),
    });

    wait_for_local_corpus_ready(&studio, SearchCorpusKind::KnowledgeSection).await;
    wait_for_local_corpus_ready(&studio, SearchCorpusKind::LocalSymbol).await;
    wait_for_local_corpus_ready(&studio, SearchCorpusKind::Attachment).await;
    wait_for_local_corpus_ready(&studio, SearchCorpusKind::ReferenceOccurrence).await;

    let telemetry = studio.search_plane.repeat_work_telemetry();
    assert!(
        telemetry.source_operations.iter().any(|entry| {
            entry.source == "config_apply"
                && entry.operation == "scan_supported_project_files"
                && entry.file_observation_count >= 2
        }),
        "startup should record the shared configured-project scan bundle"
    );
    assert!(
        telemetry.source_operations.iter().all(|entry| {
            !((entry.source == "knowledge_section.fingerprint"
                && entry.operation == "scan_note_project_files")
                || (entry.source == "attachment.fingerprint"
                    && entry.operation == "scan_note_project_files")
                || (entry.source == "local_symbol.fingerprint"
                    && entry.operation == "scan_symbol_project_files")
                || (entry.source == "reference_occurrence.fingerprint"
                    && entry.operation == "scan_source_project_files"))
        }),
        "optimized startup should avoid one filesystem walk per corpus"
    );
}

#[tokio::test]
async fn knowledge_search_uses_shared_note_scan_bundle_and_primes_attachment() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::write(
        project_root.join("docs/intro.md"),
        "# Shared Note Search\n\nOne note search should prime attachments too.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));

    let plugin_registry = Arc::new(
        crate::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let search_plane = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:studio-state:note-search-bundle"),
        SearchMaintenancePolicy::default(),
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path_and_background_indexing(
        plugin_registry,
        project_root.clone(),
        project_root.clone(),
        None,
        search_plane,
        false,
    );
    studio.apply_ui_config(
        UiConfig {
            projects: vec![UiProjectConfig {
                name: "kernel".to_string(),
                root: ".".to_string(),
                dirs: vec!["docs".to_string()],
            }],
            repo_projects: Vec::new(),
        },
        false,
    );

    studio
        .ensure_knowledge_section_index_started()
        .unwrap_or_else(|error| panic!("ensure knowledge section index started: {error:?}"));
    studio
        .ensure_attachment_index_started()
        .unwrap_or_else(|error| panic!("ensure attachment index started: {error:?}"));

    assert_ne!(
        studio
            .search_plane
            .coordinator()
            .status_for(SearchCorpusKind::Attachment)
            .phase,
        SearchPlanePhase::Idle
    );

    let telemetry = studio.search_plane.repeat_work_telemetry();
    assert_eq!(
        telemetry
            .source_operations
            .iter()
            .filter(|entry| {
                entry.source == "note_search_bundle"
                    && entry.operation == "scan_supported_project_files"
            })
            .count(),
        1,
        "paired note search routes should share one scan bundle"
    );
    assert!(
        telemetry.source_operations.iter().all(|entry| {
            !((entry.source == "knowledge_section.fingerprint"
                && entry.operation == "scan_note_project_files")
                || (entry.source == "attachment.fingerprint"
                    && entry.operation == "scan_note_project_files"))
        }),
        "paired note route startup should avoid per-corpus note scans"
    );

    let cold_start = studio.search_cold_start_telemetry();
    for corpus in [
        SearchCorpusKind::KnowledgeSection,
        SearchCorpusKind::Attachment,
    ] {
        let telemetry = cold_start
            .corpora
            .iter()
            .find(|entry| entry.corpus == corpus.as_str())
            .unwrap_or_else(|| panic!("missing cold-start telemetry for `{corpus}`"));
        assert_eq!(
            telemetry
                .first_index_started
                .as_ref()
                .and_then(|event| event.source.as_deref()),
            Some("knowledge_search")
        );
    }
}

#[tokio::test]
async fn reference_search_uses_shared_code_scan_bundle_and_primes_local_symbol() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src dir: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "pub struct BundleSymbol;\npub fn bundle_reference() { let _ = BundleSymbol; }\n",
    )
    .unwrap_or_else(|error| panic!("write source: {error}"));

    let plugin_registry = Arc::new(
        crate::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let search_plane = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:studio-state:code-search-bundle"),
        SearchMaintenancePolicy::default(),
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path_and_background_indexing(
        plugin_registry,
        project_root.clone(),
        project_root.clone(),
        None,
        search_plane,
        false,
    );
    studio.apply_ui_config(
        UiConfig {
            projects: vec![UiProjectConfig {
                name: "kernel".to_string(),
                root: ".".to_string(),
                dirs: vec!["src".to_string()],
            }],
            repo_projects: Vec::new(),
        },
        false,
    );

    studio
        .ensure_reference_occurrence_index_started()
        .unwrap_or_else(|error| panic!("ensure reference occurrence index started: {error:?}"));
    studio
        .ensure_local_symbol_index_started()
        .unwrap_or_else(|error| panic!("ensure local symbol index started: {error:?}"));

    assert_ne!(
        studio
            .search_plane
            .coordinator()
            .status_for(SearchCorpusKind::LocalSymbol)
            .phase,
        SearchPlanePhase::Idle
    );

    let telemetry = studio.search_plane.repeat_work_telemetry();
    assert_eq!(
        telemetry
            .source_operations
            .iter()
            .filter(|entry| {
                entry.source == "code_search_bundle"
                    && entry.operation == "scan_supported_project_files"
            })
            .count(),
        1,
        "paired code search routes should share one scan bundle"
    );
    assert!(
        telemetry.source_operations.iter().all(|entry| {
            !((entry.source == "local_symbol.fingerprint"
                && entry.operation == "scan_symbol_project_files")
                || (entry.source == "reference_occurrence.fingerprint"
                    && entry.operation == "scan_source_project_files"))
        }),
        "paired code route startup should avoid per-corpus code scans"
    );

    let cold_start = studio.search_cold_start_telemetry();
    for corpus in [
        SearchCorpusKind::LocalSymbol,
        SearchCorpusKind::ReferenceOccurrence,
    ] {
        let telemetry = cold_start
            .corpora
            .iter()
            .find(|entry| entry.corpus == corpus.as_str())
            .unwrap_or_else(|| panic!("missing cold-start telemetry for `{corpus}`"));
        assert_eq!(
            telemetry
                .first_index_started
                .as_ref()
                .and_then(|event| event.source.as_deref()),
            Some("reference_search")
        );
    }
}

async fn wait_for_local_corpus_ready(studio: &StudioState, corpus: SearchCorpusKind) {
    for _ in 0..200 {
        let status = studio.search_plane.coordinator().status_for(corpus);
        if status.phase == SearchPlanePhase::Ready && status.active_epoch.is_some() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("search corpus `{corpus}` did not reach ready state");
}

async fn wait_for_search_plane_corpus_ready(
    search_plane: &SearchPlaneService,
    corpus: SearchCorpusKind,
) {
    for _ in 0..200 {
        let status = search_plane.coordinator().status_for(corpus);
        if status.phase == SearchPlanePhase::Ready && status.active_epoch.is_some() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("search-plane corpus `{corpus}` did not reach ready state");
}
