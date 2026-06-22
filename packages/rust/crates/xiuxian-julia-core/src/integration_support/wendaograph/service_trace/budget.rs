use serde_json::{Value, json};

pub(super) fn strategy_budget_json(query_understanding: &[Value]) -> Value {
    if query_understanding.is_empty() {
        return json!({
            "source": "default",
            "loopBudget": 1,
            "judgementBudget": 1,
            "beamWidth": 3,
        });
    }
    json!({
        "source": "query_understanding",
        "loopBudget": max_int_field(query_understanding, "recommendedLoopBudget", 1),
        "judgementBudget": max_int_field(query_understanding, "recommendedJudgementBudget", 1),
        "beamWidth": max_int_field(query_understanding, "recommendedBeamWidth", 3),
    })
}

pub(super) fn context_reduction_ratio(total_context: i64, selected_context: i64) -> f64 {
    if total_context <= 0 {
        0.0
    } else {
        1.0 - (non_negative_i64_to_f64(selected_context) / non_negative_i64_to_f64(total_context))
    }
}

fn max_int_field(rows: &[Value], field: &str, default: i64) -> i64 {
    rows.iter()
        .filter_map(|row| row.get(field).and_then(Value::as_i64))
        .max()
        .unwrap_or(default)
}

fn non_negative_i64_to_f64(value: i64) -> f64 {
    let value = u32::try_from(value.max(0)).unwrap_or(u32::MAX);
    f64::from(value)
}
