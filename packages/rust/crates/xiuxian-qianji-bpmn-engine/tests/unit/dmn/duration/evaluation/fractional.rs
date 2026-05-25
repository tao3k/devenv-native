use super::evaluate_fixture;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_fractional_second_duration_comparisons() {
    let exact = evaluate_fixture(
        "subsecond-window",
        "fractional-duration-comparison-subsecond-window.dmn",
        json!({ "elapsed": "PT1.5S" }),
        "fractional-second duration comparison DMN source should parse",
        "fractional-second duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(
        exact.output,
        json!({ "band": "exact-one-point-five-seconds" })
    );
    assert_eq!(
        exact.matched_rule_ids[0].as_ref(),
        "rule_exact_one_point_five_seconds"
    );

    let below = evaluate_fixture(
        "subsecond-window",
        "fractional-duration-comparison-subsecond-window.dmn",
        json!({ "elapsed": "PT2S" }),
        "fractional-second duration comparison DMN source should parse",
        "fractional-second duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(
        below.output,
        json!({ "band": "before-two-point-two-five-seconds" })
    );
    assert_eq!(
        below.matched_rule_ids[0].as_ref(),
        "rule_before_two_point_two_five_seconds"
    );

    let at_or_above = evaluate_fixture(
        "subsecond-window",
        "fractional-duration-comparison-subsecond-window.dmn",
        json!({ "elapsed": "PT2.25S" }),
        "fractional-second duration comparison DMN source should parse",
        "fractional-second duration comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(
        at_or_above.output,
        json!({ "band": "two-point-two-five-seconds-or-more" })
    );
    assert_eq!(
        at_or_above.matched_rule_ids[0].as_ref(),
        "rule_two_point_two_five_seconds_or_more"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_fractional_second_duration_ranges() {
    let centered = evaluate_fixture(
        "subsecond-range-window",
        "fractional-duration-range-subsecond-window.dmn",
        json!({ "elapsed": "-PT0.25S" }),
        "fractional-second duration range DMN source should parse",
        "fractional-second duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(
        centered.output,
        json!({ "window": "centered-subsecond-window" })
    );
    assert_eq!(
        centered.matched_rule_ids[0].as_ref(),
        "rule_centered_subsecond_window"
    );

    let expanded = evaluate_fixture(
        "subsecond-range-window",
        "fractional-duration-range-subsecond-window.dmn",
        json!({ "elapsed": "PT2.5S" }),
        "fractional-second duration range DMN source should parse",
        "fractional-second duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(
        expanded.output,
        json!({ "window": "expanded-subsecond-window" })
    );
    assert_eq!(
        expanded.matched_rule_ids[0].as_ref(),
        "rule_expanded_subsecond_window"
    );

    let outside = evaluate_fixture(
        "subsecond-range-window",
        "fractional-duration-range-subsecond-window.dmn",
        json!({ "elapsed": "PT5S" }),
        "fractional-second duration range DMN source should parse",
        "fractional-second duration range DMN evaluator should run",
    )
    .await;
    assert_eq!(
        outside.output,
        json!({ "window": "outside-subsecond-window" })
    );
    assert_eq!(
        outside.matched_rule_ids[0].as_ref(),
        "rule_outside_subsecond_window"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_fractional_day_time_unit_comparisons() {
    let exact = evaluate_fixture(
        "fractional-hour-window",
        "fractional-duration-comparison-hour-window.dmn",
        json!({ "elapsed": "PT1.5H" }),
        "fractional day-time unit comparison DMN source should parse",
        "fractional day-time unit comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(
        exact.output,
        json!({ "band": "exact-one-point-five-hours" })
    );
    assert_eq!(
        exact.matched_rule_ids[0].as_ref(),
        "rule_exact_one_point_five_hours"
    );

    let below = evaluate_fixture(
        "fractional-hour-window",
        "fractional-duration-comparison-hour-window.dmn",
        json!({ "elapsed": "PT2H" }),
        "fractional day-time unit comparison DMN source should parse",
        "fractional day-time unit comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(
        below.output,
        json!({ "band": "before-two-point-two-five-hours" })
    );
    assert_eq!(
        below.matched_rule_ids[0].as_ref(),
        "rule_before_two_point_two_five_hours"
    );

    let at_or_above = evaluate_fixture(
        "fractional-hour-window",
        "fractional-duration-comparison-hour-window.dmn",
        json!({ "elapsed": "PT2.25H" }),
        "fractional day-time unit comparison DMN source should parse",
        "fractional day-time unit comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(
        at_or_above.output,
        json!({ "band": "two-point-two-five-hours-or-more" })
    );
    assert_eq!(
        at_or_above.matched_rule_ids[0].as_ref(),
        "rule_two_point_two_five_hours_or_more"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_comma_day_time_unit_comparisons() {
    let exact = evaluate_fixture(
        "comma-hour-window",
        "comma-duration-comparison-hour-window.dmn",
        json!({ "elapsed": "PT1,5H" }),
        "comma day-time unit comparison DMN source should parse",
        "comma day-time unit comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(
        exact.output,
        json!({ "band": "exact-one-point-five-hours-comma" })
    );
    assert_eq!(
        exact.matched_rule_ids[0].as_ref(),
        "rule_exact_one_point_five_hours_comma"
    );

    let below = evaluate_fixture(
        "comma-hour-window",
        "comma-duration-comparison-hour-window.dmn",
        json!({ "elapsed": "PT2H" }),
        "comma day-time unit comparison DMN source should parse",
        "comma day-time unit comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(
        below.output,
        json!({ "band": "before-two-point-two-five-hours-comma" })
    );
    assert_eq!(
        below.matched_rule_ids[0].as_ref(),
        "rule_before_two_point_two_five_hours_comma"
    );

    let at_or_above = evaluate_fixture(
        "comma-hour-window",
        "comma-duration-comparison-hour-window.dmn",
        json!({ "elapsed": "PT2,25H" }),
        "comma day-time unit comparison DMN source should parse",
        "comma day-time unit comparison DMN evaluator should run",
    )
    .await;
    assert_eq!(
        at_or_above.output,
        json!({ "band": "two-point-two-five-hours-or-more-comma" })
    );
    assert_eq!(
        at_or_above.matched_rule_ids[0].as_ref(),
        "rule_two_point_two_five_hours_or_more_comma"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_fractional_day_time_unit_ranges() {
    let minute_window = evaluate_fixture(
        "fractional-day-minute-window",
        "fractional-duration-range-day-minute-window.dmn",
        json!({ "elapsed": "PT2M" }),
        "fractional day-time unit range DMN source should parse",
        "fractional day-time unit range DMN evaluator should run",
    )
    .await;
    assert_eq!(minute_window.output, json!({ "window": "minute-window" }));
    assert_eq!(
        minute_window.matched_rule_ids[0].as_ref(),
        "rule_fractional_minute_window"
    );

    let day_window = evaluate_fixture(
        "fractional-day-minute-window",
        "fractional-duration-range-day-minute-window.dmn",
        json!({ "elapsed": "P1.5D" }),
        "fractional day-time unit range DMN source should parse",
        "fractional day-time unit range DMN evaluator should run",
    )
    .await;
    assert_eq!(day_window.output, json!({ "window": "day-window" }));
    assert_eq!(
        day_window.matched_rule_ids[0].as_ref(),
        "rule_fractional_day_window"
    );

    let outside = evaluate_fixture(
        "fractional-day-minute-window",
        "fractional-duration-range-day-minute-window.dmn",
        json!({ "elapsed": "PT10M" }),
        "fractional day-time unit range DMN source should parse",
        "fractional day-time unit range DMN evaluator should run",
    )
    .await;
    assert_eq!(
        outside.output,
        json!({ "window": "outside-day-minute-window" })
    );
    assert_eq!(
        outside.matched_rule_ids[0].as_ref(),
        "rule_outside_fractional_day_minute_window"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn dmn_evaluation_unique_matches_comma_day_time_unit_ranges() {
    let minute_window = evaluate_fixture(
        "comma-day-minute-window",
        "comma-duration-range-day-minute-window.dmn",
        json!({ "elapsed": "PT2M" }),
        "comma day-time unit range DMN source should parse",
        "comma day-time unit range DMN evaluator should run",
    )
    .await;
    assert_eq!(
        minute_window.output,
        json!({ "window": "minute-window-comma" })
    );
    assert_eq!(
        minute_window.matched_rule_ids[0].as_ref(),
        "rule_comma_minute_window"
    );

    let day_window = evaluate_fixture(
        "comma-day-minute-window",
        "comma-duration-range-day-minute-window.dmn",
        json!({ "elapsed": "P1,5D" }),
        "comma day-time unit range DMN source should parse",
        "comma day-time unit range DMN evaluator should run",
    )
    .await;
    assert_eq!(day_window.output, json!({ "window": "day-window-comma" }));
    assert_eq!(
        day_window.matched_rule_ids[0].as_ref(),
        "rule_comma_day_window"
    );

    let outside = evaluate_fixture(
        "comma-day-minute-window",
        "comma-duration-range-day-minute-window.dmn",
        json!({ "elapsed": "PT10M" }),
        "comma day-time unit range DMN source should parse",
        "comma day-time unit range DMN evaluator should run",
    )
    .await;
    assert_eq!(
        outside.output,
        json!({ "window": "outside-day-minute-window-comma" })
    );
    assert_eq!(
        outside.matched_rule_ids[0].as_ref(),
        "rule_outside_comma_day_minute_window"
    );
}
