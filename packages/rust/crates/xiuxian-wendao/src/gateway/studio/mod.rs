//! Studio API gateway for Qianji frontend.
//!
//! Provides HTTP endpoints for VFS operations, graph queries, and UI configuration.

pub mod types;

#[cfg(feature = "zhenfa-router")]
mod analysis;
#[cfg(feature = "zhenfa-router")]
mod pathing;
/// Performance fixtures and helpers for Studio gateway benchmarks.
#[cfg(all(feature = "zhenfa-router", feature = "performance"))]
pub mod perf_support;
#[cfg(feature = "zhenfa-router")]
pub(crate) mod router;
#[cfg(feature = "zhenfa-router")]
pub(crate) mod search;
/// Gateway startup dependency health probes and reporting.
#[cfg(feature = "zhenfa-router")]
pub(crate) mod startup_health;
#[cfg(feature = "zhenfa-router")]
pub(crate) mod symbol_index;
#[cfg(feature = "zhenfa-router")]
mod vfs;

#[cfg(feature = "zhenfa-router")]
pub(crate) use analysis::compile_markdown_nodes;
#[cfg(feature = "zhenfa-router")]
pub use router::{
    GatewayState, StudioApiError, StudioBootstrapBackgroundIndexingTelemetry,
    StudioSearchColdStartCorpusTelemetry, StudioSearchColdStartEvent,
    StudioSearchColdStartTelemetry, StudioState, configured_repositories, configured_repository,
    load_ui_config_from_wendao_toml, map_repo_intelligence_error, resolve_studio_config_root,
    sanitize_path_like, sanitize_path_list, studio_effective_wendao_toml_path, studio_router,
    studio_routes, studio_wendao_overlay_toml_path, studio_wendao_toml_path,
};
#[cfg(feature = "zhenfa-router")]
pub(crate) use router::{registered_repository_search_seeds, resolve_registered_repository_id};
#[cfg(all(feature = "zhenfa-router", test))]
pub(crate) use search::build_ast_index;
#[cfg(feature = "zhenfa-router")]
pub use search::handlers::{
    StudioRepoSearchFlightRouteProvider, bootstrap_sample_repo_search_content,
    build_repo_search_flight_service, build_repo_search_flight_service_with_weights,
    build_studio_flight_service, build_studio_flight_service_for_roots,
    build_studio_flight_service_for_roots_with_weights, build_studio_flight_service_with_weights,
};
#[cfg(feature = "zhenfa-router")]
pub use startup_health::{
    GatewayStartupDependencyCheck, GatewayStartupDependencyStatus, GatewayStartupHealthReport,
    describe_gateway_startup_health, probe_gateway_startup_health,
};

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/support.rs"]
pub(crate) mod test_support;

#[cfg(all(test, feature = "zhenfa-router"))]
#[path = "../../../tests/unit/studio_vfs_performance.rs"]
mod studio_vfs_performance_tests;

#[cfg(all(test, feature = "zhenfa-router"))]
#[path = "../../../tests/unit/studio_repo_sync_api/mod.rs"]
mod studio_repo_sync_api_tests;
