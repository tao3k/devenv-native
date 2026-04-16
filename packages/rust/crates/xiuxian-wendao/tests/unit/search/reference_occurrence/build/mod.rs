use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;

use crate::gateway::studio::types::UiProjectConfig;
use crate::search::cache::SearchPlaneCache;
use crate::search::local_symbol::plan_local_symbol_build;
use crate::search::reference_occurrence::build::{
    ensure_reference_occurrence_index_started, fingerprint_projects,
    plan_reference_occurrence_build,
};
use crate::search::reference_occurrence::search_reference_occurrences;
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlanePhase,
    SearchPlaneService,
};

fn planning_service(project_root: &Path) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        project_root.to_path_buf(),
        project_root.join(".data/search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:search_plane:reference-occurrence-plan"),
        SearchMaintenancePolicy::default(),
    )
}

async fn wait_for_reference_occurrence_ready(
    service: &SearchPlaneService,
    previous_epoch: Option<u64>,
) {
    for _ in 0..100 {
        let status = service
            .coordinator()
            .status_for(SearchCorpusKind::ReferenceOccurrence);
        if status.phase == SearchPlanePhase::Ready
            && status.active_epoch.is_some()
            && previous_epoch.is_none_or(|epoch| status.active_epoch.unwrap_or_default() > epoch)
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("reference occurrence build did not reach ready state");
}

fn repeat_work_demo_fixture() -> (
    tempfile::TempDir,
    std::path::PathBuf,
    Vec<UiProjectConfig>,
    SearchPlaneService,
) {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().to_path_buf();
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn alpha() {}\nfn use_alpha() { alpha(); }\n",
    )
    .unwrap_or_else(|error| panic!("write lib: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = planning_service(project_root.as_path());
    (temp_dir, project_root, projects, service)
}

#[test]
fn plan_reference_occurrence_build_only_reparses_changed_files() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn alpha() {}\nfn use_alpha() { alpha(); }\n",
    )
    .unwrap_or_else(|error| panic!("write lib: {error}"));
    std::fs::write(
        project_root.join("src/extra.rs"),
        "fn gamma() {}\nfn use_gamma() { gamma(); }\n",
    )
    .unwrap_or_else(|error| panic!("write extra: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = planning_service(project_root);

    let first = plan_reference_occurrence_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    assert_eq!(first.base_epoch, None);
    assert!(
        first
            .changed_hits
            .iter()
            .any(|hit| hit.path == "src/lib.rs" && hit.name == "alpha")
    );
    assert!(
        first
            .changed_hits
            .iter()
            .any(|hit| hit.path == "src/extra.rs" && hit.name == "gamma")
    );

    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn beta() {}\nfn use_beta() { beta(); }\n",
    )
    .unwrap_or_else(|error| panic!("rewrite lib: {error}"));

    let second = plan_reference_occurrence_build(
        &service,
        project_root,
        project_root,
        &projects,
        Some(7),
        &first.file_fingerprints,
    );
    assert_eq!(second.base_epoch, Some(7));
    assert_eq!(
        second.replaced_paths,
        BTreeSet::from(["src/lib.rs".to_string()])
    );
    assert!(
        second
            .changed_hits
            .iter()
            .all(|hit| hit.path == "src/lib.rs")
    );
    assert!(
        second.changed_hits.iter().any(|hit| hit.name == "beta"),
        "changed-file rebuild must include the updated token"
    );
    assert!(
        second.changed_hits.iter().all(|hit| hit.name != "gamma"),
        "unchanged file rows must not be reparsed into the changed set"
    );
}

#[test]
fn plan_reference_occurrence_build_ignores_metadata_only_edits_when_hits_are_unchanged() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn alpha() {}\nfn use_alpha() { alpha(); }\n",
    )
    .unwrap_or_else(|error| panic!("write lib: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = planning_service(project_root);

    let first = plan_reference_occurrence_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    let first_fingerprint = first
        .file_fingerprints
        .get("src/lib.rs")
        .unwrap_or_else(|| panic!("initial reference occurrence fingerprint"));

    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn alpha() {}\nfn use_alpha() { alpha(); }\n\n",
    )
    .unwrap_or_else(|error| panic!("rewrite lib: {error}"));

    let second = plan_reference_occurrence_build(
        &service,
        project_root,
        project_root,
        &projects,
        Some(7),
        &first.file_fingerprints,
    );
    let second_fingerprint = second
        .file_fingerprints
        .get("src/lib.rs")
        .unwrap_or_else(|| panic!("updated reference occurrence fingerprint"));

    assert_eq!(second.base_epoch, Some(7));
    assert!(second.replaced_paths.is_empty());
    assert!(second.changed_hits.is_empty());
    assert_ne!(first_fingerprint.size_bytes, second_fingerprint.size_bytes);
    assert_eq!(first_fingerprint.blake3, second_fingerprint.blake3);
}

