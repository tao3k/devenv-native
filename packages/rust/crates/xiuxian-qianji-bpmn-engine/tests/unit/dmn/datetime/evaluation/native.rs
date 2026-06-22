use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use serde_json::json;
use xiuxian_qianji_bpmn_engine::{
    DmnDecisionRef, DmnEvaluationRequest, evaluate_dmn_decision, parse_dmn_decision,
};

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_datetime_comparisons() {
    let decision = parse_dmn_decision(&fixture_source("datetime-comparison-release-window.dmn"))
        .must("datetime comparison DMN source should parse");

    let exact = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window")
                .with_source_id("datetime-comparison-release-window.dmn"),
            json!({ "release_timestamp": "2026-04-20T09:00:00" }),
        ),
    )
    .await
    .must("datetime comparison DMN evaluator should run");
    assert_eq!(exact.output, json!({ "phase": "launch-minute" }));
    assert_eq!(exact.matched_rule_ids[0].as_ref(), "rule_launch_minute");

    let before = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window")
                .with_source_id("datetime-comparison-release-window.dmn"),
            json!({ "release_timestamp": "2026-04-20T12:00:00" }),
        ),
    )
    .await
    .must("datetime comparison DMN evaluator should run");
    assert_eq!(before.output, json!({ "phase": "day-one" }));
    assert_eq!(before.matched_rule_ids[0].as_ref(), "rule_day_one");

    let after = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window")
                .with_source_id("datetime-comparison-release-window.dmn"),
            json!({ "release_timestamp": "2026-04-21T00:00:00" }),
        ),
    )
    .await
    .must("datetime comparison DMN evaluator should run");
    assert_eq!(after.output, json!({ "phase": "post-day-one" }));
    assert_eq!(after.matched_rule_ids[0].as_ref(), "rule_post_day_one");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_datetime_ranges() {
    let decision = parse_dmn_decision(&fixture_source("datetime-range-maintenance-window.dmn"))
        .must("datetime range DMN source should parse");

    let morning = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window")
                .with_source_id("datetime-range-maintenance-window.dmn"),
            json!({ "maintenance_at": "2026-05-01T09:30:00" }),
        ),
    )
    .await
    .must("datetime range DMN evaluator should run");
    assert_eq!(morning.output, json!({ "window": "morning-maintenance" }));
    assert_eq!(morning.matched_rule_ids[0].as_ref(), "rule_morning_window");

    let afternoon = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window")
                .with_source_id("datetime-range-maintenance-window.dmn"),
            json!({ "maintenance_at": "2026-05-01T15:00:00" }),
        ),
    )
    .await
    .must("datetime range DMN evaluator should run");
    assert_eq!(
        afternoon.output,
        json!({ "window": "afternoon-maintenance" })
    );
    assert_eq!(
        afternoon.matched_rule_ids[0].as_ref(),
        "rule_afternoon_window"
    );

    let outside = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window")
                .with_source_id("datetime-range-maintenance-window.dmn"),
            json!({ "maintenance_at": "2026-05-01T12:30:00" }),
        ),
    )
    .await
    .must("datetime range DMN evaluator should run");
    assert_eq!(outside.output, json!({ "window": "outside-window" }));
    assert_eq!(outside.matched_rule_ids[0].as_ref(), "rule_outside_window");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_offset_datetime_comparisons() {
    let decision = parse_dmn_decision(&fixture_source(
        "datetime-comparison-release-window-offset.dmn",
    ))
    .must("offset datetime comparison DMN source should parse");

    let exact = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window-offset")
                .with_source_id("datetime-comparison-release-window-offset.dmn"),
            json!({ "release_timestamp": "2026-04-20T09:00:00Z" }),
        ),
    )
    .await
    .must("offset datetime comparison DMN evaluator should run");
    assert_eq!(exact.output, json!({ "phase": "launch-minute-offset" }));
    assert_eq!(
        exact.matched_rule_ids[0].as_ref(),
        "rule_launch_minute_offset"
    );

    let before = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window-offset")
                .with_source_id("datetime-comparison-release-window-offset.dmn"),
            json!({ "release_timestamp": "2026-04-21T08:59:59+09:00" }),
        ),
    )
    .await
    .must("offset datetime comparison DMN evaluator should run");
    assert_eq!(before.output, json!({ "phase": "day-one-offset" }));
    assert_eq!(before.matched_rule_ids[0].as_ref(), "rule_day_one_offset");

    let after = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("release-window-offset")
                .with_source_id("datetime-comparison-release-window-offset.dmn"),
            json!({ "release_timestamp": "2026-04-21T09:00:00+09:00" }),
        ),
    )
    .await
    .must("offset datetime comparison DMN evaluator should run");
    assert_eq!(after.output, json!({ "phase": "post-day-one-offset" }));
    assert_eq!(
        after.matched_rule_ids[0].as_ref(),
        "rule_post_day_one_offset"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_offset_datetime_ranges() {
    let decision = parse_dmn_decision(&fixture_source(
        "datetime-range-maintenance-window-offset.dmn",
    ))
    .must("offset datetime range DMN source should parse");

    let morning = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window-offset")
                .with_source_id("datetime-range-maintenance-window-offset.dmn"),
            json!({ "maintenance_at": "2026-05-01T00:30:00Z" }),
        ),
    )
    .await
    .must("offset datetime range DMN evaluator should run");
    assert_eq!(
        morning.output,
        json!({ "window": "morning-maintenance-offset" })
    );
    assert_eq!(
        morning.matched_rule_ids[0].as_ref(),
        "rule_morning_window_offset"
    );

    let afternoon = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window-offset")
                .with_source_id("datetime-range-maintenance-window-offset.dmn"),
            json!({ "maintenance_at": "2026-05-01T06:00:00Z" }),
        ),
    )
    .await
    .must("offset datetime range DMN evaluator should run");
    assert_eq!(
        afternoon.output,
        json!({ "window": "afternoon-maintenance-offset" })
    );
    assert_eq!(
        afternoon.matched_rule_ids[0].as_ref(),
        "rule_afternoon_window_offset"
    );

    let outside = evaluate_dmn_decision(
        &decision,
        &DmnEvaluationRequest::new(
            DmnDecisionRef::new("maintenance-window-offset")
                .with_source_id("datetime-range-maintenance-window-offset.dmn"),
            json!({ "maintenance_at": "2026-05-01T03:30:00Z" }),
        ),
    )
    .await
    .must("offset datetime range DMN evaluator should run");
    assert_eq!(outside.output, json!({ "window": "outside-window-offset" }));
    assert_eq!(
        outside.matched_rule_ids[0].as_ref(),
        "rule_outside_window_offset"
    );
}
