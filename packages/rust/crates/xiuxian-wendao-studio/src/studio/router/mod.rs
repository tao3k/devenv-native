//! Studio API router for Qianji frontend.
//!
//! Provides HTTP endpoints for VFS operations, graph queries, and UI configuration.

#[path = "config/mod.rs"]
mod config;
#[path = "error.rs"]
mod error;
#[path = "handlers/mod.rs"]
pub(crate) mod handlers;
#[path = "repository.rs"]
mod repository;
#[path = "retrieval_arrow.rs"]
pub(crate) mod retrieval_arrow;
#[path = "routes.rs"]
mod routes;
#[path = "sanitization.rs"]
mod sanitization;
#[path = "state/mod.rs"]
mod state;

#[cfg(feature = "cli-bin-support")]
pub(crate) use config::load_episteme_registry_from_wendao_toml_path;
#[cfg(any(test, feature = "julia"))]
pub(crate) use config::load_wendaograph_ontology_read_model_quality_endpoint_from_wendao_toml;
pub(crate) use config::{
    load_document_extract_endpoint_from_wendao_toml, load_episteme_registry_from_wendao_toml,
    load_model_routing_config_from_wendao_toml,
};
pub use config::{
    load_ui_config_from_wendao_toml, load_ui_config_from_wendao_toml_path,
    resolve_studio_config_root, studio_effective_wendao_toml_path, studio_wendao_overlay_toml_path,
    studio_wendao_toml_path,
};
pub use error::{StudioApiError, map_repo_intelligence_error};
pub use repository::configured_repositories;
pub(crate) use repository::configured_repository;
pub(crate) use repository::{registered_repository_search_seeds, resolve_registered_repository_id};
pub use routes::{studio_router, studio_routes};
pub use sanitization::{
    sanitize_path_like, sanitize_path_list, sanitize_projects, sanitize_repo_projects,
};
#[cfg(any(test, feature = "performance"))]
pub(crate) use state::LocalCorpusScanCoalescingState;
#[cfg(any(test, feature = "performance"))]
pub(crate) use state::StudioSearchColdStartTelemetryState;
pub use state::{
    GatewayState, StudioBootstrapBackgroundIndexingTelemetry, StudioSearchColdStartCorpusTelemetry,
    StudioSearchColdStartEvent, StudioSearchColdStartTelemetry, StudioState,
};
#[cfg(test)]
pub(crate) use state::{GraphIndexCacheEntry, GraphSourceSignature};

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/router/mod.rs"]
mod tests;
