use serde_json::Value;

pub(crate) fn assert_related_verbose_diagnostics(
    payload: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let diagnostics = payload
        .get("diagnostics")
        .ok_or("missing diagnostics payload")?;

    assert_eq!(diagnostics.get("alpha").and_then(Value::as_f64), Some(0.9));
    assert_eq!(
        diagnostics.get("max_iter").and_then(Value::as_u64),
        Some(64)
    );
    assert_eq!(diagnostics.get("tol").and_then(Value::as_f64), Some(1e-6));
    assert_eq!(
        diagnostics.get("subgraph_count").and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        diagnostics
            .get("partition_max_node_count")
            .and_then(Value::as_u64),
        Some(8)
    );
    assert_eq!(
        diagnostics
            .get("partition_min_node_count")
            .and_then(Value::as_u64),
        Some(8)
    );
    assert_eq!(
        diagnostics
            .get("partition_avg_node_count")
            .and_then(Value::as_f64),
        Some(8.0)
    );
    assert_eq!(
        diagnostics.get("subgraph_mode").and_then(Value::as_str),
        Some("force")
    );
    assert_eq!(
        diagnostics
            .get("horizon_restricted")
            .and_then(Value::as_bool),
        Some(true)
    );

    for key in [
        "iteration_count",
        "candidate_count",
        "candidate_cap",
        "graph_node_count",
    ] {
        assert!(
            diagnostics.get(key).and_then(Value::as_u64).is_some(),
            "missing numeric diagnostics field: {key}"
        );
    }
    for key in [
        "final_residual",
        "total_duration_ms",
        "partition_duration_ms",
        "kernel_duration_ms",
        "fusion_duration_ms",
        "time_budget_ms",
    ] {
        assert!(
            diagnostics.get(key).and_then(Value::as_f64).is_some(),
            "missing float diagnostics field: {key}"
        );
    }
    for key in ["candidate_capped", "timed_out"] {
        assert!(
            diagnostics.get(key).and_then(Value::as_bool).is_some(),
            "missing bool diagnostics field: {key}"
        );
    }
    Ok(())
}
