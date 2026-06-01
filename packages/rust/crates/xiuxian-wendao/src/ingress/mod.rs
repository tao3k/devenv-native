//! Ingress pipeline adapters for bringing external content into Wendao.

#[path = "spider/mod.rs"]
mod spider;
mod transmuter;

pub use spider::{
    ContentHashStore, InMemoryContentHashStore, KnowledgeGraphAssimilationSink,
    NoopPartialReindexHook, PartialReindexHook, SpiderIngressError, SpiderPagePayload,
    SpiderWendaoBridge, WebAssimilationInput, WebAssimilationSink, WebIngestionSignal,
    canonical_web_uri, web_namespace_from_url,
};
