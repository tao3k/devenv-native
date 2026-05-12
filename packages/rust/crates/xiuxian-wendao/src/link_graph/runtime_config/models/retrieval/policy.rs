//! `link_graph::runtime_config::models::retrieval::policy` owns Wendao models retrieval policy behavior.

use crate::link_graph::models::LinkGraphRetrievalMode;
use crate::link_graph::runtime_config::constants::DEFAULT_LINK_GRAPH_RETRIEVAL_MODE;
use xiuxian_wendao_core::capabilities::PluginCapabilityBinding;
use xiuxian_wendao_runtime::config::LinkGraphRetrievalBaseRuntimeConfig;
use xiuxian_wendao_runtime::transport::RerankScoreWeights;

use super::semantic_ignition::LinkGraphSemanticIgnitionRuntimeConfig;
/// `LinkGraphRetrievalPolicyRuntimeConfig` public type boundary for Wendao.

pub struct LinkGraphRetrievalPolicyRuntimeConfig {
    pub mode: LinkGraphRetrievalMode,
    pub candidate_multiplier: usize,
    pub max_sources: usize,
    pub hybrid_min_hits: usize,
    pub hybrid_min_top_score: f64,
    pub graph_rows_per_source: usize,
    pub semantic_ignition: LinkGraphSemanticIgnitionRuntimeConfig,
    pub rerank_binding: Option<PluginCapabilityBinding>,
    pub rerank_schema_version: Option<String>,
    pub rerank_score_weights: Option<RerankScoreWeights>,
}

impl Default for LinkGraphRetrievalPolicyRuntimeConfig {
    fn default() -> Self {
        Self::from(LinkGraphRetrievalBaseRuntimeConfig::default())
    }
}

impl LinkGraphRetrievalPolicyRuntimeConfig {
    /// Resolve the current rerank provider binding through the generic plugin-runtime model.
    #[must_use]
    pub fn rerank_binding(&self) -> Option<PluginCapabilityBinding> {
        self.rerank_binding.clone()
    }

    /// Resolve the current rerank-side schema version through the generic runtime model.
    #[must_use]
    pub fn rerank_schema_version(&self) -> Option<String> {
        self.rerank_schema_version.clone()
    }

    /// Resolve the current rerank score weights through the generic runtime model.
    #[must_use]
    pub fn rerank_score_weights(&self) -> Option<RerankScoreWeights> {
        self.rerank_score_weights
    }
}

impl From<LinkGraphRetrievalBaseRuntimeConfig> for LinkGraphRetrievalPolicyRuntimeConfig {
    fn from(base: LinkGraphRetrievalBaseRuntimeConfig) -> Self {
        Self {
            mode: LinkGraphRetrievalMode::from_alias(DEFAULT_LINK_GRAPH_RETRIEVAL_MODE)
                .unwrap_or_default(),
            candidate_multiplier: base.candidate_multiplier,
            max_sources: base.max_sources,
            hybrid_min_hits: base.hybrid_min_hits,
            hybrid_min_top_score: base.hybrid_min_top_score,
            graph_rows_per_source: base.graph_rows_per_source,
            semantic_ignition: base.semantic_ignition,
            rerank_binding: None,
            rerank_schema_version: None,
            rerank_score_weights: None,
        }
    }
}
