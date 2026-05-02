use super::{CompositeWendaoSink, FileWendaoSink, InMemoryWendaoSink, WendaoIngestionSink};
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use xiuxian_wendao_core::CognitiveTraceRecord;

#[test]
fn file_sink_new_creates_base_dir() {
    let sink = FileWendaoSink::new("/tmp/traces");
    assert_eq!(sink.base_dir(), Path::new("/tmp/traces"));
    assert!(sink.create_dir);
}

#[test]
fn file_sink_new_no_create() {
    let sink = FileWendaoSink::new_no_create("/tmp/traces");
    assert!(!sink.create_dir);
}

#[test]
fn file_sink_clone_preserves_settings() {
    let sink = FileWendaoSink::new("/tmp/traces");
    let cloned = sink.clone();
    assert_eq!(sink.base_dir(), cloned.base_dir());
    assert_eq!(sink.create_dir, cloned.create_dir);
}

#[tokio::test]
async fn file_sink_writes_trace_file() {
    let temp_dir = TempDir::new().unwrap_or_else(|err| panic!("failed to create temp dir: {err}"));
    let sink = FileWendaoSink::new(temp_dir.path());

    let trace = CognitiveTraceRecord::new(
        "trace-test-123".to_string(),
        Some("session-abc".to_string()),
        "AuditNode".to_string(),
        "Critique the agenda".to_string(),
    );

    let doc = trace.to_semantic_document("doc-1", "traces/test.md");
    let result = sink.ingest_trace(&trace, &doc).await;

    assert!(matches!(
        result.as_deref(),
        Ok(anchor_id)
            if anchor_id.starts_with("file:") && anchor_id.contains("trace-test-123.md")
    ));
}

#[tokio::test]
async fn file_sink_creates_directory() {
    let temp_dir = TempDir::new().unwrap_or_else(|err| panic!("failed to create temp dir: {err}"));
    let nested_path = temp_dir.path().join("nested").join("dir");
    let sink = FileWendaoSink::new(&nested_path);

    let trace = CognitiveTraceRecord::new(
        "trace-dir-test".to_string(),
        None,
        "TestNode".to_string(),
        "Test".to_string(),
    );

    let doc = trace.to_semantic_document("doc-2", "test.md");
    let result = sink.ingest_trace(&trace, &doc).await;

    assert!(result.is_ok());
    assert!(nested_path.exists());
}

#[tokio::test]
async fn file_sink_produces_valid_markdown() {
    let temp_dir = TempDir::new().unwrap_or_else(|err| panic!("failed to create temp dir: {err}"));
    let sink = FileWendaoSink::new(temp_dir.path());

    let trace = CognitiveTraceRecord::new(
        "trace-md-test".to_string(),
        Some("session-xyz".to_string()),
        "PlanNode".to_string(),
        "Generate a plan".to_string(),
    );

    let doc = trace.to_semantic_document("doc-3", "test.md");
    sink.ingest_trace(&trace, &doc)
        .await
        .unwrap_or_else(|err| panic!("failed to ingest trace: {err}"));

    let file_path = temp_dir.path().join("trace-md-test.md");
    let content = tokio::fs::read_to_string(&file_path)
        .await
        .unwrap_or_else(|err| panic!("failed to read markdown output: {err}"));

    assert!(content.contains("---"));
    assert!(content.contains("trace_id: trace-md-test"));
    assert!(content.contains("session_id: session-xyz"));
    assert!(content.contains("node_id: PlanNode"));
    assert!(content.contains("# Cognitive Trace: PlanNode"));
    assert!(content.contains("## Intent"));
    assert!(content.contains("Generate a plan"));
    assert!(content.contains("## Reasoning"));
}

#[tokio::test]
async fn file_sink_includes_outcome() {
    let temp_dir = TempDir::new().unwrap_or_else(|err| panic!("failed to create temp dir: {err}"));
    let sink = FileWendaoSink::new(temp_dir.path());

    let mut trace = CognitiveTraceRecord::new(
        "trace-outcome".to_string(),
        None,
        "TestNode".to_string(),
        "Test".to_string(),
    );
    trace.outcome = Some(Arc::<str>::from("Task completed successfully"));

    let doc = trace.to_semantic_document("doc-4", "test.md");
    sink.ingest_trace(&trace, &doc)
        .await
        .unwrap_or_else(|err| panic!("failed to ingest trace: {err}"));

    let file_path = temp_dir.path().join("trace-outcome.md");
    let content = tokio::fs::read_to_string(&file_path)
        .await
        .unwrap_or_else(|err| panic!("failed to read markdown output: {err}"));

    assert!(content.contains("## Outcome"));
    assert!(content.contains("Task completed successfully"));
}

#[tokio::test]
async fn file_sink_includes_coherence_score() {
    let temp_dir = TempDir::new().unwrap_or_else(|err| panic!("failed to create temp dir: {err}"));
    let sink = FileWendaoSink::new(temp_dir.path());

    let mut trace = CognitiveTraceRecord::new(
        "trace-coherence".to_string(),
        None,
        "MonitorNode".to_string(),
        "Monitor".to_string(),
    );
    trace.coherence_score = Some(0.85);
    trace.early_halt_triggered = true;

    let doc = trace.to_semantic_document("doc-5", "test.md");
    sink.ingest_trace(&trace, &doc)
        .await
        .unwrap_or_else(|err| panic!("failed to ingest trace: {err}"));

    let file_path = temp_dir.path().join("trace-coherence.md");
    let content = tokio::fs::read_to_string(&file_path)
        .await
        .unwrap_or_else(|err| panic!("failed to read markdown output: {err}"));

    assert!(content.contains("coherence_score: 0.85"));
    assert!(content.contains("early_halt_triggered: true"));
}

