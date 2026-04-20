use super::*;
use tempfile::TempDir;

#[test]
fn wendao_index_adapter_new_creates_adapter() {
    let adapter = WendaoIndexAdapter::new();
    assert!(adapter.file_sink.base_dir().ends_with(".cognitive/traces"));
}

#[test]
fn wendao_index_adapter_default() {
    let adapter = WendaoIndexAdapter::default();
    assert!(adapter.file_sink.base_dir().ends_with(".cognitive/traces"));
}

#[test]
fn wendao_index_adapter_with_file_sink() {
    let temp_dir = TempDir::new().unwrap();
    let file_sink = FileWendaoSink::new(temp_dir.path());
    let adapter = WendaoIndexAdapter::with_file_sink(file_sink.clone());
    assert_eq!(adapter.file_sink.base_dir(), temp_dir.path());
}

#[test]
fn wendao_index_adapter_builder_file_sink() {
    let temp_dir = TempDir::new().unwrap();
    let file_sink = FileWendaoSink::new(temp_dir.path());
    let adapter = WendaoIndexAdapter::builder().file_sink(file_sink).build();
    assert_eq!(adapter.file_sink.base_dir(), temp_dir.path());
}

#[test]
fn wendao_index_adapter_builder_memory_sink() {
    let temp_dir = TempDir::new().unwrap();
    let adapter = WendaoIndexAdapter::builder()
        .file_sink(FileWendaoSink::new(temp_dir.path()))
        .memory_sink(InMemoryWendaoSink::new())
        .build();

    assert!(adapter.memory_sink.is_empty());
}

#[test]
fn wendao_index_adapter_builder_requires_file_sink() {
    let result = std::panic::catch_unwind(|| {
        let _adapter = WendaoIndexAdapterBuilder::new().build();
    });
    assert!(result.is_err());
}

#[tokio::test]
async fn wendao_index_adapter_ingest_trace() {
    let temp_dir = TempDir::new().unwrap();
    let file_sink = FileWendaoSink::new(temp_dir.path());
    let adapter = WendaoIndexAdapter::with_file_sink(file_sink);

    let trace = CognitiveTraceRecord::new(
        "trace-adapter-test".to_string(),
        None,
        "AdapterNode".to_string(),
        "Test trace".to_string(),
    );

    let doc = trace.to_semantic_document("doc-1", "test.md");
    let result = adapter.ingest_trace(&trace, &doc).await;

    assert!(result.is_ok());
    let anchor_id = result.unwrap();
    assert!(anchor_id.starts_with("file:"));
}

#[tokio::test]
async fn wendao_index_adapter_fallback_to_memory() {
    let file_sink = FileWendaoSink::new_no_create("/nonexistent/path/that/cannot/be/created");
    let memory_sink = InMemoryWendaoSink::new();
    let adapter = WendaoIndexAdapter::with_sinks(file_sink, memory_sink);

    let trace = CognitiveTraceRecord::new(
        "trace-fallback-test".to_string(),
        None,
        "FallbackNode".to_string(),
        "Test fallback".to_string(),
    );

    let doc = trace.to_semantic_document("doc-2", "test.md");
    let result = adapter.ingest_trace(&trace, &doc).await;

    assert!(result.is_ok());
    let anchor_id = result.unwrap();
    assert!(anchor_id.starts_with("memory:"));
}

#[test]
fn wendao_index_adapter_debug_format() {
    let adapter = WendaoIndexAdapter::new();
    let debug_str = format!("{:?}", adapter);
    assert!(debug_str.contains("WendaoIndexAdapter"));
}
