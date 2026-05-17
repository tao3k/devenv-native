use super::support::{
    TEST_VALKEY_URL, assert_snapshot_eq, clear_prefix, unique_prefix, valkey_connection,
};
use serde::Serialize;
use xiuxian_wendao::link_graph::{
    LinkGraphSaliencyTouchRequest, valkey_saliency_touch_with_valkey,
};

#[derive(Serialize)]
struct InboundEdgeSnapshot {
    current_saliency: String,
    zscore: String,
}

#[test]
fn test_saliency_touch_updates_inbound_edge_zset() -> Result<(), String> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let inbound_key = format!("{prefix}:kg:edge:in:note-b");
    let out_key = format!("{prefix}:kg:edge:out:note-a");
    let mut conn = valkey_connection()?;
    redis::cmd("SADD")
        .arg(&inbound_key)
        .arg("note-a")
        .query::<i64>(&mut conn)
        .map_err(|err| err.to_string())?;

    let state = valkey_saliency_touch_with_valkey(
        LinkGraphSaliencyTouchRequest {
            node_id: "note-b".to_string(),
            activation_delta: 2,
            saliency_base: Some(7.0),
            decay_rate: Some(0.05),
            alpha: Some(0.5),
            minimum_saliency: Some(1.0),
            maximum_saliency: Some(10.0),
            now_unix: Some(1_700_000_000),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
    )
    .map_err(|err| err.to_string())?;

    let zscore: Option<f64> = redis::cmd("ZSCORE")
        .arg(&out_key)
        .arg("note-b")
        .query(&mut conn)
        .map_err(|err| err.to_string())?;
    let score = zscore.ok_or_else(|| "missing zscore for updated edge".to_string())?;

    let snapshot = InboundEdgeSnapshot {
        current_saliency: format!("{:.6}", state.current_saliency),
        zscore: format!("{:.6}", score),
    };
    let actual = format!(
        "{}\n",
        serde_json::to_string_pretty(&snapshot).map_err(|err| err.to_string())?
    );
    assert_snapshot_eq(
        "link_graph/saliency/saliency_touch_updates_inbound_edge_zset.json",
        actual.as_str(),
    );

    clear_prefix(&prefix)?;
    Ok(())
}
