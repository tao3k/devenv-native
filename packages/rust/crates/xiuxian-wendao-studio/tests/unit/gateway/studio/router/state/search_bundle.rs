use crate::contracts::{UiConfig, UiProjectConfig};
use crate::studio::router::state::StudioState;
use std::sync::Arc;
use xiuxian_wendao::search::{SearchCorpusKind, SearchPlanePhase};

use super::support::search_plane_with_paths;

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
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let search_plane = search_plane_with_paths(
        project_root.clone(),
        storage_root,
        "xiuxian:test:studio-state:note-search-bundle",
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path_and_background_indexing(
        plugin_registry,
        project_root.clone(),
        project_root.clone(),
        None,
        search_plane,
        false,
    );
    studio.seed_configured_owners_for_tests(
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
            let is_per_corpus_note_scan = matches!(
                entry.source.as_str(),
                "knowledge_section.fingerprint" | "attachment.fingerprint"
            ) && entry.operation == "scan_note_project_files";
            !is_per_corpus_note_scan
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
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let search_plane = search_plane_with_paths(
        project_root.clone(),
        storage_root,
        "xiuxian:test:studio-state:code-search-bundle",
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path_and_background_indexing(
        plugin_registry,
        project_root.clone(),
        project_root.clone(),
        None,
        search_plane,
        false,
    );
    studio.seed_configured_owners_for_tests(
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
