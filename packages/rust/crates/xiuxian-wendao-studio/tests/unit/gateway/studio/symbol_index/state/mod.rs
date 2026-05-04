use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::contracts::{AstSearchHit, StudioNavigationTarget, UiProjectConfig};
use crate::studio::symbol_index::state::{SymbolIndexCoordinator, fingerprint_projects};
use crate::studio::symbol_index::{SymbolIndexPhase, SymbolIndexStatus};
use xiuxian_wendao::search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService};
use xiuxian_wendao::unified_symbol::UnifiedSymbolIndex;

#[cfg(test)]
impl SymbolIndexCoordinator {
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn set_ready_index_for_test(
        &self,
        projects: &[UiProjectConfig],
        index_cache: Arc<RwLock<Option<Arc<UnifiedSymbolIndex>>>>,
        index: UnifiedSymbolIndex,
    ) {
        *self
            .active_fingerprint
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(fingerprint_projects(projects));
        *index_cache
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::new(index));
        *self
            .status
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = SymbolIndexStatus {
            phase: SymbolIndexPhase::Ready,
            last_error: None,
            updated_at: Some(crate::studio::symbol_index::state::timestamp_now()),
        };
    }
}

#[test]
fn sync_projects_resets_to_idle_when_projects_are_empty() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let coordinator = Arc::new(SymbolIndexCoordinator::new(
        temp.path().to_path_buf(),
        temp.path().to_path_buf(),
        SearchPlaneService::with_paths(
            temp.path().to_path_buf(),
            temp.path().join("search_plane"),
            SearchManifestKeyspace::new("xiuxian:test:symbol-index:empty"),
            SearchMaintenancePolicy::default(),
        ),
    ));
    let index_cache = Arc::new(RwLock::new(Some(Arc::new(UnifiedSymbolIndex::new()))));

    coordinator.sync_projects(Vec::new(), Arc::clone(&index_cache));

    assert!(
        index_cache
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none()
    );
    assert_eq!(coordinator.status().phase, SymbolIndexPhase::Idle);
}

#[tokio::test]
async fn ensure_started_marks_non_idle_for_configured_projects() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    std::fs::create_dir_all(temp.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(
        temp.path().join("src").join("lib.rs"),
        "pub struct BackgroundSymbolIndex;\n",
    )
    .unwrap_or_else(|error| panic!("write source: {error}"));
    let coordinator = Arc::new(SymbolIndexCoordinator::new(
        temp.path().to_path_buf(),
        temp.path().to_path_buf(),
        SearchPlaneService::with_paths(
            temp.path().to_path_buf(),
            temp.path().join("search_plane"),
            SearchManifestKeyspace::new("xiuxian:test:symbol-index:start"),
            SearchMaintenancePolicy::default(),
        ),
    ));
    let index_cache = Arc::new(RwLock::new(None));

    coordinator.ensure_started(
        vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["src".to_string()],
        }],
        Arc::clone(&index_cache),
    );

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    assert!(matches!(
        coordinator.status().phase,
        SymbolIndexPhase::Indexing | SymbolIndexPhase::Ready
    ));
}

#[tokio::test]
async fn stop_resets_status_to_idle_after_starting_build() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    std::fs::create_dir_all(temp.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::write(
        temp.path().join("src").join("lib.rs"),
        "pub struct BackgroundSymbolIndex;\n",
    )
    .unwrap_or_else(|error| panic!("write source: {error}"));
    let coordinator = Arc::new(SymbolIndexCoordinator::new(
        temp.path().to_path_buf(),
        temp.path().to_path_buf(),
        SearchPlaneService::with_paths(
            temp.path().to_path_buf(),
            temp.path().join("search_plane"),
            SearchManifestKeyspace::new("xiuxian:test:symbol-index:stop"),
            SearchMaintenancePolicy::default(),
        ),
    ));
    let index_cache = Arc::new(RwLock::new(None));

    coordinator.ensure_started(
        vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["src".to_string()],
        }],
        Arc::clone(&index_cache),
    );
    coordinator.stop();
    tokio::task::yield_now().await;

    assert_eq!(coordinator.status().phase, SymbolIndexPhase::Idle);
}

#[tokio::test]
async fn ensure_started_restores_symbol_index_from_local_symbol_artifact() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let project_root = temp.path().join("workspace");
    let storage_root = temp.path().join("search_plane");
    std::fs::create_dir_all(&project_root)
        .unwrap_or_else(|error| panic!("create workspace: {error}"));

    let writer = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root.clone(),
        SearchManifestKeyspace::new("xiuxian:test:symbol-index:restore-writer"),
        SearchMaintenancePolicy::default(),
    );
    writer
        .publish_local_symbol_hits(
            "fp-local-symbol-restore",
            &[AstSearchHit {
                name: "WarmRestoreSymbol".to_string(),
                signature: "fn WarmRestoreSymbol()".to_string(),
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
                    line: Some(7),
                    line_end: Some(7),
                    column: Some(1),
                },
                line_start: 7,
                line_end: 7,
                score: 0.0,
            }],
        )
        .await
        .unwrap_or_else(|error| panic!("publish local symbol hits: {error}"));

    let reader = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        SearchManifestKeyspace::new("xiuxian:test:symbol-index:restore-reader"),
        SearchMaintenancePolicy::default(),
    );
    let coordinator = Arc::new(SymbolIndexCoordinator::new(
        project_root.clone(),
        project_root,
        reader,
    ));
    let index_cache = Arc::new(RwLock::new(None));
    let projects = vec![UiProjectConfig {
        name: "kernel".to_string(),
        root: ".".to_string(),
        dirs: vec!["src".to_string()],
    }];

    coordinator.ensure_started(projects, Arc::clone(&index_cache));
    wait_for_symbol_index_ready(&coordinator).await;

    let index = index_cache
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .unwrap_or_else(|| panic!("restored symbol index should be present"));
    let results = index.search_unified("WarmRestoreSymbol", 10);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "WarmRestoreSymbol");
    assert_eq!(results[0].kind, "function");
    assert_eq!(results[0].location, "src/lib.rs:7");
}

async fn wait_for_symbol_index_ready(coordinator: &Arc<SymbolIndexCoordinator>) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        if matches!(coordinator.status().phase, SymbolIndexPhase::Ready) {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "symbol index did not become ready before timeout"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}
