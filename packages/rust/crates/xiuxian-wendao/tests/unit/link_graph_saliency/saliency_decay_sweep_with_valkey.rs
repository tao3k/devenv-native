use redis::Connection;
use std::f64::consts::LN_2;
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_wendao::link_graph::{
    LinkGraphSaliencyDecaySweepRequest, LinkGraphSaliencyTouchRequest,
    valkey_saliency_decay_all_with_valkey, valkey_saliency_get_with_valkey,
    valkey_saliency_touch_with_valkey,
};

const TEST_VALKEY_URL: &str = "redis://127.0.0.1:6379/15";

#[test]
fn test_saliency_decay_sweep_with_valkey() -> Result<(), String> {
    let prefix = unique_prefix();
    let invalid_key = format!("{prefix}:saliency:invalid");
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let stale_time = 1_700_000_000;
    let fresh_time = stale_time + 86_400;
    seed_touch("note-stale", &prefix, stale_time)?;
    seed_touch("note-fresh", &prefix, fresh_time)?;
    seed_invalid_payload(&invalid_key)?;

    let result = valkey_saliency_decay_all_with_valkey(
        LinkGraphSaliencyDecaySweepRequest {
            now_unix: Some(fresh_time),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
    )?;

    let stale_state =
        valkey_saliency_get_with_valkey("note-stale", TEST_VALKEY_URL, Some(&prefix))?
            .ok_or_else(|| "missing stale saliency state".to_string())?;
    let fresh_state =
        valkey_saliency_get_with_valkey("note-fresh", TEST_VALKEY_URL, Some(&prefix))?
            .ok_or_else(|| "missing fresh saliency state".to_string())?;

    let initial_score = 5.0 + 0.5 * LN_2;
    let expected_stale = initial_score * (-0.05f64).exp();
    let invalid_payload: Option<String> = redis::cmd("GET")
        .arg(&invalid_key)
        .query(&mut valkey_connection()?)
        .map_err(|err| err.to_string())?;

    assert_eq!(result.now_unix, fresh_time);
    assert_eq!(result.scanned_keys, 3);
    assert_eq!(result.updated_states, 1);
    assert_eq!(result.deleted_states, 1);
    assert!(approx_eq(
        stale_state.current_saliency,
        expected_stale,
        1e-6
    ));
    assert_eq!(stale_state.last_accessed_unix, fresh_time);
    assert!(approx_eq(fresh_state.current_saliency, initial_score, 1e-6));
    assert_eq!(fresh_state.last_accessed_unix, fresh_time);
    assert!(invalid_payload.is_none());

    clear_prefix(&prefix)?;
    Ok(())
}

fn seed_touch(node_id: &str, prefix: &str, now_unix: i64) -> Result<(), String> {
    valkey_saliency_touch_with_valkey(
        LinkGraphSaliencyTouchRequest {
            node_id: node_id.to_string(),
            activation_delta: 1,
            saliency_base: Some(5.0),
            alpha: Some(0.5),
            minimum_saliency: Some(1.0),
            maximum_saliency: Some(10.0),
            now_unix: Some(now_unix),
            ..Default::default()
        },
        TEST_VALKEY_URL,
        Some(prefix),
    )
    .map(|_| ())
}

fn seed_invalid_payload(invalid_key: &str) -> Result<(), String> {
    redis::cmd("SET")
        .arg(invalid_key)
        .arg(r#"{"schema":"broken"}"#)
        .query::<()>(&mut valkey_connection()?)
        .map_err(|err| err.to_string())
}

fn valkey_connection() -> Result<Connection, String> {
    let client = redis::Client::open(TEST_VALKEY_URL).map_err(|err| err.to_string())?;
    client.get_connection().map_err(|err| err.to_string())
}

fn clear_prefix(prefix: &str) -> Result<(), String> {
    let mut conn = valkey_connection()?;
    let pattern = format!("{prefix}:*");
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(&pattern)
        .query(&mut conn)
        .map_err(|err| err.to_string())?;
    if !keys.is_empty() {
        redis::cmd("DEL")
            .arg(keys)
            .query::<()>(&mut conn)
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn unique_prefix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    format!("omni:test:saliency-decay:{nanos}")
}

fn approx_eq(left: f64, right: f64, tolerance: f64) -> bool {
    (left - right).abs() <= tolerance
}
