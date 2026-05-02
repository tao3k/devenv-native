use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    DmnDecisionRef, DmnEvaluationRequest, DmnEvaluationResult, evaluate_dmn_decision,
    parse_dmn_decision,
};
use serde_json::Value;

mod day_time;
mod fractional;
mod negative;
mod year_month;

async fn evaluate_fixture(
    decision_id: &str,
    fixture_name: &str,
    input: Value,
    parse_context: &str,
    evaluation_context: &str,
) -> DmnEvaluationResult {
    let decision = parse_dmn_decision(&fixture_source(fixture_name)).must(parse_context);
    evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new(decision_id).with_source_id(fixture_name),
            input,
        ),
    )
    .await
    .must(evaluation_context)
}
