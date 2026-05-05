use serde_json::Value;

pub(crate) fn assert_related_verbose_monitor(
    payload: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let phases = payload
        .get("phases")
        .and_then(Value::as_array)
        .ok_or("missing monitor phases")?;
    for phase in [
        "link_graph.related.ppr",
        "link_graph.related.subgraph.partition",
        "link_graph.related.subgraph.fusion",
        "link_graph.overlay.promoted",
    ] {
        assert!(
            phases
                .iter()
                .any(|row| row.get("phase").and_then(Value::as_str) == Some(phase)),
            "missing monitor phase: {phase}"
        );
    }
    assert!(
        payload
            .get("monitor")
            .and_then(|row| row.get("bottlenecks"))
            .and_then(|row| row.get("slowest_phase"))
            .is_some()
    );

    let promoted_overlay = payload
        .get("promoted_overlay")
        .ok_or("missing promoted_overlay payload")?;
    assert!(
        promoted_overlay
            .get("applied")
            .and_then(Value::as_bool)
            .is_some()
    );
    assert_eq!(
        promoted_overlay.get("source").and_then(Value::as_str),
        Some("valkey.suggested_link_recent_latest")
    );
    Ok(())
}
