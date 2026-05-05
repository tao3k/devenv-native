//! Graph-planning contract payloads for execution routing.

use serde::{Deserialize, Serialize};

use super::{OmegaFallbackPolicy, OmegaRoute};

const SUPPORTED_FALLBACK_ACTIONS: &[&str] = &[
    "abort",
    "retry_react",
    "route_to_react",
    "retry_bridge_without_metadata",
];

/// Workflow mode for deterministic graph-plan execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphWorkflowMode {
    /// Direct graph execution.
    Graph,
    /// Graph execution orchestrated by omega reasoning.
    Omega,
}

/// Deterministic graph-plan step kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphPlanStepKind {
    /// Prepare context/injection metadata for graph tool call.
    PrepareInjectionContext,
    /// Invoke the selected graph bridge tool.
    InvokeGraphTool,
    /// Evaluate fallback action after bridge invocation.
    EvaluateFallback,
}

/// One step in deterministic graph execution plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPlanStep {
    /// One-based step index in execution order.
    pub index: u8,
    /// Stable step identifier.
    pub id: String,
    /// Step kind in the graph-plan lifecycle.
    pub kind: GraphPlanStepKind,
    /// Human-readable description for diagnostics and audits.
    pub description: String,
    /// Tool name for invocation steps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Fallback action for fallback-evaluation steps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_action: Option<String>,
}

/// Deterministic graph execution plan contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphExecutionPlan {
    /// Stable plan identifier.
    pub plan_id: String,
    /// Plan contract version.
    pub plan_version: String,
    /// Route selected for this deterministic plan.
    pub route: OmegaRoute,
    /// Workflow mode used for this plan.
    pub workflow_mode: GraphWorkflowMode,
    /// Primary graph tool invoked by this plan.
    pub tool_name: String,
    /// Fallback policy to apply on execution failure.
    pub fallback_policy: OmegaFallbackPolicy,
    /// Ordered deterministic execution steps.
    pub steps: Vec<GraphPlanStep>,
}

impl GraphExecutionPlan {
    /// Validate deterministic v1 graph-plan contract.
    ///
    /// # Errors
    /// Returns an error when step ordering, kinds, or fallback action violates the contract.
    pub fn validate_shortcut_contract(&self) -> Result<(), String> {
        if self.plan_version != "v1" {
            return Err(format!(
                "unsupported graph plan version `{}` (expected `v1`)",
                self.plan_version
            ));
        }
        if self.plan_id.trim().is_empty() {
            return Err("graph plan id must be non-empty".to_string());
        }
        if self.steps.len() != 3 {
            return Err(format!(
                "graph plan must contain exactly 3 steps (got {})",
                self.steps.len()
            ));
        }

        let mut ordered = self.steps.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|step| step.index);

        if ordered[0].index != 1 || ordered[1].index != 2 || ordered[2].index != 3 {
            return Err("step ordering is invalid: expected consecutive indices 1..=3".to_string());
        }

        if ordered[0].kind != GraphPlanStepKind::PrepareInjectionContext {
            return Err(
                "step ordering is invalid: step 1 must be prepare_injection_context".to_string(),
            );
        }
        if ordered[1].kind != GraphPlanStepKind::InvokeGraphTool {
            return Err("step ordering is invalid: step 2 must be invoke_graph_tool".to_string());
        }
        if ordered[2].kind != GraphPlanStepKind::EvaluateFallback {
            return Err("step ordering is invalid: step 3 must be evaluate_fallback".to_string());
        }

        if ordered[0].tool_name.is_some() || ordered[0].fallback_action.is_some() {
            return Err(
                "prepare_injection_context step must not define tool/fallback fields".into(),
            );
        }
        if ordered[1].tool_name.as_deref().is_none_or(str::is_empty) {
            return Err("invoke_graph_tool step must define non-empty tool_name".into());
        }
        if ordered[1].fallback_action.is_some() {
            return Err("invoke_graph_tool step must not define fallback_action".into());
        }

        let Some(fallback_action) = ordered[2].fallback_action.as_deref() else {
            return Err("evaluate_fallback step must define fallback_action".into());
        };
        if !SUPPORTED_FALLBACK_ACTIONS.contains(&fallback_action) {
            return Err(format!(
                "unsupported fallback_action `{fallback_action}` in evaluate_fallback step"
            ));
        }
        if ordered[2].tool_name.is_some() {
            return Err("evaluate_fallback step must not define tool_name".into());
        }

        Ok(())
    }
}
