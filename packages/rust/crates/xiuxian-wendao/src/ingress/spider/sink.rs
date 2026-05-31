//! `ingress::spider::sink` owns Wendao ingress spider sink behavior.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use super::content::build_document_description;
use super::errors::SpiderIngressError;
use super::types::WebIngestionSignal;
use crate::{Entity, EntityType, KnowledgeGraph, Relation, RelationType};

/// Destination sink for washed web content.
pub trait WebAssimilationSink: Send + Sync {
    /// Persist one washed web payload into Wendao domain storage.
    ///
    /// # Errors
    ///
    /// Returns [`SpiderIngressError`] when persistence fails.
    fn assimilate(&self, input: WebAssimilationInput<'_>) -> Result<(), SpiderIngressError>;
}

/// Washed web payload ready for sink assimilation.
pub struct WebAssimilationInput<'a> {
    /// Canonical Wendao URI assigned to the washed document.
    pub canonical_uri: &'a str,
    /// Markdown content after washer normalization.
    pub washed_markdown: &'a str,
    /// Source ingestion signal that preserves fetch provenance.
    pub signal: &'a WebIngestionSignal,
    /// Optional title extracted from the web payload.
    pub title: Option<&'a str>,
    /// Additional source metadata preserved from the ingestion lane.
    pub metadata: &'a HashMap<String, String>,
}

/// KnowledgeGraph-backed sink for web ingestion.
#[derive(Debug, Clone)]
pub struct KnowledgeGraphAssimilationSink {
    graph: Arc<KnowledgeGraph>,
}

impl KnowledgeGraphAssimilationSink {
    /// Construct sink from existing graph handle.
    #[must_use]
    pub fn new(graph: Arc<KnowledgeGraph>) -> Self {
        Self { graph }
    }

    /// Get a reference to the underlying knowledge graph.
    #[must_use]
    pub fn graph(&self) -> &KnowledgeGraph {
        self.graph.as_ref()
    }
}

impl WebAssimilationSink for KnowledgeGraphAssimilationSink {
    fn assimilate(&self, input: WebAssimilationInput<'_>) -> Result<(), SpiderIngressError> {
        let canonical_uri = input.canonical_uri;
        let washed_markdown = input.washed_markdown;
        let signal = input.signal;
        let title = input.title;
        let metadata = input.metadata;
        let namespace = super::url::web_namespace_from_url(signal.url.as_str())?;
        let cluster_id = format!("web-cluster:{namespace}");
        let cluster_name = format!("web://{namespace}");

        let cluster = Entity::new(
            cluster_id,
            cluster_name.clone(),
            EntityType::Concept,
            format!("Web namespace cluster for {namespace}"),
        )
        .with_source(Some("wendao://web".to_string()));
        self.graph
            .add_entity(cluster)
            .map_err(|error| SpiderIngressError::AssimilationFailed {
                uri: canonical_uri.to_string(),
                reason: error.to_string(),
            })?;

        let mut document = Entity::new(
            canonical_uri.to_string(),
            canonical_uri.to_string(),
            EntityType::Document,
            build_document_description(title, washed_markdown),
        )
        .with_source(Some(signal.url.clone()));
        document.metadata.insert(
            "wendao.uri".to_string(),
            Value::String(canonical_uri.to_string()),
        );
        document
            .metadata
            .insert("web.url".to_string(), Value::String(signal.url.clone()));
        document.metadata.insert(
            "web.content_hash".to_string(),
            Value::String(signal.content_hash.clone()),
        );
        document.metadata.insert(
            "web.depth".to_string(),
            Value::from(u64::from(signal.depth)),
        );
        if let Some(title) = title {
            document
                .metadata
                .insert("web.title".to_string(), Value::String(title.to_string()));
        }
        for (key, value) in metadata {
            document
                .metadata
                .insert(format!("web.meta.{key}"), Value::String(value.clone()));
        }
        self.graph.add_entity(document).map_err(|error| {
            SpiderIngressError::AssimilationFailed {
                uri: canonical_uri.to_string(),
                reason: error.to_string(),
            }
        })?;

        let relation = Relation::new(
            cluster_name,
            canonical_uri.to_string(),
            RelationType::Contains,
            "Web namespace contains crawled document".to_string(),
        )
        .with_source_doc(Some(canonical_uri.to_string()));
        self.graph.add_relation(relation).map_err(|error| {
            SpiderIngressError::AssimilationFailed {
                uri: canonical_uri.to_string(),
                reason: error.to_string(),
            }
        })?;

        Ok(())
    }
}
