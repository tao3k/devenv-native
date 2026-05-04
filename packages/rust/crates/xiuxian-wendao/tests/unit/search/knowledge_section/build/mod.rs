use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;

use crate::search::cache::SearchPlaneCache;
use crate::search::contracts::SearchProjectConfig;
use crate::search::knowledge_section::search_knowledge_sections;
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlanePhase,
    SearchPlaneService,
};

use super::orchestration::{ensure_knowledge_section_index_started, plan_knowledge_section_build};
use super::paths::fingerprint_projects;

fn planning_service(project_root: &Path) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        project_root.to_path_buf(),
        project_root.join(".data/search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:search_plane:knowledge-section-plan"),
        SearchMaintenancePolicy::default(),
    )
}

#[test]
fn plan_knowledge_section_build_only_reparses_changed_notes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("notes"))
        .unwrap_or_else(|error| panic!("create notes: {error}"));
    std::fs::write(
        project_root.join("notes/alpha.md"),
        "# Alpha\n\nAlpha body.\n\n## Overview\n\nAlpha section.\n",
    )
    .unwrap_or_else(|error| panic!("write alpha note: {error}"));
    std::fs::write(
        project_root.join("notes/gamma.md"),
        "# Gamma\n\nGamma body.\n\n## Overview\n\nGamma section.\n",
    )
    .unwrap_or_else(|error| panic!("write gamma note: {error}"));
    let projects = vec![SearchProjectConfig {
        name: "notes".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = planning_service(project_root);

    let first = plan_knowledge_section_build(
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
            .changed_rows
            .iter()
            .any(|row| row.path == "notes/alpha.md")
    );
    assert!(
        first
            .changed_rows
            .iter()
            .any(|row| row.path == "notes/gamma.md")
    );

    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(
        project_root.join("notes/alpha.md"),
        "# Beta\n\nBeta body.\n\n## Overview\n\nBeta section.\n",
    )
    .unwrap_or_else(|error| panic!("rewrite alpha note: {error}"));

    let second = plan_knowledge_section_build(
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
        BTreeSet::from(["notes/alpha.md".to_string()])
    );
    assert!(
        second
            .changed_rows
            .iter()
            .all(|row| row.path == "notes/alpha.md")
    );
    assert!(
        second
            .changed_rows
            .iter()
            .any(|row| row.search_text.contains("Beta"))
    );
    assert!(
        second
            .changed_rows
            .iter()
            .all(|row| !row.path.contains("gamma")),
        "unchanged note rows must not be reparsed into the changed set"
    );
}

#[test]
fn plan_knowledge_section_build_ignores_metadata_only_edits_when_rows_are_unchanged() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("notes"))
        .unwrap_or_else(|error| panic!("create notes: {error}"));
    std::fs::write(
        project_root.join("notes/alpha.md"),
        "# Alpha\n\nAlpha body.\n\n## Overview\n\nAlpha section.\n",
    )
    .unwrap_or_else(|error| panic!("write alpha note: {error}"));
    let projects = vec![SearchProjectConfig {
        name: "notes".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = planning_service(project_root);

    let first = plan_knowledge_section_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    let first_fingerprint = first
        .file_fingerprints
        .get("notes/alpha.md")
        .unwrap_or_else(|| panic!("initial knowledge fingerprint"));

    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(
        project_root.join("notes/alpha.md"),
        "# Alpha\n\nAlpha body.\n\n## Overview\n\nAlpha section.\n\n",
    )
    .unwrap_or_else(|error| panic!("rewrite alpha note: {error}"));

    let second = plan_knowledge_section_build(
        &service,
        project_root,
        project_root,
        &projects,
        Some(7),
        &first.file_fingerprints,
    );
    let second_fingerprint = second
        .file_fingerprints
        .get("notes/alpha.md")
        .unwrap_or_else(|| panic!("updated knowledge fingerprint"));

    assert_eq!(second.base_epoch, Some(7));
    assert!(second.replaced_paths.is_empty());
    assert!(second.changed_rows.is_empty());
    assert_ne!(first_fingerprint.size_bytes, second_fingerprint.size_bytes);
    assert_eq!(first_fingerprint.blake3, second_fingerprint.blake3);
}

#[test]
fn note_corpora_and_local_symbol_share_markdown_snapshot_entries() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs: {error}"));
    std::fs::write(
        project_root.join("docs/design.md"),
        "# Design\n\n:owner: kernel\n\n## Evidence\n\n:OBSERVE: lang:rust \"fn $NAME()\"\n",
    )
    .unwrap_or_else(|error| panic!("write design note: {error}"));
    let projects = vec![SearchProjectConfig {
        name: "kernel".to_string(),
        root: ".".to_string(),
        dirs: vec!["docs".to_string()],
    }];
    let service = planning_service(project_root);

    let knowledge = plan_knowledge_section_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    assert_eq!(service.markdown_snapshot_entry_cache_len(), 1);
    assert!(
        knowledge
            .changed_rows
            .iter()
            .any(|row| row.path == "docs/design.md")
    );
    let snapshot_entry = service.shared_markdown_snapshot_entry(
        project_root,
        &crate::search::scan_note_project_files(project_root, project_root, &projects)[0],
    );
    assert!(
        snapshot_entry.note_fingerprint.is_some(),
        "markdown snapshot should cache the parser-owned note fingerprint"
    );

    let attachment = crate::search::attachment::plan_attachment_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    assert_eq!(service.markdown_snapshot_entry_cache_len(), 1);
    assert!(attachment.changed_hits.is_empty());

    let local_symbol = crate::search::local_symbol::plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    assert_eq!(service.markdown_snapshot_entry_cache_len(), 1);
    assert!(
        local_symbol
            .partitions
            .values()
            .flat_map(|partition| partition.changed_hits.iter())
            .any(|hit| hit.path == "docs/design.md" && hit.language == "markdown")
    );
    assert!(
        local_symbol
            .partitions
            .values()
            .flat_map(|partition| partition.changed_hits.iter())
            .any(|hit| {
                hit.path == "docs/design.md"
                    && hit.node_kind.as_deref() == Some("property")
                    && hit.name.eq_ignore_ascii_case("owner")
                    && hit.signature.ends_with(" kernel")
            }),
        "markdown snapshot should preserve parser-owned property hits"
    );
    assert!(
        local_symbol
            .partitions
            .values()
            .flat_map(|partition| partition.changed_hits.iter())
            .any(|hit| {
                hit.path == "docs/design.md"
                    && hit.node_kind.as_deref() == Some("observation")
                    && hit.name == "OBSERVE"
                    && hit.signature.contains("lang:rust")
            }),
        "markdown snapshot should preserve parser-owned observation hits"
    );
}