#[tokio::test]
async fn reference_occurrence_incremental_refresh_reuses_unchanged_rows() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn alpha() {}\nfn use_alpha() { alpha(); }\n",
    )
    .unwrap_or_else(|error| panic!("write lib: {error}"));
    std::fs::write(
        project_root.join("src/extra.rs"),
        "fn gamma() {}\nfn use_gamma() { gamma(); }\n",
    )
    .unwrap_or_else(|error| panic!("write extra: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let keyspace =
        SearchManifestKeyspace::new("xiuxian:test:search_plane:reference-occurrence-incremental");
    let cache = SearchPlaneCache::for_tests(keyspace.clone());
    let service = SearchPlaneService::with_runtime(
        project_root.clone(),
        storage_root,
        keyspace,
        SearchMaintenancePolicy::default(),
        cache,
    );

    ensure_reference_occurrence_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_reference_occurrence_ready(&service, None).await;

    let initial_gamma = search_reference_occurrences(&service, "gamma", 10)
        .await
        .unwrap_or_else(|error| panic!("query gamma: {error}"));
    assert_eq!(initial_gamma.len(), 1);
    let initial_alpha = search_reference_occurrences(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query alpha: {error}"));
    assert_eq!(initial_alpha.len(), 1);

    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn beta() {}\nfn use_beta() { beta(); }\n",
    )
    .unwrap_or_else(|error| panic!("rewrite lib: {error}"));
    ensure_reference_occurrence_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_reference_occurrence_ready(&service, Some(1)).await;

    let gamma = search_reference_occurrences(&service, "gamma", 10)
        .await
        .unwrap_or_else(|error| panic!("query gamma after refresh: {error}"));
    assert_eq!(gamma.len(), 1);
    let beta = search_reference_occurrences(&service, "beta", 10)
        .await
        .unwrap_or_else(|error| panic!("query beta after refresh: {error}"));
    assert_eq!(beta.len(), 1);
    let alpha = search_reference_occurrences(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query alpha after refresh: {error}"));
    assert!(alpha.is_empty());
    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::ReferenceOccurrence)
        .active_epoch
        .unwrap_or_else(|| panic!("reference occurrence active epoch"));
    assert!(
        service
            .local_epoch_parquet_path(SearchCorpusKind::ReferenceOccurrence, active_epoch)
            .exists(),
        "missing reference occurrence parquet export"
    );
    assert_no_reference_occurrence_lance_tables(&service);
}

#[test]
fn fingerprint_projects_changes_when_scanned_file_metadata_changes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::create_dir_all(project_root.join("node_modules/pkg"))
        .unwrap_or_else(|error| panic!("create skipped dir: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn alpha() {}\nfn use_alpha() { alpha(); }\n",
    )
    .unwrap_or_else(|error| panic!("write rust source: {error}"));
    std::fs::write(
        project_root.join("node_modules/pkg/index.js"),
        "ignored();\n",
    )
    .unwrap_or_else(|error| panic!("write skipped file: {error}"));

    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];

    let first = fingerprint_projects(project_root, project_root, &projects);
    std::fs::write(
        project_root.join("node_modules/pkg/index.js"),
        "ignored-again();\n",
    )
    .unwrap_or_else(|error| panic!("rewrite skipped file: {error}"));
    let after_skipped_change = fingerprint_projects(project_root, project_root, &projects);
    assert_eq!(first, after_skipped_change);

    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn beta() {}\nfn use_beta() { beta(); }\n",
    )
    .unwrap_or_else(|error| panic!("rewrite rust source: {error}"));
    let second = fingerprint_projects(project_root, project_root, &projects);
    assert_ne!(first, second);
}

