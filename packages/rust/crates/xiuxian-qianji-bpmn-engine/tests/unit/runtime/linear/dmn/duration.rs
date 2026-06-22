use super::assert_local_business_rule_task;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_duration_decision_locally() {
    assert_local_business_rule_task(
        "dmn_local_duration",
        "wf_dmn_local_duration",
        "review-delay",
        "duration-range-review-delay.dmn",
        json!({ "elapsed": "P1DT1H30M" }),
        json!({
            "elapsed": "P1DT1H30M",
            "window": "day-window"
        }),
        74,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_year_month_duration_decision_locally() {
    assert_local_business_rule_task(
        "dmn_local_year_month_duration",
        "wf_dmn_local_year_month_duration",
        "contract-term",
        "year-month-duration-range-contract-term.dmn",
        json!({ "term": "P1Y6M" }),
        json!({
            "term": "P1Y6M",
            "window": "annual-window"
        }),
        75,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_negative_duration_decision_locally() {
    assert_local_business_rule_task(
        "dmn_local_negative_duration",
        "wf_dmn_local_negative_duration",
        "recovery-window",
        "negative-duration-comparison-recovery-window.dmn",
        json!({ "elapsed": "-PT15M" }),
        json!({
            "elapsed": "-PT15M",
            "band": "before-zero"
        }),
        76,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_fractional_duration_decision_locally() {
    assert_local_business_rule_task(
        "dmn_local_fractional_duration",
        "wf_dmn_local_fractional_duration",
        "subsecond-range-window",
        "fractional-duration-range-subsecond-window.dmn",
        json!({ "elapsed": "-PT0.25S" }),
        json!({
            "elapsed": "-PT0.25S",
            "window": "centered-subsecond-window"
        }),
        77,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_fractional_day_time_unit_decision_locally()
{
    assert_local_business_rule_task(
        "dmn_local_fractional_day_time_unit",
        "wf_dmn_local_fractional_day_time_unit",
        "fractional-day-minute-window",
        "fractional-duration-range-day-minute-window.dmn",
        json!({ "elapsed": "PT2M" }),
        json!({
            "elapsed": "PT2M",
            "window": "minute-window"
        }),
        78,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_comma_day_time_unit_decision_locally() {
    assert_local_business_rule_task(
        "dmn_local_comma_day_time_unit",
        "wf_dmn_local_comma_day_time_unit",
        "comma-day-minute-window",
        "comma-duration-range-day-minute-window.dmn",
        json!({ "elapsed": "PT2M" }),
        json!({
            "elapsed": "PT2M",
            "window": "minute-window-comma"
        }),
        79,
    )
    .await;
}
