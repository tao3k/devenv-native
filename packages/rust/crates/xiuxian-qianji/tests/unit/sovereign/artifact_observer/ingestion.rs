use super::{
    ArtifactIngestionResult, ArtifactObserver, ArtifactObserverBuilder, ArtifactObserverConfig,
    CognitiveTraceRecord, LinkGraphSemanticDocument, NoopWendaoIngestionSink, WendaoIngestionSink,
    async_trait,
};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
struct MockIngestionSink {
    call_count: AtomicUsize,
    last_trace_id: std::sync::Mutex<Option<String>>,
}

#[async_trait]
impl WendaoIngestionSink for MockIngestionSink {
    async fn ingest_trace(
        &self,
        trace: &CognitiveTraceRecord,
        _document: &LinkGraphSemanticDocument,
    ) -> Result<String, String> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let mut last = self
            .last_trace_id
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *last = Some(trace.trace_id.clone());
        Ok(format!("mock:{}", trace.trace_id))
    }
}

#[tokio::test]
async fn observer_ingest_artifact_success() {
    let observer = ArtifactObserver::default();
    let trace = CognitiveTraceRecord::new(
        "trace-ingest-1".to_string(),
        Some("session-1".to_string()),
        "AuditNode".to_string(),
        "Critique the agenda".to_string(),
    );
    let result = observer.ingest_artifact(&trace).await;
    match result {
        ArtifactIngestionResult::Ingested {
            trace_id,
            anchor_id,
        } => {
            assert_eq!(trace_id, "trace-ingest-1");
            assert_eq!(anchor_id, "noop:trace-ingest-1");
        }
        _ => panic!("expected Ingested variant, got {result:?}"),
    }
}

#[tokio::test]
async fn observer_ingest_disabled_returns_skipped() {
    let config = ArtifactObserverConfig {
        enabled: false,
        ..Default::default()
    };
    let observer = ArtifactObserver::new(config, NoopWendaoIngestionSink);
    let trace = CognitiveTraceRecord::new(
        "trace-disabled".to_string(),
        None,
        "TestNode".to_string(),
        "Test".to_string(),
    );
    let result = observer.ingest_artifact(&trace).await;
    match result {
        ArtifactIngestionResult::Skipped { reason } => {
            assert_eq!(reason.as_ref(), "ingestion disabled");
        }
        _ => panic!("expected Skipped variant"),
    }
}

#[tokio::test]
async fn observer_ingest_early_halt_skipped_when_disabled() {
    let config = ArtifactObserverConfig {
        ingest_on_early_halt: false,
        ..Default::default()
    };
    let observer = ArtifactObserver::new(config, NoopWendaoIngestionSink);
    let mut trace = CognitiveTraceRecord::new(
        "trace-halt".to_string(),
        None,
        "MonitorNode".to_string(),
        "Monitor".to_string(),
    );
    trace.early_halt_triggered = true;
    let result = observer.ingest_artifact(&trace).await;
    match result {
        ArtifactIngestionResult::Skipped { reason } => {
            assert_eq!(reason.as_ref(), "early halt ingestion disabled");
        }
        _ => panic!("expected Skipped variant"),
    }
}

#[tokio::test]
async fn observer_ingest_early_halt_allowed_when_enabled() {
    let observer = ArtifactObserver::default();
    let mut trace = CognitiveTraceRecord::new(
        "trace-halt-enabled".to_string(),
        None,
        "MonitorNode".to_string(),
        "Monitor".to_string(),
    );
    trace.early_halt_triggered = true;
    let result = observer.ingest_artifact(&trace).await;
    match result {
        ArtifactIngestionResult::Ingested { trace_id, .. } => {
            assert_eq!(trace_id, "trace-halt-enabled");
        }
        _ => panic!("expected Ingested variant"),
    }
}

#[tokio::test]
async fn observer_with_mock_sink() {
    let sink = MockIngestionSink::default();
    let observer = ArtifactObserverBuilder::new().sink(sink).build();

    let trace = CognitiveTraceRecord::new(
        "trace-mock".to_string(),
        None,
        "TestNode".to_string(),
        "Test".to_string(),
    );

    let result = observer.ingest_artifact(&trace).await;
    match result {
        ArtifactIngestionResult::Ingested {
            trace_id,
            anchor_id,
        } => {
            assert_eq!(trace_id, "trace-mock");
            assert_eq!(anchor_id, "mock:trace-mock");
        }
        _ => panic!("expected Ingested variant"),
    }
}

#[tokio::test]
async fn observer_debug_format() {
    let observer = ArtifactObserver::default();
    let debug_str = format!("{observer:?}");
    assert!(debug_str.contains("ArtifactObserver"));
}

#[test]
fn observer_config_access() {
    let config = ArtifactObserverConfig {
        enabled: false,
        trace_base_path: "test/path".to_string(),
        ingest_on_exit: true,
        ingest_on_early_halt: false,
    };
    let observer = ArtifactObserver::new(config.clone(), NoopWendaoIngestionSink);
    let observed_config = observer.config();
    assert!(!observed_config.enabled);
    assert_eq!(observed_config.trace_base_path, "test/path");
    assert!(observed_config.ingest_on_exit);
    assert!(!observed_config.ingest_on_early_halt);
}
