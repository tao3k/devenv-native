use crate::link_graph::agentic::{
    LinkGraphAgenticExecutionConfig, LinkGraphAgenticExecutionResult,
    LinkGraphAgenticExpansionConfig, LinkGraphAgenticExpansionPlan,
};
use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::runtime_config::resolve_link_graph_agentic_runtime;

impl LinkGraphIndex {
    /// Resolve bounded agentic expansion config from runtime settings.
    ///
    /// The config is read from merged `wendao.yaml` settings first, then
    /// environment fallback, via `resolve_link_graph_agentic_runtime()`.
    #[must_use]
    pub fn resolve_agentic_expansion_config() -> LinkGraphAgenticExpansionConfig {
        let runtime = resolve_link_graph_agentic_runtime();
        LinkGraphAgenticExpansionConfig {
            max_workers: runtime.expansion_max_workers,
            max_candidates: runtime.expansion_max_candidates,
            max_pairs_per_worker: runtime.expansion_max_pairs_per_worker,
            time_budget_ms: runtime.expansion_time_budget_ms,
        }
        .normalized()
    }

    /// Resolve bounded agentic execution config from runtime settings.
    ///
    /// The config is read from merged `wendao.yaml` settings first, then
    /// environment fallback, via `resolve_link_graph_agentic_runtime()`.
    #[must_use]
    pub fn resolve_agentic_execution_config() -> LinkGraphAgenticExecutionConfig {
        let runtime = resolve_link_graph_agentic_runtime();
        LinkGraphAgenticExecutionConfig {
            expansion: Self::resolve_agentic_expansion_config(),
            worker_time_budget_ms: runtime.execution_worker_time_budget_ms,
            persist_suggestions: runtime.execution_persist_suggestions_default,
            persist_retry_attempts: runtime.execution_persist_retry_attempts,
            idempotency_scan_limit: runtime.execution_idempotency_scan_limit,
            relation: runtime.execution_relation,
            agent_id: runtime.execution_agent_id,
            evidence_prefix: runtime.execution_evidence_prefix,
            created_at_unix: None,
        }
        .normalized()
    }

    /// Build a bounded sub-agent expansion plan using runtime default config.
    #[must_use]
    pub fn agentic_expansion_plan(&self, query: Option<&str>) -> LinkGraphAgenticExpansionPlan {
        self.agentic_expansion_plan_with_config(query, Self::resolve_agentic_expansion_config())
    }

    /// Build a bounded sub-agent expansion plan with explicit runtime budgets.
    #[must_use]
    pub fn agentic_expansion_plan_with_config(
        &self,
        query: Option<&str>,
        config: LinkGraphAgenticExpansionConfig,
    ) -> LinkGraphAgenticExpansionPlan {
        super::plan::agentic_expansion_plan_with_config(self, query, config)
    }

    /// Execute bounded sub-agent expansion workers using runtime default config.
    ///
    /// This runs the planner first, then processes candidate pairs per worker
    /// with runtime budgets and optional suggested-link persistence.
    #[must_use]
    pub fn agentic_expansion_execute(
        &self,
        query: Option<&str>,
    ) -> LinkGraphAgenticExecutionResult {
        self.agentic_expansion_execute_with_config(query, Self::resolve_agentic_execution_config())
    }

    /// Execute bounded sub-agent expansion workers with explicit runtime config.
    #[must_use]
    pub fn agentic_expansion_execute_with_config(
        &self,
        query: Option<&str>,
        config: LinkGraphAgenticExecutionConfig,
    ) -> LinkGraphAgenticExecutionResult {
        super::execute::agentic_expansion_execute_with_config(self, query, config)
    }
}
