//! Artifact Observer for Sovereign Memory (Blueprint V6.1).
//!
//! Observes workflow completion events and triggers final ingestion of
//! `CognitiveTrace` records into Wendao for persistent historical sovereignty.
//!
//! ## Architecture
//!
//! ```text
//! NodeTransition (Exiting) ──► ArtifactObserver ──► Wendao Ingestion
//!         │                         │                    │
//!         │                         ▼                    ▼
//!         └───���────────────► ThoughtAggregator    CognitiveTraceRecord
//!                                   .build()        persisted to LinkGraph
//! ```

use crate::telemetry::{NodeTransitionPhase, SwarmEvent};
use async_trait::async_trait;
use std::sync::Arc;
use xiuxian_wendao_core::{CognitiveTraceRecord, LinkGraphSemanticDocument};

/// Result of artifact observation and ingestion.
#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactIngestionResult {
    /// Artifact was successfully ingested into Wendao.
    Ingested {
        /// The trace ID that was ingested.
        trace_id: String,
        /// The anchor ID in Wendao.
        anchor_id: String,
    },
    /// No artifact was available to ingest.
    NoArtifact,
    /// Ingestion was skipped due to configuration.
    Skipped {
        /// Reason for skipping.
        reason: Arc<str>,
    },
    /// Ingestion failed.
    Failed {
        /// Error message.
        error: Arc<str>,
    },
}

/// Configuration for the artifact observer.
#[derive(Debug, Clone)]
pub struct ArtifactObserverConfig {
    /// Whether ingestion is enabled.
    pub enabled: bool,
    /// Base path for cognitive trace documents in Wendao.
    pub trace_base_path: String,
    /// Whether to ingest on node exit (workflow completion).
    pub ingest_on_exit: bool,
    /// Whether to ingest on early halt.
    pub ingest_on_early_halt: bool,
}

impl Default for ArtifactObserverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            trace_base_path: ".cognitive/traces".to_string(),
            ingest_on_exit: true,
            ingest_on_early_halt: true,
        }
    }
}

/// Sink trait for Wendao ingestion of cognitive traces.
#[async_trait]
pub trait WendaoIngestionSink: Send + Sync + std::fmt::Debug {
    /// Ingest a cognitive trace into Wendao.
    ///
    /// # Errors
    ///
    /// Returns an error string if ingestion fails.
    async fn ingest_trace(
        &self,
        trace: &CognitiveTraceRecord,
        document: &LinkGraphSemanticDocument,
    ) -> Result<String, String>;
}

/// No-op sink used when Wendao ingestion is disabled.
#[derive(Debug, Default)]
pub struct NoopWendaoIngestionSink;

#[async_trait]
impl WendaoIngestionSink for NoopWendaoIngestionSink {
    async fn ingest_trace(
        &self,
        trace: &CognitiveTraceRecord,
        _document: &LinkGraphSemanticDocument,
    ) -> Result<String, String> {
        Ok(format!("noop:{}", trace.trace_id))
    }
}

/// Observer for workflow artifacts that triggers Wendao ingestion.
///
/// This observer listens for workflow completion events and triggers
/// the ingestion of cognitive traces into Wendao for persistent storage.
#[derive(Debug)]
pub struct ArtifactObserver<S: WendaoIngestionSink = NoopWendaoIngestionSink> {
    /// Configuration for the observer.
    config: ArtifactObserverConfig,
    /// Sink for Wendao ingestion.
    sink: S,
}

impl Default for ArtifactObserver<NoopWendaoIngestionSink> {
    fn default() -> Self {
        Self::new(ArtifactObserverConfig::default(), NoopWendaoIngestionSink)
    }
}

impl<S: WendaoIngestionSink> ArtifactObserver<S> {
    /// Create a new artifact observer with the given configuration and sink.
    #[must_use]
    pub fn new(config: ArtifactObserverConfig, sink: S) -> Self {
        Self { config, sink }
    }

