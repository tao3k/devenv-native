//! `query_core::context` owns Wendao query core context behavior.

use std::sync::Arc;

use async_trait::async_trait;

use super::operators::{GraphNeighborsOp, PayloadFetchOp, VectorSearchOp};
use super::telemetry::{NoopWendaoExplainSink, WendaoExplainSink};
use super::types::{WendaoQueryCoreError, WendaoRelation};

/// Retrieval-side execution contract owned by Wendao query core.
#[async_trait]
pub trait RetrievalBackend: Send + Sync {
    /// Execute a retrieval-first candidate search.
    async fn vector_search(
        &self,
        op: &VectorSearchOp,
    ) -> Result<WendaoRelation, WendaoQueryCoreError>;

    /// Hydrate payload columns for a previously materialized relation.
    async fn payload_fetch(
        &self,
        relation: &WendaoRelation,
        op: &PayloadFetchOp,
    ) -> Result<WendaoRelation, WendaoQueryCoreError>;
}

/// Graph-side execution contract owned by Wendao query core.
#[async_trait]
pub trait GraphBackend: Send + Sync {
    /// Resolve graph neighbors for a seed node.
    async fn graph_neighbors(
        &self,
        op: &GraphNeighborsOp,
    ) -> Result<WendaoRelation, WendaoQueryCoreError>;
}

/// Resource hints carried with a single query-core execution.
#[derive(Debug, Clone, Default)]
pub struct WendaoResourceBudget {
    /// Preferred partition count for backend execution.
    pub target_partitions: Option<usize>,
    /// Optional memory budget in bytes.
    pub memory_budget_bytes: Option<usize>,
    /// Optional graph fanout cap.
    pub graph_fanout_limit: Option<usize>,
}

/// Execution context shared across RFC Phase-1 operators.
#[derive(Clone)]
pub struct WendaoExecutionContext {
    /// Retrieval backend adapter, when available.
    pub retrieval_backend: Option<Arc<dyn RetrievalBackend>>,
    /// Graph backend adapter, when available.
    pub graph_backend: Option<Arc<dyn GraphBackend>>,
    /// Explain sink receiving execution events.
    pub explain_sink: Arc<dyn WendaoExplainSink>,
    /// Resource hints for this execution.
    pub resources: WendaoResourceBudget,
}

impl Default for WendaoExecutionContext {
    fn default() -> Self {
        Self {
            retrieval_backend: None,
            graph_backend: None,
            explain_sink: Arc::new(NoopWendaoExplainSink),
            resources: WendaoResourceBudget::default(),
        }
    }
}

impl WendaoExecutionContext {
    /// Attach a retrieval backend adapter.
    #[must_use]
    pub fn with_retrieval_backend(mut self, backend: Arc<dyn RetrievalBackend>) -> Self {
        self.retrieval_backend = Some(backend);
        self
    }

    /// Attach a graph backend adapter.
    #[must_use]
    pub fn with_graph_backend(mut self, backend: Arc<dyn GraphBackend>) -> Self {
        self.graph_backend = Some(backend);
        self
    }

    /// Attach an explain sink.
    #[must_use]
    pub fn with_explain_sink(mut self, sink: Arc<dyn WendaoExplainSink>) -> Self {
        self.explain_sink = sink;
        self
    }
}
