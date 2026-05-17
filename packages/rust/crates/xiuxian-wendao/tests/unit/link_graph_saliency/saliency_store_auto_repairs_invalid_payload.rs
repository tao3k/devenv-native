use super::support::{
    TEST_VALKEY_URL, assert_snapshot_eq, clear_prefix, unique_prefix, valkey_connection,
};
use serde::Serialize;
use xiuxian_wendao::link_graph::{
    LinkGraphSaliencyTouchRequest, valkey_saliency_get_with_valkey,
    valkey_saliency_touch_with_valkey,
};

#[derive(Serialize)]
struct AutoRepairSnapshot {
    fetched_none: bool,
    raw_none: bool,
}

#[test]
fn test_saliency_store_auto_repairs_invalid_payload() -> Result<(), String> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let _ = valkey_saliency_touch_with_valkey(
        LinkGraphSaliencyTouchRequest {
            node_id: "note-b".to_string(),
            activation_delta: 1,
            saliency_base: Some(5.0),
            decay_rate: Some(0.01),
            alpha: None,
            minimum_saliency: None,
            maximum_saliency: None,
            now_unix: Some(1_700_000_000),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
    )
    .map_err(|err| err.to_string())?;

    let mut conn = valkey_connection()?;
    let pattern = format!("{prefix}:saliency:*");
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(&pattern)
        .query(&mut conn)
        .map_err(|err| err.to_string())?;
    if keys.is_empty() {
        clear_prefix(&prefix)?;
        return Ok(());
    }
    let key = keys[0].clone();
    redis::cmd("SET")
        .arg(&key)
        .arg("{\"schema\":\"invalid.schema\"}")
        .query::<()>(&mut conn)
        .map_err(|err| err.to_string())?;

    let fetched = valkey_saliency_get_with_valkey("note-b", TEST_VALKEY_URL, Some(&prefix))
        .map_err(|err| err.to_string())?;
    let raw: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query(&mut conn)
        .map_err(|err| err.to_string())?;

    let snapshot = AutoRepairSnapshot {
        fetched_none: fetched.is_none(),
        raw_none: raw.is_none(),
    };
    let actual = format!(
        "{}\n",
        serde_json::to_string_pretty(&snapshot).map_err(|err| err.to_string())?
    );
    assert_snapshot_eq(
        "link_graph/saliency/saliency_store_auto_repairs_invalid_payload.json",
        actual.as_str(),
    );

    clear_prefix(&prefix)?;
    Ok(())
}