    /// Check if this observer should handle the given swarm event.
    #[must_use]
    pub fn should_handle_event(&self, event: &SwarmEvent) -> bool {
        if !self.config.enabled {
            return false;
        }

        match event {
            SwarmEvent::NodeTransition { phase, .. } => match phase {
                NodeTransitionPhase::Exiting | NodeTransitionPhase::Failed => {
                    self.config.ingest_on_exit
                }
                NodeTransitionPhase::Entering => false,
            },
            _ => false,
        }
    }

    /// Ingest a cognitive trace into Wendao.
    ///
    /// This method converts the trace to a semantic document and ingests it.
    pub async fn ingest_artifact(&self, trace: &CognitiveTraceRecord) -> ArtifactIngestionResult {
        if !self.config.enabled {
            return ArtifactIngestionResult::Skipped {
                reason: Arc::from("ingestion disabled"),
            };
        }

        // Check early halt policy
        if trace.early_halt_triggered && !self.config.ingest_on_early_halt {
            return ArtifactIngestionResult::Skipped {
                reason: Arc::from("early halt ingestion disabled"),
            };
        }

        // Build the semantic document for Wendao
        let doc_id = format!("trace:{}", trace.trace_id);
        let path = format!(
            "{}/{}.md",
            self.config.trace_base_path,
            trace.trace_id.replace(':', "-")
        );
        let document = trace.to_semantic_document(&doc_id, &path);

        // Ingest into Wendao
        match self.sink.ingest_trace(trace, &document).await {
            Ok(anchor_id) => ArtifactIngestionResult::Ingested {
                trace_id: trace.trace_id.clone(),
                anchor_id,
            },
            Err(error) => ArtifactIngestionResult::Failed {
                error: Arc::from(error),
            },
        }
    }

    /// Get the observer configuration.
    #[must_use]
    pub const fn config(&self) -> &ArtifactObserverConfig {
        &self.config
    }
}

/// Builder for creating configured artifact observers.
#[derive(Debug, Default)]
pub struct ArtifactObserverBuilder<S: WendaoIngestionSink = NoopWendaoIngestionSink> {
    config: ArtifactObserverConfig,
    sink: Option<S>,
}

impl<S: WendaoIngestionSink> ArtifactObserverBuilder<S> {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ArtifactObserverConfig::default(),
            sink: None,
        }
    }

    /// Set whether ingestion is enabled.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set the base path for cognitive trace documents.
    #[must_use]
    pub fn trace_base_path(mut self, path: impl Into<String>) -> Self {
        self.config.trace_base_path = path.into();
        self
    }

    /// Set whether to ingest on node exit.
    #[must_use]
    pub fn ingest_on_exit(mut self, ingest: bool) -> Self {
        self.config.ingest_on_exit = ingest;
        self
    }

    /// Set whether to ingest on early halt.
    #[must_use]
    pub fn ingest_on_early_halt(mut self, ingest: bool) -> Self {
        self.config.ingest_on_early_halt = ingest;
        self
    }

    /// Set the Wendao ingestion sink.
    #[must_use]
    pub fn sink(mut self, sink: S) -> Self {
        self.sink = Some(sink);
        self
    }

    /// Build the artifact observer.
    ///
    /// # Panics
    ///
    /// Panics if no sink was provided.
    #[must_use]
    pub fn build(self) -> ArtifactObserver<S> {
        let Some(sink) = self.sink else {
            panic!("sink must be provided");
        };
        ArtifactObserver::new(self.config, sink)
    }
}

impl ArtifactObserverBuilder<NoopWendaoIngestionSink> {
    /// Build with the no-op sink.
    #[must_use]
    pub fn build_noop(self) -> ArtifactObserver<NoopWendaoIngestionSink> {
        ArtifactObserver::new(self.config, NoopWendaoIngestionSink)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/sovereign/artifact_observer/mod.rs"]
mod tests;