#[test]
fn memory_sink_new_creates_empty() {
    let sink = InMemoryWendaoSink::new();
    assert!(sink.is_empty());
    assert_eq!(sink.len(), 0);
}

#[tokio::test]
async fn memory_sink_stores_trace() {
    let sink = InMemoryWendaoSink::new();

    let trace = CognitiveTraceRecord::new(
        "trace-mem-1".to_string(),
        None,
        "TestNode".to_string(),
        "Test".to_string(),
    );

    let doc = trace.to_semantic_document("doc-m1", "test.md");
    let result = sink.ingest_trace(&trace, &doc).await;

    assert_eq!(result.as_deref(), Ok("memory:trace-mem-1"));
    assert_eq!(sink.len(), 1);
}

#[tokio::test]
async fn memory_sink_multiple_traces() {
    let sink = InMemoryWendaoSink::new();

    for i in 0..3 {
        let trace = CognitiveTraceRecord::new(
            format!("trace-mem-{i}"),
            None,
            "TestNode".to_string(),
            "Test".to_string(),
        );
        let doc = trace.to_semantic_document(&format!("doc-{i}"), "test.md");
        sink.ingest_trace(&trace, &doc)
            .await
            .unwrap_or_else(|err| panic!("failed to ingest trace: {err}"));
    }

    assert_eq!(sink.len(), 3);

    let traces = sink.traces();
    assert_eq!(traces.len(), 3);
}

#[tokio::test]
async fn memory_sink_clear() {
    let sink = InMemoryWendaoSink::new();

    let trace = CognitiveTraceRecord::new(
        "trace-clear".to_string(),
        None,
        "TestNode".to_string(),
        "Test".to_string(),
    );
    let doc = trace.to_semantic_document("doc-c", "test.md");
    sink.ingest_trace(&trace, &doc)
        .await
        .unwrap_or_else(|err| panic!("failed to ingest trace: {err}"));

    assert_eq!(sink.len(), 1);

    sink.clear();
    assert!(sink.is_empty());
}

#[tokio::test]
async fn composite_sink_uses_primary() {
    let primary = Arc::new(InMemoryWendaoSink::new());
    let fallback = Arc::new(InMemoryWendaoSink::new());

    let sink = CompositeWendaoSink::builder()
        .primary(primary.clone())
        .fallback(fallback.clone())
        .build();

    let trace = CognitiveTraceRecord::new(
        "trace-comp-1".to_string(),
        None,
        "TestNode".to_string(),
        "Test".to_string(),
    );
    let doc = trace.to_semantic_document("doc-c1", "test.md");

    let result = sink.ingest_trace(&trace, &doc).await;
    assert!(result.is_ok());
    assert_eq!(primary.len(), 1);
    assert_eq!(fallback.len(), 0);
}

#[test]
fn composite_sink_builder_requires_primary() {
    let result = std::panic::catch_unwind(|| {
        let _sink = CompositeWendaoSink::builder().build();
    });
    assert!(result.is_err());
}

#[test]
fn render_markdown_minimal() {
    let trace = CognitiveTraceRecord::new(
        "trace-render-min".to_string(),
        None,
        "TestNode".to_string(),
        "Test intent".to_string(),
    );

    let md = FileWendaoSink::render_markdown(&trace);

    assert!(md.contains("trace_id: trace-render-min"));
    assert!(md.contains("node_id: TestNode"));
    assert!(!md.contains("session_id:"));
    assert!(!md.contains("coherence_score:"));
}

#[test]
fn render_markdown_full() {
    let mut trace = CognitiveTraceRecord::new(
        "trace-render-full".to_string(),
        Some("session-123".to_string()),
        "FullNode".to_string(),
        "Full test".to_string(),
    );
    trace.coherence_score = Some(0.92);
    trace.early_halt_triggered = false;
    trace.commit_sha = Some("abc123def".to_string());
    trace.outcome = Some(Arc::<str>::from("Success"));

    let md = FileWendaoSink::render_markdown(&trace);

    assert!(md.contains("session_id: session-123"));
    assert!(md.contains("coherence_score: 0.92"));
    assert!(md.contains("commit_sha: abc123def"));
    assert!(md.contains("## Outcome"));
    assert!(md.contains("Success"));
}

#[test]
fn render_markdown_early_halt() {
    let mut trace = CognitiveTraceRecord::new(
        "trace-halt".to_string(),
        None,
        "HaltNode".to_string(),
        "Test".to_string(),
    );
    trace.early_halt_triggered = true;
    trace.coherence_score = Some(0.15);

    let md = FileWendaoSink::render_markdown(&trace);

    assert!(md.contains("early_halt_triggered: true"));
    assert!(md.contains("coherence_score: 0.15"));
}
