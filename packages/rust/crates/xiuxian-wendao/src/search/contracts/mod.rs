//! Search contract types shared by Wendao domain search and Studio adapters.
//!
//! Studio owns HTTP route inventory and TypeScript schema export. Wendao keeps
//! the search payload records that are still used by domain search runtime.

mod analysis;
mod attachments;
mod config;
mod definitions;
#[cfg(feature = "search-runtime")]
mod helpers;
mod hits;
mod navigation;
mod retrieval;
#[cfg(feature = "search-runtime")]
#[path = "search_index/mod.rs"]
mod search_index;
mod symbols;

pub use analysis::{AnalysisNode, AnalysisNodeKind};
pub use attachments::AttachmentSearchHit;
#[cfg(feature = "search-runtime")]
pub(crate) use config::materialize_project_configs;
pub use config::{ProjectConfigView, SearchProjectConfig};
pub use definitions::{AstSearchHit, DefinitionSearchHit, ObservationHint, ReferenceSearchHit};
#[cfg(feature = "search-runtime")]
pub(crate) use helpers::{
    SearchProjectMetadata, ast_search_lang, build_code_ast_hits_from_content,
    build_markdown_ast_hits_from_sections, compile_markdown_nodes, configured_project_scopes,
    index_path_for_entry, infer_crate_name, is_markdown_path, markdown_scope_name,
    project_metadata_for_path, resolve_project_root_path, score_reference_hit, should_skip_entry,
};
pub use hits::{IntentSearchHit, KnowledgeSearchHit, SearchBacklinkItem, SearchHit};
pub use navigation::StudioNavigationTarget;
pub use retrieval::{RetrievalChunk, RetrievalChunkSurface};
#[cfg(feature = "search-runtime")]
pub use search_index::{
    QUERY_TELEMETRY_DIAGNOSTICS_TABLE, REPO_READ_PRESSURE_DIAGNOSTICS_TABLE,
    STATUS_DIAGNOSTICS_TABLE, STATUS_REASON_DIAGNOSTICS_TABLE, SearchCorpusIndexStatus,
    SearchIndexMaintenanceStatus, SearchIndexPhase, SearchIndexStatusResponse,
    diagnostics_schema_ref, query_telemetry_contract, repo_read_pressure_contract,
    status_reason_contract, status_snapshot_contract,
};
pub use symbols::AutocompleteSuggestion;
