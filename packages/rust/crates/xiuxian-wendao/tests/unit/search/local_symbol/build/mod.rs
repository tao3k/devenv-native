use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;

use crate::gateway::studio::types::UiProjectConfig;
use crate::search::cache::SearchPlaneCache;
use crate::search::local_symbol::build::{
    LocalSymbolBuildPlan, LocalSymbolPartitionBuildPlan, ensure_local_symbol_index_started,
    fingerprint_projects, plan_local_symbol_build,
};
use crate::search::local_symbol::search_local_symbols;
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlanePhase,
    SearchPlaneService,
};

fn planning_service(project_root: &Path) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        project_root.to_path_buf(),
        project_root.join(".data/search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:search_plane:local-symbol-plan"),
        SearchMaintenancePolicy::default(),
    )
}

#[test]
fn fingerprint_projects_changes_when_scanned_file_metadata_changes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::create_dir_all(project_root.join("node_modules/pkg"))
        .unwrap_or_else(|error| panic!("create skipped dir: {error}"));
    std::fs::write(project_root.join("src/lib.rs"), "fn alpha() {}\n")
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
        "fn alpha() {}\nfn beta() {}\n",
    )
    .unwrap_or_else(|error| panic!("rewrite rust source: {error}"));
    let second = fingerprint_projects(project_root, project_root, &projects);
    assert_ne!(first, second);
}

#[test]
fn plan_local_symbol_build_only_reparses_changed_files() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(project_root.join("src/lib.rs"), "fn alpha() {}\n")
        .unwrap_or_else(|error| panic!("write lib: {error}"));
    std::fs::write(project_root.join("src/extra.rs"), "fn gamma() {}\n")
        .unwrap_or_else(|error| panic!("write extra: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = planning_service(project_root);

    let first = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    assert_eq!(first.base_epoch, None);
    assert_eq!(count_changed_hits(&first), 2);

    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(project_root.join("src/lib.rs"), "fn beta() {}\n")
        .unwrap_or_else(|error| panic!("rewrite lib: {error}"));

    let second = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        Some(7),
        &first.file_fingerprints,
    );
    assert_eq!(second.base_epoch, Some(7));
    let changed_partition = only_partition(&second);
    assert_eq!(
        changed_partition.replaced_paths,
        BTreeSet::from(["src/lib.rs".to_string()])
    );
    assert_eq!(changed_partition.changed_hits.len(), 1);
    assert_eq!(changed_partition.changed_hits[0].name, "beta");
}