#[tokio::test]
async fn knowledge_section_runtime_build_reuses_fingerprint_scan_inventory() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("notes"))
        .unwrap_or_else(|error| panic!("create notes: {error}"));
    std::fs::write(
        project_root.join("notes/alpha.md"),
        "# Alpha\n\nAlpha body.\n\n## Overview\n\nAlpha section.\n",
    )
    .unwrap_or_else(|error| panic!("write alpha note: {error}"));
    let projects = vec![SearchProjectConfig {
        name: "notes".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:search_plane:knowledge-section-scan-reuse"),
        SearchMaintenancePolicy::default(),
    );

    ensure_knowledge_section_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_knowledge_section_ready(&service, None).await;

    let telemetry = service.repeat_work_telemetry();
    assert!(
        telemetry.source_operations.iter().any(|entry| {
            entry.source == "knowledge_section.fingerprint"
                && entry.operation == "scan_note_project_files"
        }),
        "runtime build should still record the fingerprint scan inventory"
    );
    assert!(
        telemetry.source_operations.iter().all(|entry| {
            !(entry.source == "knowledge_section.plan"
                && entry.operation == "scan_note_project_files")
        }),
        "runtime build should reuse fingerprint scan inventory instead of rescanning at plan time"
    );
}

#[tokio::test]
async fn knowledge_section_incremental_refresh_reuses_unchanged_rows() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("notes"))
        .unwrap_or_else(|error| panic!("create notes: {error}"));
    std::fs::write(
        project_root.join("notes/alpha.md"),
        "# Alpha\n\nAlpha body.\n\n## Overview\n\nAlpha section.\n",
    )
    .unwrap_or_else(|error| panic!("write alpha note: {error}"));
    std::fs::write(
        project_root.join("notes/gamma.md"),
        "# Gamma\n\nGamma body.\n\n## Overview\n\nGamma section.\n",
    )
    .unwrap_or_else(|error| panic!("write gamma note: {error}"));
    let projects = vec![SearchProjectConfig {
        name: "notes".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let keyspace =
        SearchManifestKeyspace::new("xiuxian:test:search_plane:knowledge-section-incremental");
    let cache = SearchPlaneCache::for_tests(keyspace.clone());
    let service = SearchPlaneService::with_runtime(
        project_root.clone(),
        storage_root,
        keyspace,
        SearchMaintenancePolicy::default(),
        cache,
    );

    ensure_knowledge_section_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_knowledge_section_ready(&service, None).await;

    let initial_gamma = search_knowledge_sections(&service, "Gamma body", 10)
        .await
        .unwrap_or_else(|error| panic!("query gamma: {error}"));
    assert_eq!(initial_gamma.len(), 1);
    let initial_alpha = search_knowledge_sections(&service, "Alpha body", 10)
        .await
        .unwrap_or_else(|error| panic!("query alpha: {error}"));
    assert_eq!(initial_alpha.len(), 1);

    std::fs::write(
        project_root.join("notes/alpha.md"),
        "# Beta\n\nBeta body.\n\n## Overview\n\nBeta section.\n",
    )
    .unwrap_or_else(|error| panic!("rewrite alpha note: {error}"));
    ensure_knowledge_section_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_knowledge_section_ready(&service, Some(1)).await;

    let gamma = search_knowledge_sections(&service, "Gamma body", 10)
        .await
        .unwrap_or_else(|error| panic!("query gamma after refresh: {error}"));
    assert_eq!(gamma.len(), 1);
    let beta = search_knowledge_sections(&service, "Beta body", 10)
        .await
        .unwrap_or_else(|error| panic!("query beta after refresh: {error}"));
    assert_eq!(beta.len(), 1);
    let alpha = search_knowledge_sections(&service, "Alpha body", 10)
        .await
        .unwrap_or_else(|error| panic!("query alpha after refresh: {error}"));
    assert!(alpha.is_empty());
    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::KnowledgeSection)
        .active_epoch
        .unwrap_or_else(|| panic!("knowledge section active epoch"));
    assert!(
        service
            .local_epoch_parquet_path(SearchCorpusKind::KnowledgeSection, active_epoch)
            .exists(),
        "missing knowledge section parquet export"
    );
    assert_no_knowledge_section_lance_tables(&service);
}

