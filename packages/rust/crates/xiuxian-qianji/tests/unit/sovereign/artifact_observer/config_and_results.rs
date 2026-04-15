use super::*;

#[test]
fn config_default_values() {
    let config = ArtifactObserverConfig::default();
    assert!(config.enabled);
    assert_eq!(config.trace_base_path, ".cognitive/traces");
    assert!(config.ingest_on_exit);
    assert!(config.ingest_on_early_halt);
}

#[test]
fn config_clone_preserves_values() {
    let config = ArtifactObserverConfig {
        enabled: false,
        trace_base_path: "custom/path".to_string(),
        ingest_on_exit: false,
        ingest_on_early_halt: true,
    };
    let cloned = config.clone();
    assert!(!cloned.enabled);
    assert_eq!(cloned.trace_base_path, "custom/path");
    assert!(!cloned.ingest_on_exit);
    assert!(cloned.ingest_on_early_halt);
}

#[test]
fn ingestion_result_ingested() {
    let result = ArtifactIngestionResult::Ingested {
        trace_id: "trace-123".to_string(),
        anchor_id: "anchor-456".to_string(),
    };
    match result {
        ArtifactIngestionResult::Ingested {
            trace_id,
            anchor_id,
        } => {
            assert_eq!(trace_id, "trace-123");
            assert_eq!(anchor_id, "anchor-456");
        }
        _ => panic!("expected Ingested variant"),
    }
}

#[test]
fn ingestion_result_no_artifact() {
    let result = ArtifactIngestionResult::NoArtifact;
    assert!(matches!(result, ArtifactIngestionResult::NoArtifact));
}

#[test]
fn ingestion_result_skipped() {
    let result = ArtifactIngestionResult::Skipped {
        reason: Arc::from("test skip"),
    };
    match result {
        ArtifactIngestionResult::Skipped { reason } => {
            assert_eq!(reason.as_ref(), "test skip");
        }
        _ => panic!("expected Skipped variant"),
    }
}

#[test]
fn ingestion_result_failed() {
    let result = ArtifactIngestionResult::Failed {
        error: Arc::from("test error"),
    };
    match result {
        ArtifactIngestionResult::Failed { error } => {
            assert_eq!(error.as_ref(), "test error");
        }
        _ => panic!("expected Failed variant"),
    }
}

#[test]
fn ingestion_result_clone() {
    let result = ArtifactIngestionResult::Ingested {
        trace_id: "trace-789".to_string(),
        anchor_id: "anchor-012".to_string(),
    };
    let cloned = result.clone();
    assert_eq!(result, cloned);
}

#[test]
fn ingestion_result_partial_eq() {
    let result1 = ArtifactIngestionResult::Ingested {
        trace_id: "trace-1".to_string(),
        anchor_id: "anchor-1".to_string(),
    };
    let result2 = ArtifactIngestionResult::Ingested {
        trace_id: "trace-1".to_string(),
        anchor_id: "anchor-1".to_string(),
    };
    let result3 = ArtifactIngestionResult::Ingested {
        trace_id: "trace-2".to_string(),
        anchor_id: "anchor-1".to_string(),
    };
    assert_eq!(result1, result2);
    assert_ne!(result1, result3);
}

#[tokio::test]
async fn noop_sink_returns_ok() {
    let sink = NoopWendaoIngestionSink;
    let trace = CognitiveTraceRecord::new(
        "trace-test".to_string(),
        None,
        "TestNode".to_string(),
        "Test intent".to_string(),
    );
    let doc = trace.to_semantic_document("doc-1", "path.md");
    let result = sink.ingest_trace(&trace, &doc).await;
    assert_eq!(result.as_deref(), Ok("noop:trace-test"));
}
