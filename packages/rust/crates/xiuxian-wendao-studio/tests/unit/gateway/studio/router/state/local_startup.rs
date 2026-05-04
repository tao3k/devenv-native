use crate::contracts::{UiConfig, UiProjectConfig};
use crate::studio::router::state::StudioState;
use std::sync::Arc;
use xiuxian_wendao::search::SearchCorpusKind;

use super::support::{search_plane_with_paths, wait_for_local_corpus_ready};

#[tokio::test]
async fn seed_eager_configured_owners_for_tests_starts_local_search_plane_indexes_for_configured_projects()
 {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src dir: {error}"));
    std::fs::write(
        project_root.join("docs/intro.md"),
        "# Startup\n\nConfigured local docs should bootstrap search.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "pub struct StartupSymbol;\npub fn startup_reference() {}\n",
    )
    .unwrap_or_else(|error| panic!("write source: {error}"));

    let plugin_registry = Arc::new(
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let search_plane = search_plane_with_paths(
        project_root.clone(),
        storage_root,
        "xiuxian:test:studio-state:configured-owner-startup",
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry,
        project_root.clone(),
        project_root.clone(),
        search_plane,
    );

    studio.seed_eager_configured_owners_for_tests(UiConfig {
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
    assert_eq!(
        telemetry.deferred_activation_source(),
        Some("test_configured_owner_seed")
    );
    for corpus in &cold_start.corpora {
        assert_eq!(
            corpus
                .first_index_started
                .as_ref()
                .and_then(|event| event.source.as_deref()),
            Some("test_configured_owner_seed")
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
async fn seed_eager_configured_owners_for_tests_uses_shared_scan_bundle_for_local_corpus_startup() {
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
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let search_plane = search_plane_with_paths(
        project_root.clone(),
        storage_root,
        "xiuxian:test:studio-state:shared-startup-scan-bundle",
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry,
        project_root.clone(),
        project_root.clone(),
        search_plane,
    );

    studio.seed_eager_configured_owners_for_tests(UiConfig {
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
            entry.source == "test_configured_owner_seed"
                && entry.operation == "scan_supported_project_files"
                && entry.file_observation_count >= 2
        }),
        "fixture seeding should record the shared configured-project scan bundle"
    );
    assert!(
        telemetry.source_operations.iter().all(|entry| {
            let is_per_corpus_scan = match entry.source.as_str() {
                "knowledge_section.fingerprint" | "attachment.fingerprint" => {
                    entry.operation == "scan_note_project_files"
                }
                "local_symbol.fingerprint" => entry.operation == "scan_symbol_project_files",
                "reference_occurrence.fingerprint" => {
                    entry.operation == "scan_source_project_files"
                }
                _ => false,
            };
            !is_per_corpus_scan
        }),
        "fixture seeding should avoid one filesystem walk per corpus"
    );
}

#[tokio::test]
async fn eager_bootstrap_uses_studio_bootstrap_source_instead_of_test_configured_owner_seed() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src dir: {error}"));
    std::fs::write(
        project_root.join("docs/intro.md"),
        "# Studio Bootstrap\n\nStartup should own configured projects directly.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "pub fn studio_bootstrap() {}\n",
    )
    .unwrap_or_else(|error| panic!("write source: {error}"));
    let config_path = project_root.join("wendao.toml");
    std::fs::write(
        &config_path,
        r#"[link_graph.projects.kernel]
root = "."
dirs = ["docs", "src"]
"#,
    )
    .unwrap_or_else(|error| panic!("write wendao config: {error}"));

    let plugin_registry = Arc::new(
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    );
    let search_plane = search_plane_with_paths(
        project_root.clone(),
        storage_root,
        "xiuxian:test:studio-state:bootstrap-runtime-source",
    );
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path_and_background_indexing(
        plugin_registry,
        project_root.clone(),
        project_root.clone(),
        Some(config_path.as_path()),
        search_plane,
        true,
    );

    wait_for_local_corpus_ready(&studio, SearchCorpusKind::KnowledgeSection).await;
    wait_for_local_corpus_ready(&studio, SearchCorpusKind::LocalSymbol).await;
    wait_for_local_corpus_ready(&studio, SearchCorpusKind::Attachment).await;
    wait_for_local_corpus_ready(&studio, SearchCorpusKind::ReferenceOccurrence).await;

    let repeat_work = studio.search_plane.repeat_work_telemetry();
    assert!(
        repeat_work.source_operations.iter().any(|entry| {
            entry.source == "studio_bootstrap"
                && entry.operation == "scan_supported_project_files"
                && entry.file_observation_count >= 2
        }),
        "bootstrap startup should record the shared configured-project scan bundle"
    );
    assert!(
        repeat_work
            .source_operations
            .iter()
            .all(|entry| entry.source != "test_configured_owner_seed"),
        "bootstrap startup should not reuse the retired config-apply source"
    );

    let cold_start = studio.search_cold_start_telemetry();
    for corpus in &cold_start.corpora {
        assert_eq!(
            corpus
                .first_index_started
                .as_ref()
                .and_then(|event| event.source.as_deref()),
            Some("studio_bootstrap")
        );
    }
}