#[tokio::test]
async fn knowledge_section_build_with_no_supported_notes_publishes_empty_epoch() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(&project_root)
        .unwrap_or_else(|error| panic!("create workspace root: {error}"));
    let projects = vec![SearchProjectConfig {
        name: "notes".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let keyspace =
        SearchManifestKeyspace::new("xiuxian:test:search_plane:knowledge-section-empty-epoch");
    let cache = SearchPlaneCache::for_tests(keyspace.clone());
    let service = SearchPlaneService::with_runtime(
        project_root.clone(),
        storage_root,
        keyspace,
        SearchMaintenancePolicy::default(),
        cache,
    );

    ensure_knowledge_section_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_knowledge_section_ready(&service, None).await;

    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::KnowledgeSection)
        .active_epoch
        .unwrap_or_else(|| panic!("knowledge section active epoch"));
    assert!(
        service
            .local_epoch_parquet_path(SearchCorpusKind::KnowledgeSection, active_epoch)
            .exists(),
        "missing empty knowledge section parquet export"
    );

    let results = search_knowledge_sections(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query empty knowledge section epoch: {error}"));
    assert!(results.is_empty());
}

async fn wait_for_knowledge_section_ready(
    service: &SearchPlaneService,
    previous_epoch: Option<u64>,
) {
    for _ in 0..100 {
        let status = service
            .coordinator()
            .status_for(SearchCorpusKind::KnowledgeSection);
        if status.phase == SearchPlanePhase::Ready
            && status.active_epoch.is_some()
            && previous_epoch.is_none_or(|epoch| status.active_epoch.unwrap_or_default() > epoch)
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("knowledge section build did not reach ready state");
}

fn assert_no_knowledge_section_lance_tables(service: &SearchPlaneService) {
    let corpus_root = service.corpus_root(SearchCorpusKind::KnowledgeSection);
    let entries = std::fs::read_dir(corpus_root.as_path())
        .unwrap_or_else(|error| panic!("read knowledge-section corpus root: {error}"));
    for entry in entries {
        let entry =
            entry.unwrap_or_else(|error| panic!("read knowledge-section corpus entry: {error}"));
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        assert!(
            !file_name.ends_with(".lance"),
            "unexpected Lance table left behind for knowledge_section: {file_name}"
        );
    }
}

#[test]
fn fingerprint_projects_changes_when_scanned_note_metadata_changes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("notes"))
        .unwrap_or_else(|error| panic!("create notes: {error}"));
    std::fs::create_dir_all(project_root.join("node_modules/pkg"))
        .unwrap_or_else(|error| panic!("create skipped dir: {error}"));
    std::fs::write(
        project_root.join("notes/alpha.md"),
        "# Alpha\n\nAlpha body.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));
    std::fs::write(
        project_root.join("node_modules/pkg/ignored.md"),
        "# Ignored\n",
    )
    .unwrap_or_else(|error| panic!("write skipped file: {error}"));

    let projects = vec![SearchProjectConfig {
        name: "notes".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];

    let first = fingerprint_projects(project_root, project_root, &projects);
    std::fs::write(
        project_root.join("node_modules/pkg/ignored.md"),
        "# Still Ignored\n",
    )
    .unwrap_or_else(|error| panic!("rewrite skipped note: {error}"));
    let after_skipped_change = fingerprint_projects(project_root, project_root, &projects);
    assert_eq!(first, after_skipped_change);

    std::fs::write(
        project_root.join("notes/alpha.md"),
        "# Beta\n\nBeta body.\n",
    )
    .unwrap_or_else(|error| panic!("rewrite note: {error}"));
    let second = fingerprint_projects(project_root, project_root, &projects);
    assert_ne!(first, second);
}
