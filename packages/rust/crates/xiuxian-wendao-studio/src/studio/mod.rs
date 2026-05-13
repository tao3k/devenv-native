//! Studio API gateway for Qianji frontend.
//!
//! Provides HTTP endpoints for VFS operations, graph queries, and UI configuration.

/// Studio public API surface.
/// Studio public API surface.
#[path = "types/mod.rs"]
pub mod types;

/// Feature-gated PDF page rendering and OCR shard manifest helpers.
#[cfg(feature = "document-extract-pdf-source-range")]
#[doc(hidden)]
pub use xiuxian_wendao_attachments::pdf::render as document_extract_pdf_render;

/// Feature-gated PDF OCR shard Arrow contract helpers.
#[cfg(feature = "document-extract-pdf-source-range")]
#[doc(hidden)]
pub use xiuxian_wendao_attachments::pdf::ocr as document_extract_pdf_ocr;

/// Feature-gated PDF OCR shard Flight client proof helpers.
#[cfg(feature = "document-extract-pdf-source-range")]
#[doc(hidden)]
#[path = "document_extract_pdf_ocr_client.rs"]
pub mod document_extract_pdf_ocr_client;
#[cfg(feature = "document-extract-pdf-source-range")]
pub(crate) use document_extract_pdf_ocr_client::PdfOcrShardSchedulerTrace;

#[cfg(feature = "zhenfa-router")]
#[path = "analysis/mod.rs"]
mod analysis;
#[cfg(feature = "zhenfa-router")]
#[path = "arrow_types.rs"]
pub(crate) mod arrow_types;
#[cfg(feature = "zhenfa-router")]
#[path = "pathing.rs"]
mod pathing;
/// Performance fixtures and helpers for Studio gateway benchmarks.
#[cfg(all(feature = "zhenfa-router", feature = "performance"))]
#[path = "perf_support/mod.rs"]
pub mod perf_support;
#[cfg(feature = "zhenfa-router")]
#[path = "router/mod.rs"]
pub(crate) mod router;
#[cfg(feature = "zhenfa-router")]
#[path = "search/mod.rs"]
pub(crate) mod search;
/// Gateway startup dependency health probes and reporting.
#[cfg(feature = "zhenfa-router")]
#[path = "startup_health/mod.rs"]
pub(crate) mod startup_health;
#[cfg(feature = "zhenfa-router")]
#[path = "symbol_index/mod.rs"]
pub(crate) mod symbol_index;
#[cfg(feature = "zhenfa-router")]
#[path = "vfs/mod.rs"]
mod vfs;

#[cfg(feature = "zhenfa-router")]
pub use router::{
    GatewayState, StudioApiError, StudioBootstrapBackgroundIndexingTelemetry,
    StudioSearchColdStartCorpusTelemetry, StudioSearchColdStartEvent,
    StudioSearchColdStartTelemetry, StudioState, configured_repositories,
    load_ui_config_from_wendao_toml, map_repo_intelligence_error, resolve_studio_config_root,
    sanitize_path_like, sanitize_path_list, studio_effective_wendao_toml_path, studio_router,
    studio_routes, studio_wendao_overlay_toml_path, studio_wendao_toml_path,
};
#[cfg(feature = "zhenfa-router")]
pub(crate) use router::{
    configured_repository, registered_repository_search_seeds, resolve_registered_repository_id,
};
#[cfg(feature = "zhenfa-router")]
pub use search::build_ast_index;
#[cfg(feature = "flight-server-bin-support")]
pub(crate) use search::handlers::build_studio_flight_service_for_roots_with_weights;
#[cfg(feature = "cli-bin-support")]
pub(crate) use search::handlers::build_studio_flight_service_with_weights;
#[cfg(feature = "zhenfa-router")]
pub use search::handlers::{
    StudioFlightRoots, StudioRepoSearchFlightRouteProvider, bootstrap_sample_repo_search_content,
    build_repo_search_flight_service, build_repo_search_flight_service_with_weights,
    build_studio_flight_service, build_studio_flight_service_for_roots,
};
#[cfg(feature = "zhenfa-router")]
pub use startup_health::{
    GatewayStartupDependencyCheck, GatewayStartupDependencyStatus, GatewayStartupHealthReport,
    describe_gateway_startup_health, probe_gateway_startup_health,
};

/// SearchStrategyFlow proof surfaces owned by Studio.
#[cfg(all(feature = "zhenfa-router", feature = "julia"))]
pub mod search_strategy_flow;

#[cfg(test)]
#[path = "../../tests/unit/gateway/studio/support.rs"]
pub(crate) mod test_support;

#[cfg(all(test, feature = "zhenfa-router"))]
#[path = "../../tests/unit/studio_vfs_performance.rs"]
mod studio_vfs_performance_tests;

#[cfg(all(test, feature = "zhenfa-router"))]
#[path = "../../tests/unit/studio_repo_sync_api/mod.rs"]
pub(crate) mod studio_repo_sync_api_tests;
