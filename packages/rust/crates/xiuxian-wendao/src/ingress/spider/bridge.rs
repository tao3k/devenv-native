//! `ingress::spider::bridge` owns Wendao ingress spider bridge behavior.

use std::sync::{Arc, Mutex};

use super::errors::SpiderIngressError;
use super::hash_store::{ContentHashStore, InMemoryContentHashStore};
use super::locking::{default_ingest_lock_segments, lock_slot_for_hash};
use super::reindex::NoopPartialReindexHook;
use super::sink::{KnowledgeGraphAssimilationSink, WebAssimilationSink};
use super::types::{SpiderPagePayload, WebIngestionSignal};
use super::url::{canonical_web_uri, web_namespace_from_url};
use crate::KnowledgeGraph;
use crate::sync::SyncEngine;

/// Spider-to-Wendao bridge that enforces URI normalization, deduplication,
/// Zhenfa transmutation, assimilation, and partial re-index hooks.
pub struct SpiderWendaoBridge {
    content_hash_store: Arc<dyn ContentHashStore>,
    sink: Arc<dyn WebAssimilationSink>,
    reindex_hook: Arc<dyn super::reindex::PartialReindexHook>,
    ingest_locks: Vec<Mutex<()>>,
}

impl SpiderWendaoBridge {
    /// Construct a bridge from explicit extension points.
    #[must_use]
    pub fn new(
        content_hash_store: Arc<dyn ContentHashStore>,
        sink: Arc<dyn WebAssimilationSink>,
        reindex_hook: Arc<dyn super::reindex::PartialReindexHook>,
    ) -> Self {
        Self::with_lock_segments(
            content_hash_store,
            sink,
            reindex_hook,
            default_ingest_lock_segments(),
        )
    }

    /// Construct a bridge with explicit lock-striping width for ingestion.
    #[must_use]
    pub fn with_lock_segments(
        content_hash_store: Arc<dyn ContentHashStore>,
        sink: Arc<dyn WebAssimilationSink>,
        reindex_hook: Arc<dyn super::reindex::PartialReindexHook>,
        lock_segments: usize,
    ) -> Self {
        let lock_count = lock_segments.max(1);
        let mut ingest_locks = Vec::with_capacity(lock_count);
        for _ in 0..lock_count {
            ingest_locks.push(Mutex::new(()));
        }
        Self {
            content_hash_store,
            sink,
            reindex_hook,
            ingest_locks,
        }
    }

    /// Construct a bridge with in-memory dedup + `KnowledgeGraph` sink.
    #[must_use]
    pub fn for_knowledge_graph(graph: KnowledgeGraph) -> Self {
        Self::for_shared_knowledge_graph(Arc::new(graph))
    }

    /// Construct a bridge with in-memory dedup + shared `KnowledgeGraph` sink.
    #[must_use]
    pub fn for_shared_knowledge_graph(graph: Arc<KnowledgeGraph>) -> Self {
        Self::new(
            Arc::new(InMemoryContentHashStore::new()),
            Arc::new(KnowledgeGraphAssimilationSink::new(graph)),
            Arc::new(NoopPartialReindexHook),
        )
    }

    /// Assimilate one crawled web page into Wendao.
    ///
    /// Returns `Ok(None)` when payload is deduplicated by `content_hash`.
    ///
    /// # Errors
    ///
    /// Returns [`SpiderIngressError`] when URL normalization, transmutation,
    /// sink persistence, or partial re-index dispatch fails.
    pub fn ingest_page(
        &self,
        payload: &SpiderPagePayload,
    ) -> Result<Option<WebIngestionSignal>, SpiderIngressError> {
        let canonical_uri = canonical_web_uri(payload.url.as_str())?;
        let content_hash = SyncEngine::compute_hash(payload.markdown_content.as_ref());
        let lock_slot = lock_slot_for_hash(content_hash.as_str(), self.ingest_locks.len());

        let guard = self.ingest_locks[lock_slot]
            .lock()
            .map_err(|_| SpiderIngressError::StateLockPoisoned)?;
        if self.content_hash_store.contains_hash(content_hash.as_str()) {
            return Ok(None);
        }

        let washed_markdown = crate::ingress::transmuter::resolve_and_wash(
            canonical_uri.as_str(),
            payload.markdown_content.as_ref(),
        )
        .map_err(|error| SpiderIngressError::TransmutationFailed {
            uri: canonical_uri.clone(),
            reason: error.to_string(),
        })?;
        let signal = WebIngestionSignal {
            url: payload.url.clone(),
            depth: payload.depth,
            content_hash: content_hash.clone(),
        };
        self.sink
            .assimilate(
                canonical_uri.as_str(),
                washed_markdown.as_str(),
                &signal,
                payload.title.as_deref(),
                &payload.metadata,
            )
            .map_err(|error| SpiderIngressError::AssimilationFailed {
                uri: canonical_uri.clone(),
                reason: error.to_string(),
            })?;
        self.content_hash_store.insert_hash(content_hash.as_str());
        drop(guard);

        let namespace = web_namespace_from_url(payload.url.as_str())?;
        self.reindex_hook
            .trigger_partial_reindex(namespace.as_str(), std::slice::from_ref(&canonical_uri))
            .map_err(|error| SpiderIngressError::PartialReindexFailed {
                namespace,
                reason: error.to_string(),
            })?;
        Ok(Some(signal))
    }
}
