//! Spider ingress unit tests.

use std::sync::{Arc, Mutex, PoisonError};

use super::content::build_document_description;
use super::locking::lock_slot_for_hash;
use super::{
    InMemoryContentHashStore, PartialReindexHook, SpiderIngressError, SpiderPagePayload,
    SpiderWendaoBridge, WebAssimilationInput, WebAssimilationSink, canonical_web_uri,
    web_namespace_from_url,
};
use crate::{KnowledgeGraphAssimilationSink, RelationType, SyncEngine};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Default)]
struct RecordingSink {
    payloads: Mutex<Vec<String>>,
}

impl RecordingSink {
    fn payloads(&self) -> Vec<String> {
        self.payloads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl WebAssimilationSink for RecordingSink {
    fn assimilate(&self, input: WebAssimilationInput<'_>) -> Result<(), SpiderIngressError> {
        self.payloads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(input.washed_markdown.to_string());
        Ok(())
    }
}

#[derive(Default)]
struct RecordingReindexHook {
    calls: Mutex<Vec<(String, Vec<String>)>>,
}

impl RecordingReindexHook {
    fn calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl PartialReindexHook for RecordingReindexHook {
    fn trigger_partial_reindex(
        &self,
        namespace: &str,
        changed_uris: &[String],
    ) -> Result<(), SpiderIngressError> {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((namespace.to_string(), changed_uris.to_vec()));
        Ok(())
    }
}

#[test]
fn canonical_web_uri_normalizes_absolute_url_and_namespace() {
    let uri = canonical_web_uri("https://docs.rs/spider/latest/spider/?q=1#frag")
        .unwrap_or_else(|error| panic!("canonical uri should parse: {error}"));
    assert_eq!(
        uri,
        "wendao://web/https://docs.rs/spider/latest/spider/?q=1"
    );
    let namespace = web_namespace_from_url("https://docs.rs/spider/latest/spider/?q=1#frag")
        .unwrap_or_else(|error| panic!("namespace should parse: {error}"));
    assert_eq!(namespace, "docs.rs");
}

#[test]
fn canonical_web_uri_rejects_non_http_scheme() {
    let Err(error) = canonical_web_uri("file:///tmp/index.html") else {
        panic!("must fail");
    };
    assert!(matches!(
        error,
        SpiderIngressError::UnsupportedWebScheme { .. }
    ));
}

#[test]
fn build_document_description_uses_title_and_first_content_line() {
    let description = build_document_description(Some("Guide"), "\n\nAlpha\nBeta");
    assert_eq!(description, "Guide: Alpha");
}

#[test]
fn lock_slot_for_hash_is_bounded_by_segment_count() {
    let slot = lock_slot_for_hash("same-hash", 16);
    assert!(slot < 16);
    assert_eq!(slot, lock_slot_for_hash("same-hash", 16));
}

#[test]
fn spider_bridge_deduplicates_by_content_hash() -> TestResult {
    let sink = Arc::new(RecordingSink::default());
    let reindex = Arc::new(RecordingReindexHook::default());
    let bridge = SpiderWendaoBridge::new(
        Arc::new(InMemoryContentHashStore::new()),
        sink.clone(),
        reindex.clone(),
    );

    let payload_a = SpiderPagePayload::new("https://example.com/a", 0, Arc::<str>::from("same"));
    let payload_b = SpiderPagePayload::new("https://example.com/b", 0, Arc::<str>::from("same"));

    let first = bridge
        .ingest_page(&payload_a)
        .map_err(std::io::Error::other)?;
    let second = bridge
        .ingest_page(&payload_b)
        .map_err(std::io::Error::other)?;

    assert!(first.is_some());
    assert!(second.is_none());
    assert_eq!(sink.payloads().len(), 1);
    assert_eq!(reindex.calls().len(), 1);
    Ok(())
}

#[test]
fn spider_bridge_washes_content_and_triggers_partial_reindex() -> TestResult {
    let sink = Arc::new(RecordingSink::default());
    let reindex = Arc::new(RecordingReindexHook::default());
    let bridge = SpiderWendaoBridge::new(
        Arc::new(InMemoryContentHashStore::new()),
        sink.clone(),
        reindex.clone(),
    );

    let payload = SpiderPagePayload::new(
        "https://example.com/docs",
        2,
        Arc::<str>::from("line-1\r\n\r\n\r\nline-2"),
    );
    let signal = bridge
        .ingest_page(&payload)
        .map_err(std::io::Error::other)?
        .ok_or_else(|| std::io::Error::other("ingestion should not dedup"))?;

    assert_eq!(
        signal.content_hash,
        SyncEngine::compute_hash(payload.markdown_content.as_ref())
    );
    let washed = sink.payloads();
    assert_eq!(washed.len(), 1);
    assert!(!washed[0].contains('\r'));
    assert_eq!(washed[0], "line-1\n\n\nline-2");

    let reindex_calls = reindex.calls();
    assert_eq!(reindex_calls.len(), 1);
    assert_eq!(reindex_calls[0].0, "example.com");
    assert_eq!(
        reindex_calls[0].1[0],
        "wendao://web/https://example.com/docs"
    );
    Ok(())
}

#[test]
fn spider_bridge_for_knowledge_graph_persists_document_entity() -> TestResult {
    let sink = Arc::new(KnowledgeGraphAssimilationSink::new(Arc::new(
        crate::KnowledgeGraph::new(),
    )));
    let bridge = SpiderWendaoBridge::new(
        Arc::new(InMemoryContentHashStore::new()),
        Arc::clone(&sink) as Arc<dyn WebAssimilationSink>,
        Arc::new(crate::NoopPartialReindexHook),
    );
    let payload = SpiderPagePayload::new(
        "https://example.com/guide",
        1,
        Arc::<str>::from("# Guide\n\ncontent"),
    )
    .with_title("Guide");

    let _signal = bridge
        .ingest_page(&payload)
        .map_err(std::io::Error::other)?
        .ok_or_else(|| std::io::Error::other("ingestion should not dedup"))?;

    let graph = sink.graph();
    let canonical =
        canonical_web_uri("https://example.com/guide").map_err(std::io::Error::other)?;
    let entity = graph
        .get_entity_by_name(canonical.as_str())
        .ok_or_else(|| std::io::Error::other("document entity should exist"))?;
    assert_eq!(
        entity.metadata.get("web.title"),
        Some(&serde_json::Value::String("Guide".to_string()))
    );

    let relations = graph.get_relations(Some(canonical.as_str()), Some(RelationType::Contains));
    assert_eq!(relations.len(), 1);
    Ok(())
}
