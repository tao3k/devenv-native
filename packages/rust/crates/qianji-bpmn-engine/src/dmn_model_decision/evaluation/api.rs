//! Public dmn model decision evaluation contracts for BPMN/DMN engine integration.

use super::{Arc, DmnDecisionRef, Value};

/// DMN evaluation request surface.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnEvaluationRequest {
    /// Target decision reference.
    pub decision: DmnDecisionRef,
    /// Input variables supplied by the host.
    pub variables: Value,
}

impl DmnEvaluationRequest {
    /// Creates one DMN evaluation request.
    #[must_use]
    pub fn new(decision: DmnDecisionRef, variables: Value) -> Self {
        Self {
            decision,
            variables,
        }
    }
}

/// DMN evaluation result surface.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnEvaluationResult {
    /// Evaluated decision identity.
    pub decision_id: Arc<str>,
    /// Output payload.
    pub output: Value,
    /// Rule identifiers that matched during evaluation.
    pub matched_rule_ids: Vec<Arc<str>>,
}

impl DmnEvaluationResult {
    /// Creates one DMN evaluation result.
    #[must_use]
    pub fn new(
        decision_id: impl AsRef<str>,
        output: Value,
        matched_rule_ids: Vec<Arc<str>>,
    ) -> Self {
        Self {
            decision_id: Arc::<str>::from(decision_id.as_ref()),
            output,
            matched_rule_ids,
        }
    }
}
