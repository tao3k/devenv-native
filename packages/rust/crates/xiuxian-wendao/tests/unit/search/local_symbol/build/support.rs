use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use crate::search::cache::SearchPlaneCache;
use crate::search::contracts::SearchProjectConfig;
use crate::search::local_symbol::build::{
    LocalSymbolBuildPlan, LocalSymbolPartitionBuildPlan, ensure_local_symbol_index_started,
};
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlanePhase,
    SearchPlaneService,
};

pub(super) fn demo_projects() -> Vec<SearchProjectConfig> {
    vec![SearchProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }]
}

pub(super) fn planning_service(project_root: &Path) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        project_root.to_path_buf(),
        project_root.join(".data/search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:search_plane:local-symbol-plan"),
        SearchMaintenancePolicy::default(),
    )
}

pub(super) fn incremental_service(
    project_root: &Path,
    storage_root: &Path,
    keyspace_name: &str,
) -> SearchPlaneService {
    let keyspace = SearchManifestKeyspace::new(keyspace_name);
    let cache = SearchPlaneCache::for_tests(keyspace.clone());
    SearchPlaneService::with_runtime(
        project_root.to_path_buf(),
        storage_root.to_path_buf(),
        keyspace,
        SearchMaintenancePolicy::default(),
        cache,
    )
}

pub(super) fn count_changed_hits(plan: &LocalSymbolBuildPlan) -> usize {
    plan.partitions
        .values()
        .map(|partition| partition.changed_hits.len())
        .sum()
}

pub(super) fn only_partition(plan: &LocalSymbolBuildPlan) -> &LocalSymbolPartitionBuildPlan {
    assert_eq!(plan.partitions.len(), 1);
    let Some(partition) = plan.partitions.values().next() else {
        panic!("single partition");
    };
    partition
}

pub(super) fn assert_no_local_symbol_lance_tables(service: &SearchPlaneService) {
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

pub(super) async fn wait_for_local_symbol_ready(
    service: &SearchPlaneService,
    previous_epoch: Option<u64>,
) {
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

pub(super) fn write_demo_source(project_root: &Path, relative_path: &str, content: &str) {
    let absolute_path = project_root.join(relative_path);
    if let Some(parent) = absolute_path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("create parent {}: {error}", parent.display()));
    }
    std::fs::write(&absolute_path, content)
        .unwrap_or_else(|error| panic!("write {}: {error}", absolute_path.display()));
}

pub(super) fn singleton_replaced_path(path: &str) -> BTreeSet<String> {
    BTreeSet::from([path.to_string()])
}

pub(super) async fn start_local_symbol_index(
    service: &SearchPlaneService,
    project_root: &Path,
    projects: &[SearchProjectConfig],
) {
    ensure_local_symbol_index_started(service, project_root, project_root, projects);
    wait_for_local_symbol_ready(service, None).await;
}
