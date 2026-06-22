use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use serde_json::json;
use xiuxian_qianji_bpmn_engine::{
    DmnDecisionRef, DmnEvaluationRequest, evaluate_dmn_decision, parse_dmn_decision,
};

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_coerces_local_literal_rule_and_offset_input_through_utc() {
    let decision = parse_dmn_decision(&fixture_source("datetime-comparison-release-window.dmn"))
        .must("datetime comparison DMN source should parse");

    let exact = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window")
                .with_source_id("datetime-comparison-release-window.dmn"),
            json!({ "release_timestamp": "2026-04-20T09:00:00Z" }),
        ),
    )
    .await
    .must("mixed local-literal/offset-input evaluation should run");
    assert_eq!(exact.output, json!({ "phase": "launch-minute" }));
    assert_eq!(exact.matched_rule_ids[0].as_ref(), "rule_launch_minute");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_coerces_offset_literal_rule_and_local_input_through_utc() {
    let decision = parse_dmn_decision(&fixture_source(
        "datetime-comparison-release-window-offset.dmn",
    ))
    .must("offset datetime comparison DMN source should parse");

    let exact = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window-offset")
                .with_source_id("datetime-comparison-release-window-offset.dmn"),
            json!({ "release_timestamp": "2026-04-20T09:00:00" }),
        ),
    )
    .await
    .must("mixed offset-literal/local-input evaluation should run");
    assert_eq!(exact.output, json!({ "phase": "launch-minute-offset" }));
    assert_eq!(
        exact.matched_rule_ids[0].as_ref(),
        "rule_launch_minute_offset"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_coerces_local_rule_and_offset_input_through_utc() {
    let decision = parse_dmn_decision(&fixture_source("datetime-comparison-release-window.dmn"))
        .must("datetime comparison DMN source should parse");

    let after = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window")
                .with_source_id("datetime-comparison-release-window.dmn"),
            json!({ "release_timestamp": "2026-04-21T00:00:00+00:00" }),
        ),
    )
    .await
    .must("mixed local-rule/offset-input evaluation should run");
    assert_eq!(after.output, json!({ "phase": "post-day-one" }));
    assert_eq!(after.matched_rule_ids[0].as_ref(), "rule_post_day_one");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_coerces_offset_rule_and_local_input_through_utc() {
    let decision = parse_dmn_decision(&fixture_source(
        "datetime-comparison-release-window-offset.dmn",
    ))
    .must("offset datetime comparison DMN source should parse");

    let before = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window-offset")
                .with_source_id("datetime-comparison-release-window-offset.dmn"),
            json!({ "release_timestamp": "2026-04-20T23:59:59" }),
        ),
    )
    .await
    .must("mixed offset-rule/local-input evaluation should run");
    assert_eq!(before.output, json!({ "phase": "day-one-offset" }));
    assert_eq!(before.matched_rule_ids[0].as_ref(), "rule_day_one_offset");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_coerces_local_range_and_offset_input_through_utc() {
    let decision = parse_dmn_decision(&fixture_source("datetime-range-maintenance-window.dmn"))
        .must("datetime range DMN source should parse");

    let morning = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window")
                .with_source_id("datetime-range-maintenance-window.dmn"),
            json!({ "maintenance_at": "2026-05-01T09:30:00Z" }),
        ),
    )
    .await
    .must("mixed local-range/offset-input evaluation should run");
    assert_eq!(morning.output, json!({ "window": "morning-maintenance" }));
    assert_eq!(morning.matched_rule_ids[0].as_ref(), "rule_morning_window");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_coerces_offset_range_and_local_input_through_utc() {
    let decision = parse_dmn_decision(&fixture_source(
        "datetime-range-maintenance-window-offset.dmn",
    ))
    .must("offset datetime range DMN source should parse");

    let morning = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window-offset")
                .with_source_id("datetime-range-maintenance-window-offset.dmn"),
            json!({ "maintenance_at": "2026-05-01T00:30:00" }),
        ),
    )
    .await
    .must("mixed offset-range/local-input evaluation should run");
    assert_eq!(
        morning.output,
        json!({ "window": "morning-maintenance-offset" })
    );
    assert_eq!(
        morning.matched_rule_ids[0].as_ref(),
        "rule_morning_window_offset"
    );
}
