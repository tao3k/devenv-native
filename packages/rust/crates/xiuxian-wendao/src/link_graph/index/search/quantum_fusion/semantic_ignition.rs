//! `link_graph::index::search::quantum_fusion::semantic_ignition` owns Wendao search quantum fusion semantic ignition behavior.

use super::orchestrate::QuantumContextBuildError;
use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::models::{
    LinkGraphRetrievalPlanRecord, LinkGraphSemanticSearchPolicy, QuantumAnchorHit, QuantumContext,
    QuantumFusionOptions, QuantumSemanticSearchRequest,
};
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

/// Boxed future returned by semantic-ignition backends.
pub type QuantumSemanticIgnitionFuture<'a, E> =
    Pin<Box<dyn Future<Output = Result<Vec<QuantumAnchorHit>, E>> + Send + 'a>>;

/// Error returned when semantic ignition or subsequent quantum-context
/// orchestration fails.
#[derive(Debug, Error)]
/// Generic bound boundary: this public error type keeps derived trait bounds at the boundary.
pub enum QuantumSemanticIgnitionError<E>
where
    E: std::error::Error + 'static,
{
    /// Semantic backend failed to return anchor hits.
    #[error("semantic ignition backend `{backend_name}` failed")]
    Backend {
        /// Stable backend identifier returned by the ignition provider.
        backend_name: String,
        /// Original backend-specific error.
        #[source]
        source: E,
    },
    /// Wendao failed to turn anchor hits into scored quantum contexts.
    #[error("quantum-context orchestration failed after backend `{backend_name}` returned anchors")]
    Orchestration {
        /// Stable backend identifier returned by the ignition provider.
        backend_name: String,
        /// Typed orchestration failure emitted by Wendao.
        #[source]
        source: QuantumContextBuildError,
    },
}

/// Backend-agnostic semantic ignition provider for hybrid retrieval.
pub trait QuantumSemanticIgnition: Send + Sync {
    /// Backend-specific error type emitted during anchor search.
    type Error;

    /// Stable backend identifier for telemetry and retrieval-plan integration.
    fn backend_name(&self) -> &str;

    /// Search for semantic anchors that can seed quantum-fusion retrieval.
    fn search_anchors<'a>(
        &'a self,
        request: QuantumSemanticSearchRequest<'a>,
    ) -> QuantumSemanticIgnitionFuture<'a, Self::Error>;
}

impl LinkGraphIndex {
    /// Build quantum-fusion contexts from a planned retrieval budget plus semantic policy.
    ///
    /// # Errors
    ///
    /// Returns [`QuantumSemanticIgnitionError::Backend`] when the semantic
    /// backend fails, or [`QuantumSemanticIgnitionError::Orchestration`] when
    /// Wendao cannot convert returned anchor hits into scored quantum contexts.
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub async fn quantum_contexts_from_retrieval_plan<I>(
        &self,
        ignition: &I,
        query_text: Option<&str>,
        query_vector: &[f32],
        retrieval_plan: Option<&LinkGraphRetrievalPlanRecord>,
        semantic_policy: Option<LinkGraphSemanticSearchPolicy>,
        options: &QuantumFusionOptions,
    ) -> Result<Vec<QuantumContext>, QuantumSemanticIgnitionError<I::Error>>
    where
        I: QuantumSemanticIgnition + ?Sized,
        I::Error: std::error::Error + 'static,
    {
        let request = QuantumSemanticSearchRequest::from_retrieval_budget(
            query_text,
            query_vector,
            retrieval_plan.map(|plan| &plan.budget),
            semantic_policy,
        );
        self.quantum_contexts_from_semantic_ignition(ignition, request, options)
            .await
    }

    /// Build quantum-fusion contexts by delegating anchor search to a semantic-ignition backend.
    ///
    /// # Errors
    ///
    /// Returns [`QuantumSemanticIgnitionError::Backend`] when the semantic
    /// backend fails, or [`QuantumSemanticIgnitionError::Orchestration`] when
    /// Wendao cannot convert returned anchor hits into scored quantum contexts.
    pub async fn quantum_contexts_from_semantic_ignition<I>(
        &self,
        ignition: &I,
        request: QuantumSemanticSearchRequest<'_>,
        options: &QuantumFusionOptions,
    ) -> Result<Vec<QuantumContext>, QuantumSemanticIgnitionError<I::Error>>
    where
        I: QuantumSemanticIgnition + ?Sized,
        I::Error: std::error::Error + 'static,
    {
        let request = request.normalized();
        if request.is_empty() {
            return Ok(Vec::new());
        }

        let backend_name = ignition.backend_name().to_string();
        let mut anchors = ignition.search_anchors(request).await.map_err(|source| {
            QuantumSemanticIgnitionError::Backend {
                backend_name: backend_name.clone(),
                source,
            }
        })?;
        anchors.retain(|anchor| request.allows_vector_score(anchor.vector_score));
        if anchors.is_empty() {
            return Ok(Vec::new());
        }

        self.quantum_contexts_from_anchors(&anchors, options)
            .map_err(|source| QuantumSemanticIgnitionError::Orchestration {
                backend_name,
                source,
            })
    }
}
