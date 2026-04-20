//! Integration target for bounded depth-2 coactivation diffusion.

use serial_test::serial;
use std::f64::consts::LN_2;
use std::fs;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_wendao::link_graph::{
    LinkGraphSaliencyTouchRequest, set_link_graph_wendao_config_override,
    valkey_saliency_get_with_valkey, valkey_saliency_touch_with_valkey,
};

const TEST_VALKEY_URL: &str = "redis://127.0.0.1:6379/0";

#[test]
#[serial(link_graph_runtime_config)]
fn test_multihop_coactivation_respects_depth_and_total_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let outcome = (|| -> Result<(), Box<dyn std::error::Error>> {
        let _temp = configure_multihop_runtime(&prefix)?;
        seed_graphmem_neighbors(&prefix)?;

        valkey_saliency_touch_with_valkey(
            LinkGraphSaliencyTouchRequest {
                node_id: "note-root".to_string(),
                activation_delta: 1,
                saliency_base: Some(5.0),
                alpha: Some(0.5),
                minimum_saliency: Some(1.0),
                maximum_saliency: Some(10.0),
                now_unix: Some(1_700_000_000),
                ..Default::default()
            },
            TEST_VALKEY_URL,
            Some(&prefix),
        )
        .map_err(std::io::Error::other)?;

        let hop1_hot =
            wait_for_saliency("note-hop1-hot", &prefix)?.ok_or_else(missing_state_error)?;
        let hop1_cold =
            wait_for_saliency("note-hop1-cold", &prefix)?.ok_or_else(missing_state_error)?;
        let hop2_hot =
            wait_for_saliency("note-hop2-hot", &prefix)?.ok_or_else(missing_state_error)?;
        let hop2_cold = current_saliency("note-hop2-cold", &prefix)?;
        let hop2_tail = current_saliency("note-hop2-tail", &prefix)?;

        assert!(hop1_hot > hop1_cold);
        assert!(hop1_hot > hop2_hot);
        assert!(approx_eq(hop1_hot, 5.0 + 0.25 * LN_2, 1e-6));
        assert!(approx_eq(hop1_cold, 5.0 + 0.125 * LN_2, 1e-6));
        assert!(approx_eq(hop2_hot, 5.0 + 0.125 * LN_2, 1e-6));
        assert!(hop2_cold.is_none());
        assert!(hop2_tail.is_none());

        Ok(())
    })();

    let _ = clear_prefix(&prefix);
    outcome
}

fn configure_multihop_runtime(
    prefix: &str,
) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao-test.toml");
    fs::write(
        &config_path,
        format!(
            "[link_graph.cache]\nvalkey_url = \"{TEST_VALKEY_URL}\"\nkey_prefix = \"{prefix}\"\n\n[link_graph.saliency.coactivation]\nenabled = true\nalpha_scale = 0.5\nmax_neighbors_per_direction = 2\nmax_hops = 2\nmax_total_propagations = 3\nhop_decay_scale = 0.5\n"
        ),
    )?;
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);
    Ok(temp)
}

fn seed_graphmem_neighbors(prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = valkey_connection()?;
    let root_outbound_key = format!("{prefix}:kg:edge:out:note-root");
    let hop1_hot_outbound_key = format!("{prefix}:kg:edge:out:note-hop1-hot");
    let hop1_cold_outbound_key = format!("{prefix}:kg:edge:out:note-hop1-cold");
    redis::cmd("ZADD")
        .arg(&root_outbound_key)
        .arg(9.0)
        .arg("note-hop1-hot")
        .arg(7.0)
        .arg("note-hop1-cold")
        .query::<()>(&mut conn)?;
    redis::cmd("ZADD")
        .arg(&hop1_hot_outbound_key)
        .arg(8.0)
        .arg("note-hop2-hot")
        .arg(6.0)
        .arg("note-hop2-tail")
        .query::<()>(&mut conn)?;
    redis::cmd("ZADD")
        .arg(&hop1_cold_outbound_key)
        .arg(7.0)
        .arg("note-hop2-cold")
        .query::<()>(&mut conn)?;
    Ok(())
}

fn current_saliency(
    node_id: &str,
    prefix: &str,
) -> Result<Option<f64>, Box<dyn std::error::Error>> {
    valkey_saliency_get_with_valkey(node_id, TEST_VALKEY_URL, Some(prefix))
        .map(|state| state.map(|row| row.current_saliency))
        .map_err(|err| std::io::Error::other(err).into())
}

fn wait_for_saliency(
    node_id: &str,
    prefix: &str,
) -> Result<Option<f64>, Box<dyn std::error::Error>> {
    for _ in 0..50 {
        if let Some(value) = current_saliency(node_id, prefix)? {
            return Ok(Some(value));
        }
        thread::sleep(Duration::from_millis(10));
    }
    Ok(None)
}

fn valkey_connection() -> Result<redis::Connection, Box<dyn std::error::Error>> {
    let client = redis::Client::open(TEST_VALKEY_URL)?;
    Ok(client.get_connection()?)
}

fn clear_prefix(prefix: &str) -> Result<(), String> {
    let mut conn = valkey_connection().map_err(|err| err.to_string())?;
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
        .map_or(0, |value| value.as_nanos());
    format!("omni:test:coactivation-multihop:{nanos}")
}

fn approx_eq(left: f64, right: f64, tolerance: f64) -> bool {
    (left - right).abs() <= tolerance
}

fn missing_state_error() -> std::io::Error {
    std::io::Error::other("missing propagated saliency state")
}
