use crate::agent::reflection::PolicyHintDirective;
use crate::agent::{Agent, omega};
use crate::contracts::{OmegaDecision, OmegaFallbackPolicy};
use crate::observability::SessionEvent;

impl Agent {
    pub(super) async fn prepare_react_decision(
        &self,
        session_id: &str,
        force_react: bool,
    ) -> (OmegaDecision, Option<PolicyHintDirective>) {
        let policy_hint = self.take_reflection_policy_hint(session_id).await;
        if let Some(hint) = policy_hint.as_ref() {
            tracing::debug!(
                event = SessionEvent::ReflectionPolicyHintApplied.as_str(),
                session_id,
                source_turn_id = hint.source_turn_id,
                preferred_route = hint.preferred_route.as_str(),
                risk_floor = hint.risk_floor.as_str(),
                fallback_override = hint.fallback_override.map(OmegaFallbackPolicy::as_str),
                tool_trust_class = hint.tool_trust_class.as_str(),
                reason = %hint.reason,
                "reflection policy hint applied to route decision"
            );
        }
        let decision = omega::apply_quality_gate(omega::apply_policy_hint(
            omega::decide_for_standard_turn(force_react),
            policy_hint.as_ref(),
        ));
        Self::record_omega_decision(session_id, &decision, None, None);
        (decision, policy_hint)
    }
}
