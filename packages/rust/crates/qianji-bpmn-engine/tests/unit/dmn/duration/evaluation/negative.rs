use super::evaluate_fixture;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_negative_duration_comparisons() {
    let exact = evaluate_fixture(
        "recovery-window",
        "negative-duration-comparison-recovery-window.dmn",
        json!({ "elapsed": "-PT30M" }),
        "negative duration comparison DMN source should parse",
        "negative duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(exact.output, json!({ "band": "exact-negative-half-hour" }));
    assert_eq!(
        exact.matched_rule_ids[0].as_ref(),
        "rule_exact_negative_half_hour"
    );

    let before_zero = evaluate_fixture(
        "recovery-window",
        "negative-duration-comparison-recovery-window.dmn",
        json!({ "elapsed": "-PT15M" }),
        "negative duration comparison DMN source should parse",
        "negative duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(before_zero.output, json!({ "band": "before-zero" }));
    assert_eq!(before_zero.matched_rule_ids[0].as_ref(), "rule_before_zero");

    let zero_or_later = evaluate_fixture(
        "recovery-window",
        "negative-duration-comparison-recovery-window.dmn",
        json!({ "elapsed": "PT0S" }),
        "negative duration comparison DMN source should parse",
        "negative duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(zero_or_later.output, json!({ "band": "zero-or-later" }));
    assert_eq!(
        zero_or_later.matched_rule_ids[0].as_ref(),
        "rule_zero_or_later"
    );
}
