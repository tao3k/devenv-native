use super::support::{TEST_VALKEY_URL, assert_snapshot_eq, clear_prefix, unique_prefix};
use serde::Serialize;
use xiuxian_wendao::link_graph::{
    LinkGraphSaliencyTouchRequest, valkey_saliency_get_with_valkey,
    valkey_saliency_touch_with_valkey,
};

#[derive(Serialize)]
struct SaliencyTouchSnapshot {
    first_activation_count: u64,
    first_current_saliency: String,
    first_saliency_base: String,
    second_activation_count: u64,
    second_current_saliency: String,
    second_saliency_base: String,
    fetched_activation_count: u64,
    fetched_last_accessed_unix: i64,
    fetched_current_saliency: String,
}

#[test]
fn test_saliency_touch_and_get_with_valkey() -> Result<(), String> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let first = valkey_saliency_touch_with_valkey(
        LinkGraphSaliencyTouchRequest {
            node_id: "note-a".to_string(),
            activation_delta: 2,
            saliency_base: Some(5.0),
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

    let second = valkey_saliency_touch_with_valkey(
        LinkGraphSaliencyTouchRequest {
            node_id: "note-a".to_string(),
            activation_delta: 3,
            saliency_base: None,
            decay_rate: None,
            alpha: Some(0.5),
            minimum_saliency: Some(1.0),
            maximum_saliency: Some(10.0),
            now_unix: Some(1_700_086_400),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
    )
    .map_err(|err| err.to_string())?;

    let fetched = valkey_saliency_get_with_valkey("note-a", TEST_VALKEY_URL, Some(&prefix))
        .map_err(|err| err.to_string())?;
    let state = fetched.ok_or_else(|| "missing saliency state after touch".to_string())?;

    let snapshot = SaliencyTouchSnapshot {
        first_activation_count: first.activation_count,
        first_current_saliency: format!("{:.6}", first.current_saliency),
        first_saliency_base: format!("{:.6}", first.saliency_base),
        second_activation_count: second.activation_count,
        second_current_saliency: format!("{:.6}", second.current_saliency),
        second_saliency_base: format!("{:.6}", second.saliency_base),
        fetched_activation_count: state.activation_count,
        fetched_last_accessed_unix: state.last_accessed_unix,
        fetched_current_saliency: format!("{:.6}", state.current_saliency),
    };
    let actual = format!(
        "{}\n",
        serde_json::to_string_pretty(&snapshot).map_err(|err| err.to_string())?
    );
    assert_snapshot_eq(
        "link_graph/saliency/saliency_touch_and_get_with_valkey.json",
        actual.as_str(),
    );

    clear_prefix(&prefix)?;
    Ok(())
}