#[test]
fn repeat_work_telemetry_exposes_cross_corpus_code_hot_paths() {
    let (_temp_dir, project_root, projects, service) = repeat_work_demo_fixture();

    let _ = plan_local_symbol_build(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
        None,
        &BTreeMap::new(),
    );
    let _ = plan_reference_occurrence_build(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
        None,
        &BTreeMap::new(),
    );

    let telemetry = service.repeat_work_telemetry();
    for (operation, message) in [
        (
            "read_ast_extract",
            "shared source snapshot should build code AST extraction once",
        ),
        (
            "cache_hit",
            "second corpus should reuse the shared source snapshot",
        ),
        (
            "cache_miss",
            "first source snapshot request should record the cache miss",
        ),
    ] {
        assert!(
            telemetry.source_operations.iter().any(|entry| {
                entry.source == "source_snapshot"
                    && entry.operation == operation
                    && entry.file_observation_count == 1
            }),
            "{message}"
        );
    }
    assert!(
        telemetry.hot_paths.iter().any(|entry| {
            entry.path == "src/lib.rs"
                && entry.observations >= 3
                && entry.source_count >= 1
                && entry
                    .sources
                    .iter()
                    .any(|source| source == "source_snapshot")
                && entry
                    .operations
                    .iter()
                    .any(|operation| operation == "read_ast_extract")
                && entry
                    .operations
                    .iter()
                    .any(|operation| operation == "cache_hit")
        }),
        "telemetry should surface the shared snapshot path and its reuse operations"
    );
    assert!(
        telemetry.findings.iter().any(|entry| {
            entry.kind == "cross_operation_hot_path"
                && entry.path.as_deref() == Some("src/lib.rs")
                && entry.observations >= 3
                && entry
                    .sources
                    .iter()
                    .any(|source| source == "source_snapshot")
                && entry
                    .operations
                    .iter()
                    .any(|operation| operation == "read_ast_extract")
                && entry
                    .operations
                    .iter()
                    .any(|operation| operation == "cache_hit")
        }),
        "repeat-detect findings should flag the shared source snapshot hot path"
    );
    assert!(
        telemetry.source_operations.iter().all(|entry| {
            !(entry.source == "reference_occurrence.extract"
                || entry.source == "local_symbol.extract")
        }),
        "heavy code extraction should move to the shared source snapshot layer"
    );
}

#[test]
fn local_symbol_and_reference_occurrence_share_source_snapshot_entries() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn alpha() {}\nfn use_alpha() { alpha(); }\n",
    )
    .unwrap_or_else(|error| panic!("write lib: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = planning_service(project_root);

    let _ = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    assert_eq!(service.source_snapshot_entry_cache_len(), 1);

    let _ = plan_reference_occurrence_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    assert_eq!(service.source_snapshot_entry_cache_len(), 1);
}

#[tokio::test]
async fn reference_occurrence_runtime_build_reuses_fingerprint_scan_inventory() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(
        project_root.join("src/lib.rs"),
        "fn alpha() {}\nfn use_alpha() { alpha(); }\n",
    )
    .unwrap_or_else(|error| panic!("write lib: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:search_plane:reference-occurrence-scan-reuse"),
        SearchMaintenancePolicy::default(),
    );

    ensure_reference_occurrence_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_reference_occurrence_ready(&service, None).await;

    let telemetry = service.repeat_work_telemetry();
    assert!(
        telemetry.source_operations.iter().any(|entry| {
            entry.source == "reference_occurrence.fingerprint"
                && entry.operation == "scan_source_project_files"
        }),
        "runtime build should still record the fingerprint scan inventory"
    );
    assert!(
        telemetry.source_operations.iter().all(|entry| {
            !(entry.source == "reference_occurrence.plan"
                && entry.operation == "scan_source_project_files")
        }),
        "runtime build should reuse fingerprint scan inventory instead of rescanning at plan time"
    );
}

fn assert_no_reference_occurrence_lance_tables(service: &SearchPlaneService) {
    let corpus_root = service.corpus_root(SearchCorpusKind::ReferenceOccurrence);
    let entries = std::fs::read_dir(corpus_root.as_path())
        .unwrap_or_else(|error| panic!("read reference occurrence corpus root: {error}"));
    for entry in entries {
        let entry =
            entry.unwrap_or_else(|error| panic!("read reference occurrence corpus entry: {error}"));
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        assert!(
            !file_name.ends_with(".lance"),
            "unexpected Lance table left behind for reference_occurrence: {file_name}"
        );
    }
}
