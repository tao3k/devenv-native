//! Internal query-core building blocks for Phase-1 Wendao RFC execution.

/// Execution context and backend traits for query-core operators.
#[path = "context.rs"]
pub mod context;
/// Adapter-backed operator execution entry points.
#[path = "execute/mod.rs"]
pub mod execute;
/// Query-core-native graph projections.
#[path = "graph.rs"]
pub mod graph;
/// Typed operator request models.
#[path = "operators/mod.rs"]
pub mod operators;
/// Thin internal facade helpers for common Wendao query-core calls.
#[path = "service.rs"]
pub mod service;
/// Explain and telemetry contracts for query-core execution.
#[path = "telemetry.rs"]
pub mod telemetry;
/// Shared relation, backend, and error types.
#[path = "types.rs"]
pub mod types;

pub use context::{GraphBackend, RetrievalBackend, WendaoExecutionContext, WendaoResourceBudget};
pub use execute::{
    LinkGraphNeighborsBackend, SearchPlaneRetrievalBackend, execute_column_mask,
    execute_graph_neighbors, execute_payload_fetch, execute_vector_search,
};
pub use graph::{
    WendaoGraphLink, WendaoGraphNode, WendaoGraphProjection, graph_projection_from_relation,
};
pub use operators::{
    ColumnMaskOp, ColumnMaskPredicate, GraphDirection, GraphNeighborsOp, PayloadFetchOp,
    RetrievalCorpus, VectorSearchOp,
};
pub use service::{
    RepoCodeQueryPolicy, RepoCodeQueryRelation, RepoCodeQueryRequest,
    RepoEntityTypedResultsContract, RepoRetrievalQuery, query_graph_neighbors_projection,
    query_graph_neighbors_relation, query_repo_code_relation, query_repo_content_relation,
    query_repo_entity_example_results_if_published, query_repo_entity_import_results_if_published,
    query_repo_entity_module_results_if_published, query_repo_entity_relation,
    query_repo_entity_results_if_published, query_repo_entity_symbol_results_if_published,
    repo_entity_example_results_contract, repo_entity_module_results_contract,
    repo_entity_symbol_results_contract,
};
pub use telemetry::{
    InMemoryWendaoExplainSink, NoopWendaoExplainSink, WendaoExplainEvent, WendaoExplainSink,
    explain_events_summary,
};
pub use types::{WendaoBackendKind, WendaoOperatorKind, WendaoQueryCoreError, WendaoRelation};

#[cfg(test)]
#[path = "../../tests/unit/query_core/mod.rs"]
mod tests;
