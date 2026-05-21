//! Integration tests for LinkGraph quantum-context snapshots.

use redis::Connection;
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_wendao::{
    QuantumAnchorHit, QuantumContext, QuantumContextSnapshot, QuantumFusionOptions,
    quantum_context_snapshot_id, valkey_quantum_context_snapshot_get_with_valkey,
    valkey_quantum_context_snapshot_rollback_with_valkey,
    valkey_quantum_context_snapshot_save_with_valkey,
};

const TEST_VALKEY_URL: &str = "redis://127.0.0.1:6379/0";
const QUERY_TEXT: &str = "context snapshot regression query";
const ANCHOR_ID_A: &str = "docs/a#alpha";
const ANCHOR_ID_B: &str = "docs/b#beta";
const DOC_A: &str = "docs/a";
const DOC_B: &str = "docs/b";
const PATH_A: &str = "docs/a.md";
const PATH_B: &str = "docs/b.md";
const SEMANTIC_ROOT: &str = "Root";
const TRACE_LABEL: &str = "[Path: Root]";
const CLUSTER_A: &str = "docs/c";
const CLUSTER_B: &str = "docs/d";
const VECTOR_SCORE_A: f64 = 0.92;
const VECTOR_SCORE_B: f64 = 0.64;
const SALIENCY_SCORE_A: f64 = 0.83;
const SALIENCY_SCORE_B: f64 = 0.71;
const TOPOLOGY_SCORE_A: f64 = 0.48;
const TOPOLOGY_SCORE_B: f64 = 0.22;
const ALPHA: f64 = 0.6;
const MAX_DISTANCE: usize = 2;
const RELATED_LIMIT: usize = 3;

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
    format!("omni:test:context_snapshot:{nanos}")
}

#[test]
fn test_quantum_context_snapshot_roundtrip_with_valkey() -> Result<(), String> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let anchors = vec![
        QuantumAnchorHit {
            anchor_id: ANCHOR_ID_A.to_string(),
            vector_score: VECTOR_SCORE_A,
        },
        QuantumAnchorHit {
            anchor_id: ANCHOR_ID_B.to_string(),
            vector_score: VECTOR_SCORE_B,
        },
    ];
    let options = QuantumFusionOptions {
        alpha: ALPHA,
        max_distance: MAX_DISTANCE,
        related_limit: RELATED_LIMIT,
        ppr: None,
    };
    let contexts = vec![
        QuantumContext {
            anchor_id: ANCHOR_ID_A.to_string(),
            doc_id: DOC_A.to_string().into(),
            path: PATH_A.to_string().into(),
            semantic_path: vec![SEMANTIC_ROOT.to_string()],
            trace_label: Some(TRACE_LABEL.to_string()),
            related_clusters: vec![CLUSTER_A.to_string()],
            saliency_score: SALIENCY_SCORE_A,
            vector_score: VECTOR_SCORE_A,
            topology_score: TOPOLOGY_SCORE_A,
        },
        QuantumContext {
            anchor_id: ANCHOR_ID_B.to_string(),
            doc_id: DOC_B.to_string().into(),
            path: PATH_B.to_string().into(),
            semantic_path: vec![SEMANTIC_ROOT.to_string()],
            trace_label: Some(TRACE_LABEL.to_string()),
            related_clusters: vec![CLUSTER_B.to_string()],
            saliency_score: SALIENCY_SCORE_B,
            vector_score: VECTOR_SCORE_B,
            topology_score: TOPOLOGY_SCORE_B,
        },
    ];

    let snapshot_id = quantum_context_snapshot_id(QUERY_TEXT, &anchors, &options)?;
    let snapshot = QuantumContextSnapshot::new(
        snapshot_id.clone(),
        Some(QUERY_TEXT),
        anchors.clone(),
        options.clone(),
        contexts.clone(),
    );

    valkey_quantum_context_snapshot_save_with_valkey(&snapshot, TEST_VALKEY_URL, Some(&prefix))?;
    let loaded = valkey_quantum_context_snapshot_get_with_valkey(
        snapshot_id.as_str(),
        TEST_VALKEY_URL,
        Some(&prefix),
    )?
    .ok_or_else(|| "missing stored snapshot".to_string())?;

    assert_eq!(loaded, snapshot);

    let rolled_back = valkey_quantum_context_snapshot_rollback_with_valkey(
        snapshot_id.as_str(),
        TEST_VALKEY_URL,
        Some(&prefix),
    )?;
    assert_eq!(rolled_back, Some(contexts));

    clear_prefix(&prefix)?;
    Ok(())
}