#[test]
fn plan_local_symbol_build_ignores_metadata_only_edits_when_ast_hits_are_unchanged() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(project_root.join("src/lib.rs"), "fn alpha() {}\n")
        .unwrap_or_else(|error| panic!("write lib: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = planning_service(project_root);

    let first = plan_local_symbol_build(
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
        .unwrap_or_else(|| panic!("initial local symbol fingerprint"));

    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(project_root.join("src/lib.rs"), "fn alpha() {}\n\n")
        .unwrap_or_else(|error| panic!("rewrite lib: {error}"));

    let second = plan_local_symbol_build(
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
        .unwrap_or_else(|| panic!("updated local symbol fingerprint"));

    assert_eq!(second.base_epoch, Some(7));
    assert!(second.partitions.is_empty());
    assert_ne!(first_fingerprint.size_bytes, second_fingerprint.size_bytes);
    assert_eq!(first_fingerprint.blake3, second_fingerprint.blake3);
}

#[test]
fn local_symbol_build_ignores_metadata_only_markdown_edits_when_symbol_surface_is_unchanged() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs: {error}"));
    std::fs::write(
        project_root.join("docs/guide.md"),
        concat!(
            "# Alpha\n\n",
            "Body text.\n\n",
            "- [ ] Ship parser lane\n\n",
            "## Overview\n",
            ":PROPERTIES:\n",
            ":ID: alpha\n",
            ":OBSERVE: lang:rust \"fn $NAME()\"\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write guide: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = planning_service(project_root);

    let first = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    let first_fingerprint = first
        .file_fingerprints
        .get("docs/guide.md")
        .unwrap_or_else(|| panic!("initial markdown local symbol fingerprint"));

    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(
        project_root.join("docs/guide.md"),
        concat!(
            "# Alpha\n\n",
            "Body text with more prose.\n\n",
            "\n",
            "- [ ] Ship parser lane\n\n",
            "## Overview\n",
            ":PROPERTIES:\n",
            ":ID: alpha\n",
            ":OBSERVE: lang:rust \"fn $NAME()\"\n",
            ":END:\n\n",
        ),
    )
    .unwrap_or_else(|error| panic!("rewrite guide: {error}"));

    let second = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        Some(7),
        &first.file_fingerprints,
    );
    let second_fingerprint = second
        .file_fingerprints
        .get("docs/guide.md")
        .unwrap_or_else(|| panic!("updated markdown local symbol fingerprint"));

    assert_eq!(second.base_epoch, Some(7));
    assert!(second.partitions.is_empty());
    assert_ne!(first_fingerprint.size_bytes, second_fingerprint.size_bytes);
    assert_eq!(first_fingerprint.blake3, second_fingerprint.blake3);
}

#[test]
fn local_symbol_build_reparses_markdown_files_when_symbol_surface_changes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs: {error}"));
    std::fs::write(
        project_root.join("docs/guide.md"),
        concat!(
            "# Alpha\n\n",
            "- [ ] Ship parser lane\n\n",
            "## Overview\n",
            ":PROPERTIES:\n",
            ":ID: alpha\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write guide: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let service = planning_service(project_root);

    let first = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );

    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(
        project_root.join("docs/guide.md"),
        concat!(
            "# Alpha\n\n",
            "- [ ] Ship parser lane\n\n",
            "## Implementation\n",
            ":PROPERTIES:\n",
            ":ID: beta\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("rewrite guide: {error}"));

    let second = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        Some(7),
        &first.file_fingerprints,
    );

    assert_eq!(second.base_epoch, Some(7));
    let changed_partition = only_partition(&second);
    assert_eq!(
        changed_partition.replaced_paths,
        BTreeSet::from(["docs/guide.md".to_string()])
    );
    assert!(
        changed_partition
            .changed_hits
            .iter()
            .any(|hit| hit.name == "Implementation"),
        "expected changed markdown heading hit"
    );
}

#[tokio::test]
async fn local_symbol_incremental_refresh_reuses_unchanged_rows() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(project_root.join("src/lib.rs"), "fn alpha() {}\n")
        .unwrap_or_else(|error| panic!("write lib: {error}"));
    std::fs::write(project_root.join("src/extra.rs"), "fn gamma() {}\n")
        .unwrap_or_else(|error| panic!("write extra: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let keyspace =
        SearchManifestKeyspace::new("xiuxian:test:search_plane:local-symbol-incremental");
    let cache = SearchPlaneCache::for_tests(keyspace.clone());
    let service = SearchPlaneService::with_runtime(
        project_root.clone(),
        storage_root,
        keyspace,
        SearchMaintenancePolicy::default(),
        cache,
    );

    ensure_local_symbol_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_local_symbol_ready(&service, None).await;

    let initial_gamma = search_local_symbols(&service, "gamma", 10)
        .await
        .unwrap_or_else(|error| panic!("query gamma: {error}"));
    assert_eq!(initial_gamma.len(), 1);
    let initial_alpha = search_local_symbols(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query alpha: {error}"));
    assert_eq!(initial_alpha.len(), 1);

    std::fs::write(project_root.join("src/lib.rs"), "fn beta() {}\n")
        .unwrap_or_else(|error| panic!("rewrite lib: {error}"));
    ensure_local_symbol_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_local_symbol_ready(&service, Some(1)).await;

    let gamma = search_local_symbols(&service, "gamma", 10)
        .await
        .unwrap_or_else(|error| panic!("query gamma after refresh: {error}"));
    assert_eq!(gamma.len(), 1);
    let beta = search_local_symbols(&service, "beta", 10)
        .await
        .unwrap_or_else(|error| panic!("query beta after refresh: {error}"));
    assert_eq!(beta.len(), 1);
    let alpha = search_local_symbols(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query alpha after refresh: {error}"));
    assert!(alpha.is_empty());
    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::LocalSymbol)
        .active_epoch
        .unwrap_or_else(|| panic!("local symbol active epoch"));
    let table_names =
        service.local_epoch_table_names_for_reads(SearchCorpusKind::LocalSymbol, active_epoch);
    assert!(
        !table_names.is_empty(),
        "expected local symbol partition tables"
    );
    for table_name in table_names {
        assert!(
            service
                .local_table_parquet_path(SearchCorpusKind::LocalSymbol, table_name.as_str())
                .exists(),
            "missing local symbol parquet export for {table_name}"
        );
    }
    assert_no_local_symbol_lance_tables(&service);
}

#[tokio::test]
async fn local_symbol_build_writes_partitioned_epoch_tables_for_multiple_scopes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("packages/alpha/src"))
        .unwrap_or_else(|error| panic!("create alpha: {error}"));
    std::fs::create_dir_all(project_root.join("packages/beta/src"))
        .unwrap_or_else(|error| panic!("create beta: {error}"));
    std::fs::write(
        project_root.join("packages/alpha/src/lib.rs"),
        "fn alpha() {}\n",
    )
    .unwrap_or_else(|error| panic!("write alpha: {error}"));
    std::fs::write(
        project_root.join("packages/beta/src/lib.rs"),
        "fn beta() {}\n",
    )
    .unwrap_or_else(|error| panic!("write beta: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec!["packages/alpha".to_string(), "packages/beta".to_string()],
    }];
    let service = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:search_plane:local-symbol-partitioned-build"),
        SearchMaintenancePolicy::default(),
    );

    ensure_local_symbol_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_local_symbol_ready(&service, None).await;

    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::LocalSymbol)
        .active_epoch
        .unwrap_or_default();
    let table_names =
        service.local_epoch_table_names_for_reads(SearchCorpusKind::LocalSymbol, active_epoch);
    assert_eq!(table_names.len(), 2);
    for table_name in &table_names {
        assert!(
            service
                .local_table_parquet_path(SearchCorpusKind::LocalSymbol, table_name.as_str())
                .exists(),
            "missing local symbol parquet export for {table_name}"
        );
    }
    assert_no_local_symbol_lance_tables(&service);

    let alpha = search_local_symbols(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query alpha: {error}"));
    assert_eq!(alpha.len(), 1);
    assert_eq!(alpha[0].project_name.as_deref(), Some("demo"));
    assert_eq!(alpha[0].root_label.as_deref(), Some("alpha"));

    let beta = search_local_symbols(&service, "beta", 10)
        .await
        .unwrap_or_else(|error| panic!("query beta: {error}"));
    assert_eq!(beta.len(), 1);
    assert_eq!(beta[0].project_name.as_deref(), Some("demo"));
    assert_eq!(beta[0].root_label.as_deref(), Some("beta"));
}

#[tokio::test]
async fn local_symbol_build_with_no_supported_sources_publishes_empty_epoch() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(&project_root)
        .unwrap_or_else(|error| panic!("create workspace root: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    let keyspace =
        SearchManifestKeyspace::new("xiuxian:test:search_plane:local-symbol-empty-epoch");
    let cache = SearchPlaneCache::for_tests(keyspace.clone());
    let service = SearchPlaneService::with_runtime(
        project_root.clone(),
        storage_root,
        keyspace,
        SearchMaintenancePolicy::default(),
        cache,
    );

    ensure_local_symbol_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_local_symbol_ready(&service, None).await;

    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::LocalSymbol)
        .active_epoch
        .unwrap_or_else(|| panic!("local symbol active epoch"));
    let table_names =
        service.local_epoch_table_names_for_reads(SearchCorpusKind::LocalSymbol, active_epoch);
    assert!(
        !table_names.is_empty(),
        "expected empty local symbol publication"
    );
    for table_name in &table_names {
        assert!(
            service
                .local_table_parquet_path(SearchCorpusKind::LocalSymbol, table_name.as_str())
                .exists(),
            "missing empty local symbol parquet export for {table_name}"
        );
    }

    let results = search_local_symbols(&service, "alpha", 10)
        .await
        .unwrap_or_else(|error| panic!("query empty local symbol epoch: {error}"));
    assert!(results.is_empty());
}

fn count_changed_hits(plan: &LocalSymbolBuildPlan) -> usize {
    plan.partitions
        .values()
        .map(|partition| partition.changed_hits.len())
        .sum()
}

fn only_partition(plan: &LocalSymbolBuildPlan) -> &LocalSymbolPartitionBuildPlan {
    assert_eq!(plan.partitions.len(), 1);
    let Some(partition) = plan.partitions.values().next() else {
        panic!("single partition");
    };
    partition
}

fn assert_no_local_symbol_lance_tables(service: &SearchPlaneService) {
    let corpus_root = service.corpus_root(SearchCorpusKind::LocalSymbol);
    let entries = std::fs::read_dir(corpus_root.as_path())
        .unwrap_or_else(|error| panic!("read local symbol corpus root: {error}"));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("read local symbol corpus entry: {error}"));
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        assert!(
            !file_name.ends_with(".lance"),
            "unexpected Lance table left behind for local_symbol: {file_name}"
        );
    }
}

async fn wait_for_local_symbol_ready(service: &SearchPlaneService, previous_epoch: Option<u64>) {
    for _ in 0..100 {
        let status = service
            .coordinator()
            .status_for(SearchCorpusKind::LocalSymbol);
        if status.phase == SearchPlanePhase::Ready
            && status.active_epoch.is_some()
            && previous_epoch.is_none_or(|epoch| status.active_epoch.unwrap_or_default() > epoch)
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("local symbol build did not reach ready state");
}
