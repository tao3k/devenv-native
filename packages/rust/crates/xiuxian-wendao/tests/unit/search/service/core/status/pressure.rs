use std::path::PathBuf;
use std::sync::Arc;

use crate::search::service::core::types::SearchPlaneService;
use crate::search::{SearchMaintenancePolicy, SearchManifestKeyspace};

#[test]
fn status_snapshot_surfaces_repo_read_pressure() {
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        PathBuf::from("/tmp/search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:search-plane:repo-read-pressure"),
        SearchMaintenancePolicy::default(),
    );
    let permit_budget = service.repo_search_read_concurrency_limit;
    let held_permits = permit_budget.min(2);
    let held = Arc::clone(&service.repo_search_read_permits)
        .try_acquire_many_owned(u32::try_from(held_permits).unwrap_or(u32::MAX))
        .unwrap_or_else(|error| panic!("acquire repo read permits: {error}"));
    service.record_repo_search_dispatch(177, 96, permit_budget);

    let snapshot = service.status();
    let repo_read_pressure = snapshot
        .repo_read_pressure
        .as_ref()
        .unwrap_or_else(|| panic!("repo read pressure should be present"));

    assert_eq!(
        repo_read_pressure.budget,
        u32::try_from(permit_budget).unwrap_or(u32::MAX)
    );
    assert_eq!(
        repo_read_pressure.in_flight,
        u32::try_from(held_permits).unwrap_or(u32::MAX)
    );
    assert_eq!(repo_read_pressure.requested_repo_count, Some(177));
    assert_eq!(repo_read_pressure.searchable_repo_count, Some(96));
    assert_eq!(
        repo_read_pressure.parallelism,
        Some(u32::try_from(permit_budget).unwrap_or(u32::MAX))
    );
    assert!(repo_read_pressure.fanout_capped);
    assert!(repo_read_pressure.captured_at.is_some());

    drop(held);
}
