//! Reflection helpers exposed for integration tests.

use std::fmt;

use crate::agent::reflection;
use crate::{OmegaFallbackPolicy, OmegaRiskLevel, OmegaRoute, OmegaToolTrustClass};

/// Reflection lifecycle stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestReflectiveRuntimeStage {
    /// Diagnose stage.
    Diagnose,
    /// Plan stage.
    Plan,
    /// Apply stage.
    Apply,
}

/// Runtime error for illegal reflection lifecycle transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestReflectiveRuntimeError {
    /// Previous stage before transition.
    pub from: Option<TestReflectiveRuntimeStage>,
    /// Requested transition target stage.
    pub to: TestReflectiveRuntimeStage,
}

impl fmt::Display for TestReflectiveRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from = self.from.map_or("none", TestReflectiveRuntimeStage::as_str);
        write!(
            f,
            "illegal reflection lifecycle transition: {from} -> {}",
            self.to.as_str()
        )
    }
}

impl std::error::Error for TestReflectiveRuntimeError {}

/// Test-facing reflection runtime lifecycle guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TestReflectiveRuntime {
    inner: reflection::ReflectiveRuntime,
}

/// Opaque turn reflection payload wrapper for test workflows.
#[derive(Debug, Clone, PartialEq)]
pub struct TestTurnReflection {
    inner: reflection::TurnReflection,
}

/// Test-facing policy hint payload.
#[derive(Debug, Clone, PartialEq)]
pub struct TestPolicyHintDirective {
    /// Source turn id used by routing policy.
    pub source_turn_id: u64,
    /// Preferred route for next turn.
    pub preferred_route: OmegaRoute,
    /// Confidence delta adjustment.
    pub confidence_delta: f32,
    /// Lower bound risk floor.
    pub risk_floor: OmegaRiskLevel,
    /// Optional fallback route override.
    pub fallback_override: Option<OmegaFallbackPolicy>,
    /// Tool trust class for execution strategy.
    pub tool_trust_class: OmegaToolTrustClass,
    /// Human-readable hint reason.
    pub reason: String,
}

impl TestReflectiveRuntimeStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Diagnose => "diagnose",
            Self::Plan => "plan",
            Self::Apply => "apply",
        }
    }
}

impl TestReflectiveRuntime {
    /// Return the current lifecycle stage.
    #[must_use]
    pub fn stage(self) -> Option<TestReflectiveRuntimeStage> {
        self.inner.stage().map(from_internal_stage)
    }

    /// Transition runtime lifecycle to the next stage.
    ///
    /// # Errors
    ///
    /// Returns an error when transition order is illegal.
    pub fn transition(
        &mut self,
        next: TestReflectiveRuntimeStage,
    ) -> Result<(), TestReflectiveRuntimeError> {
        self.inner
            .transition(to_internal_stage(next))
            .map_err(from_internal_error)
    }
}

/// Build turn reflection payload.
#[must_use]
pub fn test_build_turn_reflection(
    route: &str,
    user_message: &str,
    assistant_message: &str,
    outcome: &str,
    tool_calls: u32,
) -> TestTurnReflection {
    TestTurnReflection {
        inner: reflection::build_turn_reflection(
            route,
            user_message,
            assistant_message,
            outcome,
            tool_calls,
        ),
    }
}

/// Derive policy hint from one reflection snapshot.
#[must_use]
pub fn test_derive_policy_hint(
    reflection: &TestTurnReflection,
    source_turn: u64,
) -> Option<TestPolicyHintDirective> {
    let source_turn_id = source_turn;
    reflection::derive_policy_hint(&reflection.inner, source_turn_id).map(|hint| {
        TestPolicyHintDirective {
            source_turn_id: hint.source_turn_id,
            preferred_route: hint.preferred_route,
            confidence_delta: hint.confidence_delta,
            risk_floor: hint.risk_floor,
            fallback_override: hint.fallback_override,
            tool_trust_class: hint.tool_trust_class,
            reason: hint.reason,
        }
    })
}

fn to_internal_stage(stage: TestReflectiveRuntimeStage) -> reflection::ReflectiveRuntimeStage {
    match stage {
        TestReflectiveRuntimeStage::Diagnose => reflection::ReflectiveRuntimeStage::Diagnose,
        TestReflectiveRuntimeStage::Plan => reflection::ReflectiveRuntimeStage::Plan,
        TestReflectiveRuntimeStage::Apply => reflection::ReflectiveRuntimeStage::Apply,
    }
}

fn from_internal_stage(stage: reflection::ReflectiveRuntimeStage) -> TestReflectiveRuntimeStage {
    match stage {
        reflection::ReflectiveRuntimeStage::Diagnose => TestReflectiveRuntimeStage::Diagnose,
        reflection::ReflectiveRuntimeStage::Plan => TestReflectiveRuntimeStage::Plan,
        reflection::ReflectiveRuntimeStage::Apply => TestReflectiveRuntimeStage::Apply,
    }
}

fn from_internal_error(error: reflection::ReflectiveRuntimeError) -> TestReflectiveRuntimeError {
    TestReflectiveRuntimeError {
        from: error.from.map(from_internal_stage),
        to: from_internal_stage(error.to),
    }
}
