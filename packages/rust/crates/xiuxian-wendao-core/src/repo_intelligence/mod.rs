//! Repository-intelligence plugin contracts, records, registry, and errors.

#[cfg(feature = "arrow-transport")]
mod arrow_transport;
mod builtin;
mod config;
mod errors;
mod plugin;
mod projection;
mod records;
mod registry;

#[cfg(feature = "arrow-transport")]
pub use arrow_transport::{
    JULIA_ARROW_ANALYZER_SCORE_COLUMN, JULIA_ARROW_DOC_ID_COLUMN, JULIA_ARROW_EMBEDDING_COLUMN,
    JULIA_ARROW_FINAL_SCORE_COLUMN, JULIA_ARROW_QUERY_EMBEDDING_COLUMN,
    JULIA_ARROW_TRACE_ID_COLUMN, JULIA_ARROW_VECTOR_SCORE_COLUMN, julia_arrow_request_schema,
    julia_arrow_response_schema,
};
pub use builtin::{BuiltinPluginId, BuiltinPluginRegistrar, builtin_plugin_registrars};
pub use config::{
    RegisteredRepository, RepoIntelligenceConfig, RepositoryPluginConfig, RepositoryRef,
    RepositoryRefreshPolicy,
};
pub use errors::{ProjectedGapId, ProjectedNodeId, ProjectedPageId, RepoIntelligenceError};
pub use plugin::{
    AnalysisContext, PluginAnalysisOutput, PluginLinkContext, RepoIntelligencePlugin,
    RepoSourceFile, RepositoryAnalysisOutput,
};
pub use projection::ProjectionPageKind;
pub use records::{
    DiagnosticRecord, DocRecord, DocRecordId, DocTargetKind, DocTargetRecord, ExampleRecord,
    ExampleRecordId, ImportKind, ImportRecord, ModuleRecord, ModuleRecordId, RelationKind,
    RelationRecord, RepoAuditStatus, RepoIntelligencePluginId, RepoRecordId, RepoRecordPath,
    RepoSymbolKind, RepoVerificationState, RepositoryRecord, SymbolRecord, SymbolRecordId,
};
pub use registry::PluginRegistry;
