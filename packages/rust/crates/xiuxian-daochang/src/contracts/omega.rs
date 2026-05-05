//! Omega decision contract payloads for strategic supervision.

use serde::{Deserialize, Serialize};

/// Route selected by Omega governance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OmegaRoute {
    /// Fast-path `ReAct` route.
    React,
    /// Deterministic graph route.
    Graph,
}

impl OmegaRoute {
    /// Return canonical `snake_case` label used in telemetry and contracts.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::React => "react",
            Self::Graph => "graph",
        }
    }
}

/// Risk classification emitted by Omega route policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OmegaRiskLevel {
    /// Low-risk execution context.
    Low,
    /// Medium-risk execution context.
    Medium,
    /// High-risk execution context.
    High,
    /// Critical-risk execution context.
    Critical,
}

impl OmegaRiskLevel {
    /// Return canonical `snake_case` label used in telemetry and contracts.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Fallback action when selected route fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OmegaFallbackPolicy {
    /// Retry on the `ReAct` route.
    RetryReact,
    /// Switch execution to graph route.
    SwitchToGraph,
    /// Abort execution and surface failure.
    Abort,
}

impl OmegaFallbackPolicy {
    /// Return canonical `snake_case` label used in telemetry and contracts.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RetryReact => "retry_react",
            Self::SwitchToGraph => "switch_to_graph",
            Self::Abort => "abort",
        }
    }
}

/// Tool trust class emitted by Omega for execution governance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OmegaToolTrustClass {
    /// Evidence-producing tool class.
    Evidence,
    /// Verification-oriented tool class.
    Verification,
    /// Other/uncategorized tool class.
    Other,
}

impl OmegaToolTrustClass {
    /// Return canonical `snake_case` label used in telemetry and contracts.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Evidence => "evidence",
            Self::Verification => "verification",
            Self::Other => "other",
        }
    }
}

/// Decision envelope emitted by Omega.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OmegaDecision {
    /// Selected execution route.
    pub route: OmegaRoute,
    /// Calibrated confidence for route selection.
    pub confidence: f32,
    /// Risk class for this decision.
    pub risk_level: OmegaRiskLevel,
    /// Fallback policy if execution fails.
    pub fallback_policy: OmegaFallbackPolicy,
    /// Tool trust class for this route decision.
    pub tool_trust_class: OmegaToolTrustClass,
    /// Human/audit-readable rationale.
    pub reason: String,
    /// Optional policy profile identifier.
    pub policy_id: Option<String>,
    /// 2026 Strategic Extension: Threshold for trajectory drift detection.
    /// If semantic drift exceeds this value, Omega triggers re-planning.
    pub drift_tolerance: Option<f32>,
    /// Turn index when the next strategic re-calibration is mandatory.
    pub next_audit_turn: Option<u32>,
}
