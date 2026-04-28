//! Studio API router for Qianji frontend.
//!
//! Provides HTTP endpoints for VFS operations, graph queries, and UI configuration.

/// Code-AST response builders and repository/path resolution helpers.
#[path = "router/code_ast/mod.rs"]
mod code_ast;
#[path = "router/config/mod.rs"]
mod config;
#[path = "router/error.rs"]
mod error;
#[path = "router/handlers.rs"]
pub(crate) mod handlers;
#[path = "router/repository.rs"]
mod repository;
#[path = "router/retrieval_arrow.rs"]
pub(crate) mod retrieval_arrow;
#[path = "router/routes.rs"]
mod routes;
#[path = "router/sanitization.rs"]
mod sanitization;
#[path = "router/state/mod.rs"]
mod state;

pub use code_ast::build_code_ast_analysis_response;
pub(crate) use code_ast::build_generic_code_ast_analysis_response;
pub use code_ast::resolve_code_ast_repository_and_path;
pub use config::{
    load_ui_config_from_wendao_toml, load_ui_config_from_wendao_toml_path,
    resolve_studio_config_root, studio_effective_wendao_toml_path, studio_wendao_overlay_toml_path,
    studio_wendao_toml_path,
};
pub use error::{StudioApiError, map_repo_intelligence_error};
pub use repository::{configured_repositories, configured_repository};
pub(crate) use repository::{registered_repository_search_seeds, resolve_registered_repository_id};
pub use routes::{studio_router, studio_routes};
pub use sanitization::{
    sanitize_path_like, sanitize_path_list, sanitize_projects, sanitize_repo_projects,
};
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
