use super::evaluate_fixture;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_duration_comparisons() {
    let exact = evaluate_fixture(
        "sla-window",
        "duration-comparison-sla-window.dmn",
        json!({ "elapsed": "PT30M" }),
        "duration comparison DMN source should parse",
        "duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(exact.output, json!({ "band": "exact-half-hour" }));
    assert_eq!(exact.matched_rule_ids[0].as_ref(), "rule_exact_half_hour");

    let under = evaluate_fixture(
        "sla-window",
        "duration-comparison-sla-window.dmn",
        json!({ "elapsed": "PT45M" }),
        "duration comparison DMN source should parse",
        "duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(under.output, json!({ "band": "under-hour" }));
    assert_eq!(under.matched_rule_ids[0].as_ref(), "rule_under_hour");

    let over = evaluate_fixture(
        "sla-window",
        "duration-comparison-sla-window.dmn",
        json!({ "elapsed": "PT1H" }),
        "duration comparison DMN source should parse",
        "duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(over.output, json!({ "band": "hour-or-more" }));
    assert_eq!(over.matched_rule_ids[0].as_ref(), "rule_hour_or_more");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_duration_ranges() {
    let short = evaluate_fixture(
        "review-delay",
        "duration-range-review-delay.dmn",
        json!({ "elapsed": "PT30M" }),
        "duration range DMN source should parse",
        "duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(short.output, json!({ "window": "short-delay" }));
    assert_eq!(short.matched_rule_ids[0].as_ref(), "rule_short_delay");

    let day_window = evaluate_fixture(
        "review-delay",
        "duration-range-review-delay.dmn",
        json!({ "elapsed": "P1DT1H30M" }),
        "duration range DMN source should parse",
        "duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(day_window.output, json!({ "window": "day-window" }));
    assert_eq!(day_window.matched_rule_ids[0].as_ref(), "rule_day_window");

    let outside = evaluate_fixture(
        "review-delay",
        "duration-range-review-delay.dmn",
        json!({ "elapsed": "PT5M" }),
        "duration range DMN source should parse",
        "duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(outside.output, json!({ "window": "outside-window" }));
    assert_eq!(outside.matched_rule_ids[0].as_ref(), "rule_outside_window");
}
