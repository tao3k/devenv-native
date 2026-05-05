//! Strategic routing and quality-gating engine.

use crate::agent::reflection::PolicyHintDirective;
use crate::contracts::{
    OmegaDecision, OmegaFallbackPolicy, OmegaRiskLevel, OmegaRoute, OmegaToolTrustClass,
};

const OMEGA_ROLE_MIX_NORMAL_POLICY_ID: &str = "omega.role_mix.normal.v1";
const OMEGA_ROLE_MIX_RECOVERY_POLICY_ID: &str = "omega.role_mix.recovery.v1";

/// Decides the routing strategy for a standard turn.
#[must_use]
pub fn decide_for_standard_turn(force_react: bool) -> OmegaDecision {
    if force_react {
        return OmegaDecision {
            route: OmegaRoute::React,
            confidence: 1.0,
            risk_level: OmegaRiskLevel::Low,
            fallback_policy: OmegaFallbackPolicy::Abort,
            tool_trust_class: OmegaToolTrustClass::Evidence,
            reason: "force_react triggered by system prefix".into(),
            policy_id: Some(OMEGA_ROLE_MIX_NORMAL_POLICY_ID.into()),
            drift_tolerance: Some(0.0),
            next_audit_turn: None,
        };
    }

    // Default to React loop for natural language interaction
    OmegaDecision {
        route: OmegaRoute::React,
        confidence: 0.74,
        risk_level: OmegaRiskLevel::Low,
        fallback_policy: OmegaFallbackPolicy::Abort,
        tool_trust_class: OmegaToolTrustClass::Other,
        reason: "default runtime policy selected React loop".into(),
        policy_id: Some(OMEGA_ROLE_MIX_NORMAL_POLICY_ID.into()),
        drift_tolerance: Some(0.1),
        next_audit_turn: Some(10),
    }
}

/// Applies quality gate rules to a decision.
#[must_use]
pub fn apply_quality_gate(mut decision: OmegaDecision) -> OmegaDecision {
    let mut repairs: Vec<&str> = Vec::new();
    let is_graph = decision.route == OmegaRoute::Graph;
    let is_high_risk = matches!(
        decision.risk_level,
        OmegaRiskLevel::High | OmegaRiskLevel::Critical
    );

    if is_graph && is_high_risk && decision.fallback_policy == OmegaFallbackPolicy::SwitchToGraph {
        decision.fallback_policy = OmegaFallbackPolicy::RetryReact;
        repairs.push("quality_gate=graph_retry_loop_guard;repair=fallback_policy:retry_react");
    }

    if is_graph && is_high_risk && decision.tool_trust_class != OmegaToolTrustClass::Verification {
        decision.tool_trust_class = OmegaToolTrustClass::Verification;
        repairs.push(
            "quality_gate=graph_high_risk_trust_upgrade;repair=tool_trust_class:verification",
        );
    }

    if !repairs.is_empty() {
        decision.reason = append_markers(decision.reason, &repairs);
        decision.policy_id = Some(OMEGA_ROLE_MIX_RECOVERY_POLICY_ID.to_string());
    }

    decision
}

/// Applies policy hints from reflection to a decision.
#[must_use]
pub fn apply_policy_hint(
    mut decision: OmegaDecision,
    hint: Option<&PolicyHintDirective>,
) -> OmegaDecision {
    let Some(hint) = hint else {
        return decision;
    };

    decision.route = hint.preferred_route;
    decision.confidence = (decision.confidence + hint.confidence_delta).clamp(0.0, 1.0);
    decision.risk_level = max_risk(decision.risk_level, hint.risk_floor);
    if let Some(fallback_override) = hint.fallback_override {
        decision.fallback_policy = fallback_override;
    }
    decision.tool_trust_class = hint.tool_trust_class;
    decision.reason = append_markers(
        decision.reason,
        &[&format!("policy_hint={}", hint.reason.trim())],
    );
    decision.policy_id = Some(resolve_role_mix_policy_id(
        decision.risk_level,
        decision.tool_trust_class,
    ));

    decision
}

fn append_markers(mut reason: String, markers: &[&str]) -> String {
    for marker in markers {
        let trimmed = marker.trim();
        if trimmed.is_empty() {
            continue;
        }
        reason.push_str("; ");
        reason.push_str(trimmed);
    }
    reason
}

fn max_risk(left: OmegaRiskLevel, right: OmegaRiskLevel) -> OmegaRiskLevel {
    if risk_rank(right) >= risk_rank(left) {
        right
    } else {
        left
    }
}

const fn risk_rank(level: OmegaRiskLevel) -> u8 {
    match level {
        OmegaRiskLevel::Low => 0,
        OmegaRiskLevel::Medium => 1,
        OmegaRiskLevel::High => 2,
        OmegaRiskLevel::Critical => 3,
    }
}

fn resolve_role_mix_policy_id(risk: OmegaRiskLevel, trust: OmegaToolTrustClass) -> String {
    let recovery = matches!(risk, OmegaRiskLevel::High | OmegaRiskLevel::Critical)
        || trust == OmegaToolTrustClass::Verification;
    if recovery {
        OMEGA_ROLE_MIX_RECOVERY_POLICY_ID.to_string()
    } else {
        OMEGA_ROLE_MIX_NORMAL_POLICY_ID.to_string()
    }
}
