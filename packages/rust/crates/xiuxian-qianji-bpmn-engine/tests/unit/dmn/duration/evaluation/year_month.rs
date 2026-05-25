use super::evaluate_fixture;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_year_month_duration_comparisons() {
    let exact = evaluate_fixture(
        "retention-window",
        "year-month-duration-comparison-retention-window.dmn",
        json!({ "term": "P6M" }),
        "year-month duration comparison DMN source should parse",
        "year-month duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(exact.output, json!({ "band": "exact-half-year" }));
    assert_eq!(exact.matched_rule_ids[0].as_ref(), "rule_exact_half_year");

    let under = evaluate_fixture(
        "retention-window",
        "year-month-duration-comparison-retention-window.dmn",
        json!({ "term": "P9M" }),
        "year-month duration comparison DMN source should parse",
        "year-month duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(under.output, json!({ "band": "under-year" }));
    assert_eq!(under.matched_rule_ids[0].as_ref(), "rule_under_year");

    let over = evaluate_fixture(
        "retention-window",
        "year-month-duration-comparison-retention-window.dmn",
        json!({ "term": "P1Y" }),
        "year-month duration comparison DMN source should parse",
        "year-month duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(over.output, json!({ "band": "year-or-more" }));
    assert_eq!(over.matched_rule_ids[0].as_ref(), "rule_year_or_more");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_year_month_duration_ranges() {
    let mid_term = evaluate_fixture(
        "contract-term",
        "year-month-duration-range-contract-term.dmn",
        json!({ "term": "P9M" }),
        "year-month duration range DMN source should parse",
        "year-month duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(mid_term.output, json!({ "window": "mid-term" }));
    assert_eq!(mid_term.matched_rule_ids[0].as_ref(), "rule_mid_term");

    let annual = evaluate_fixture(
        "contract-term",
        "year-month-duration-range-contract-term.dmn",
        json!({ "term": "P1Y6M" }),
        "year-month duration range DMN source should parse",
        "year-month duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(annual.output, json!({ "window": "annual-window" }));
    assert_eq!(annual.matched_rule_ids[0].as_ref(), "rule_annual_window");

    let outside = evaluate_fixture(
        "contract-term",
        "year-month-duration-range-contract-term.dmn",
        json!({ "term": "P3M" }),
        "year-month duration range DMN source should parse",
        "year-month duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(outside.output, json!({ "window": "outside-window" }));
    assert_eq!(outside.matched_rule_ids[0].as_ref(), "rule_outside_window");
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_negative_year_month_duration_ranges() {
    let long_past = evaluate_fixture(
        "account-window",
        "negative-year-month-duration-range-account-window.dmn",
        json!({ "term": "-P9M" }),
        "negative year-month duration range DMN source should parse",
        "negative year-month duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(long_past.output, json!({ "window": "long-past" }));
    assert_eq!(long_past.matched_rule_ids[0].as_ref(), "rule_long_past");

    let recent_or_current = evaluate_fixture(
        "account-window",
        "negative-year-month-duration-range-account-window.dmn",
        json!({ "term": "-P3M" }),
        "negative year-month duration range DMN source should parse",
        "negative year-month duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(
        recent_or_current.output,
        json!({ "window": "recent-or-current" })
    );
    assert_eq!(
        recent_or_current.matched_rule_ids[0].as_ref(),
        "rule_recent_or_current"
    );

    let future = evaluate_fixture(
        "account-window",
        "negative-year-month-duration-range-account-window.dmn",
        json!({ "term": "P3M" }),
        "negative year-month duration range DMN source should parse",
        "negative year-month duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(future.output, json!({ "window": "future" }));
    assert_eq!(future.matched_rule_ids[0].as_ref(), "rule_future");
}
