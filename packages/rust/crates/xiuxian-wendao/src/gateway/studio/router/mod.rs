//! Studio API router for Qianji frontend.
//!
//! Provides HTTP endpoints for VFS operations, graph queries, and UI configuration.

/// Code-AST response builders and repository/path resolution helpers.
pub mod code_ast;
pub mod config;
mod error;
pub mod handlers;
mod repository;
pub(crate) mod retrieval_arrow;
mod routes;
pub mod sanitization;
mod state;

pub use code_ast::build_code_ast_analysis_response;
pub use config::{
    load_ui_config_from_wendao_toml, resolve_studio_config_root, studio_effective_wendao_toml_path,
    studio_wendao_overlay_toml_path, studio_wendao_toml_path,
};
pub use error::{StudioApiError, map_repo_intelligence_error};
pub use handlers::{
    doc_coverage, docs_family_cluster, docs_family_context, docs_family_search, docs_navigation,
    docs_navigation_search, docs_page, docs_page_index_tree, docs_planner_item, docs_planner_queue,
    docs_planner_rank, docs_planner_search, docs_planner_workset, docs_projected_gap_report,
    docs_retrieval, docs_retrieval_context, docs_retrieval_hit, docs_search, example_search,
    get_plugin_artifact, module_search, overview, projected_page, projected_page_family_cluster,
    projected_page_family_context, projected_page_family_search, projected_page_index_node,
    projected_page_index_tree, projected_page_index_tree_search, projected_page_index_trees,
    projected_page_navigation, projected_page_navigation_search, projected_page_search,
    projected_pages, projected_retrieval, projected_retrieval_context, projected_retrieval_hit,
    refine_entity_doc, symbol_search, sync, topology_3d, vfs_cat, vfs_entry, vfs_root_entries,
    vfs_scan,
};
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
#[path = "../../../../tests/unit/gateway/studio/router/mod.rs"]
mod tests;
