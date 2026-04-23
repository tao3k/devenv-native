//! Public DMN evaluation entry surface.

use crate::dmn::evaluate_dmn_decision_sync;
use crate::dmn_model_api::{DmnDecisionDefinition, DmnEvaluationRequest, DmnEvaluationResult};
use crate::error::Result;

/// Evaluates one bounded DMN decision request.
///
/// # Errors
///
/// Returns [`BpmnEngineError::DmnDecisionMismatch`] when the request references
/// a different decision than the parsed definition.
pub async fn evaluate_dmn_decision(
    decision: &DmnDecisionDefinition,
    request: &DmnEvaluationRequest,
) -> Result<DmnEvaluationResult> {
    evaluate_dmn_decision_sync(decision, request)
}
