use super::assert_local_business_rule_task;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_offset_dmn_decision_locally() {
    assert_local_business_rule_task(
        "dmn_local_offset",
        "wf_dmn_local_offset",
        "release-window-offset",
        "datetime-comparison-release-window-offset.dmn",
        json!({ "release_timestamp": "2026-04-21T09:00:00+09:00" }),
        json!({
            "release_timestamp": "2026-04-21T09:00:00+09:00",
            "phase": "post-day-one-offset"
        }),
        71,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_coerces_mixed_datetime_forms_through_utc() {
    assert_local_business_rule_task(
        "dmn_local_mixed",
        "wf_dmn_local_mixed",
        "release-window-offset",
        "datetime-comparison-release-window-offset.dmn",
        json!({ "release_timestamp": "2026-04-20T23:59:59" }),
        json!({
            "release_timestamp": "2026-04-20T23:59:59",
            "phase": "day-one-offset"
        }),
        72,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_coerces_mixed_datetime_literal_equality_through_utc() {
    assert_local_business_rule_task(
        "dmn_local_mixed_literal",
        "wf_dmn_local_mixed_literal",
        "release-window-offset",
        "datetime-comparison-release-window-offset.dmn",
        json!({ "release_timestamp": "2026-04-20T09:00:00" }),
        json!({
            "release_timestamp": "2026-04-20T09:00:00",
            "phase": "launch-minute-offset"
        }),
        73,
    )
    .await;
}
