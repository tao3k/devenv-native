//! Route trace contract payloads for agent audit trails.

use serde::{Deserialize, Serialize};

use super::{
    GraphPlanStepKind, GraphWorkflowMode, OmegaFallbackPolicy, OmegaRiskLevel, OmegaRoute,
    OmegaToolTrustClass,
};

/// Aggregated per-step route trace emitted once per turn execution outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteTrace {
    /// Logical session identifier associated with this turn.
    pub session_id: String,
    /// Monotonic turn identifier within the session.
    pub turn_id: u64,
    /// Route selected by Omega for this turn.
    pub selected_route: OmegaRoute,
    /// Confidence score assigned to the selected route.
    pub confidence: f32,
    /// Risk classification associated with this turn.
    pub risk_level: OmegaRiskLevel,
    /// Trust class used for tool-governance decisions.
    pub tool_trust_class: OmegaToolTrustClass,
    /// Whether fallback was applied during execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_applied: Option<bool>,
    /// Fallback policy used when route execution degraded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_policy: Option<OmegaFallbackPolicy>,
    /// Ordered tool chain touched while completing the turn.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tool_chain: Vec<String>,
    /// End-to-end latency for turn execution in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<f64>,
    /// Failure taxonomy tags observed during execution.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub failure_taxonomy: Vec<String>,
    /// Injection summary when qianhuan context injection was active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub injection: Option<RouteTraceInjection>,
    /// Optional deterministic graph plan identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
    /// Optional graph workflow mode used by the execution plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_mode: Option<GraphWorkflowMode>,
    /// Optional per-step graph trace for deterministic graph routes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_steps: Option<Vec<RouteTraceGraphStep>>,
}

/// Injection summary attached to route trace when qianhuan context exists.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteTraceInjection {
    /// Number of injection blocks accepted for this turn.
    pub blocks_used: u64,
    /// Total injected characters.
    pub chars_injected: u64,
    /// Number of candidate blocks dropped due to budget limits.
    pub dropped_by_budget: u64,
}

/// Per-step execution trace for deterministic graph routes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteTraceGraphStep {
    /// One-based step index in plan execution order.
    pub index: u8,
    /// Stable step identifier.
    pub id: String,
    /// Step kind from deterministic graph-plan contract.
    pub kind: GraphPlanStepKind,
    /// Attempt counter for retries of the same step.
    pub attempt: u32,
    /// Step latency in milliseconds.
    pub latency_ms: f64,
    /// Terminal status label for the step.
    pub status: String,
    /// Failure reason when status is non-success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    /// Tool name when the step invokes a tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Fallback action applied by the step, when relevant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_action: Option<String>,
}
