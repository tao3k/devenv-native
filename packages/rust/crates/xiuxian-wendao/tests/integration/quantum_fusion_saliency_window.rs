//! Integration target for saliency-aware Quantum Fusion window sizing.

use arrow::array::{Float64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use serial_test::serial;
use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_wendao::link_graph::{
    QuantumAnchorHit, QuantumFusionOptions, set_link_graph_wendao_config_override,
    valkey_saliency_touch_with_valkey,
};
use xiuxian_wendao::{LinkGraphIndex, LinkGraphSaliencyTouchRequest};

const TEST_VALKEY_URL: &str = "redis://127.0.0.1:6379/0";

#[test]
#[serial(link_graph_runtime_config)]
fn test_quantum_fusion_related_window_expands_for_hot_seed()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let outcome = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();

        for (name, content) in [
            ("alpha", "# Alpha\n\n[[beta]]\n[[gamma]]\n"),
            ("beta", "# Beta\n\n[[alpha]]\n"),
            ("gamma", "# Gamma\n\n[[alpha]]\n"),
        ] {
            fs::write(root.join(format!("{name}.md")), content)?;
        }

        let config_path = root.join("wendao-test.toml");
        fs::write(
            &config_path,
            format!(
                "[link_graph.cache]\nvalkey_url = \"{TEST_VALKEY_URL}\"\nkey_prefix = \"{prefix}\"\n"
            ),
        )?;
        let config_path_string = config_path.to_string_lossy().to_string();
        set_link_graph_wendao_config_override(&config_path_string);

        let index = LinkGraphIndex::build(root)?;
        let anchors = vec![QuantumAnchorHit {
            anchor_id: "alpha".to_string(),
            vector_score: 0.9,
        }];
        let options = QuantumFusionOptions {
            alpha: 0.5,
            max_distance: 2,
            related_limit: 1,
            ppr: None,
        };
        let anchor_batch = single_anchor_batch("alpha", 0.9)?;

        let baseline_contexts = index.quantum_contexts_from_anchors(&anchors, &options)?;
        assert_eq!(baseline_contexts.len(), 1);
        assert_eq!(
            baseline_contexts[0].related_clusters,
            vec!["beta".to_string()]
        );
        let baseline_batch_contexts = index.quantum_contexts_from_anchor_batch(
            &anchor_batch,
            "anchor_id",
            "vector_score",
            &options,
        )?;
        assert_eq!(baseline_batch_contexts.len(), 1);
        assert_eq!(
            baseline_batch_contexts[0].related_clusters,
            vec!["beta".to_string()]
        );

        valkey_saliency_touch_with_valkey(
            LinkGraphSaliencyTouchRequest {
                node_id: "alpha".to_string(),
                activation_delta: 64,
                saliency_base: Some(10.0),
                alpha: Some(1.0),
                minimum_saliency: Some(1.0),
                maximum_saliency: Some(10.0),
                now_unix: Some(1_700_000_000),
                ..Default::default()
            },
            TEST_VALKEY_URL,
            Some(&prefix),
        )
        .map_err(std::io::Error::other)?;

        let weighted_contexts = index.quantum_contexts_from_anchors(&anchors, &options)?;
        assert_eq!(weighted_contexts.len(), 1);
        assert_eq!(
            weighted_contexts[0].related_clusters,
            vec!["beta".to_string(), "gamma".to_string()]
        );
        assert!(
            weighted_contexts[0].related_clusters.len()
                > baseline_contexts[0].related_clusters.len()
        );
        let weighted_batch_contexts = index.quantum_contexts_from_anchor_batch(
            &anchor_batch,
            "anchor_id",
            "vector_score",
            &options,
        )?;
        assert_eq!(weighted_batch_contexts.len(), 1);
        assert_eq!(
            weighted_batch_contexts[0].related_clusters,
            vec!["beta".to_string(), "gamma".to_string()]
        );
        assert!(
            weighted_batch_contexts[0].related_clusters.len()
                > baseline_batch_contexts[0].related_clusters.len()
        );

        Ok(())
    })();

    let _ = clear_prefix(&prefix);
    outcome
}

fn unique_prefix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_nanos());
    format!("omni:test:quantum-fusion-window:{nanos}")
}

fn clear_prefix(prefix: &str) -> Result<(), String> {
    let client = redis::Client::open(TEST_VALKEY_URL).map_err(|err| err.to_string())?;
    let mut conn = client.get_connection().map_err(|err| err.to_string())?;
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

fn single_anchor_batch(
    anchor_id: &str,
    vector_score: f64,
) -> Result<RecordBatch, arrow::error::ArrowError> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("anchor_id", DataType::Utf8, false),
        Field::new("vector_score", DataType::Float64, false),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec![anchor_id])),
            Arc::new(Float64Array::from(vec![vector_score])),
        ],
    )
}
