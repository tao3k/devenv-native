//! Wendao Adapter for Sovereign Memory (Blueprint V6.1).
//!
//! Provides adapter implementations for the `WendaoIngestionSink` trait,
//! connecting Qianji's `ArtifactObserver` with Wendao's `LinkGraphIndex`
//! for persistent storage.

use async_trait::async_trait;
use xiuxian_wendao_core::{CognitiveTraceRecord, LinkGraphSemanticDocument};

use super::artifact_observer::WendaoIngestionSink;
use super::wendao_sink::{FileWendaoSink, InMemoryWendaoSink};

/// Adapter that bridges Qianji's `ArtifactObserver` with Wendao's `LinkGraphIndex`
/// for persistent storage.
///
/// This adapter provides a composite sink that writes cognitive traces to
/// files (for Wendao to index) with an in-memory fallback for testing.
#[derive(Debug)]
pub struct WendaoIndexAdapter {
    /// The file-based sink for persistent storage.
    file_sink: FileWendaoSink,
    /// The in-memory sink for testing/fallback.
    memory_sink: InMemoryWendaoSink,
}

impl WendaoIndexAdapter {
    /// Create a new adapter with default settings.
    ///
    /// Uses `.cognitive/traces` as the base directory for file storage.
    #[must_use]
    pub fn new() -> Self {
        Self {
            file_sink: FileWendaoSink::new(".cognitive/traces"),
            memory_sink: InMemoryWendaoSink::new(),
        }
    }

    /// Create a new adapter with custom file sink.
    #[must_use]
    pub fn with_file_sink(file_sink: FileWendaoSink) -> Self {
        Self {
            file_sink,
            memory_sink: InMemoryWendaoSink::new(),
        }
    }

    /// Create a new adapter with both sinks configured.
    #[must_use]
    pub fn with_sinks(file_sink: FileWendaoSink, memory_sink: InMemoryWendaoSink) -> Self {
        Self {
            file_sink,
            memory_sink,
        }
    }

    /// Get a builder for constructing a `WendaoIndexAdapter`.
    #[must_use]
    pub fn builder() -> WendaoIndexAdapterBuilder {
        WendaoIndexAdapterBuilder::default()
    }
}

impl Default for WendaoIndexAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WendaoIngestionSink for WendaoIndexAdapter {
    async fn ingest_trace(
        &self,
        trace: &CognitiveTraceRecord,
        document: &LinkGraphSemanticDocument,
    ) -> Result<String, String> {
        // First try the file sink
        match self.file_sink.ingest_trace(trace, document).await {
            Ok(anchor_id) => Ok(anchor_id),
            Err(file_error) => {
                // Fallback to memory sink
                self.memory_sink
                    .ingest_trace(trace, document)
                    .await
                    .map_err(|mem_error| {
                        format!("File sink failed: {file_error}; Memory sink failed: {mem_error}")
                    })
            }
        }
    }
}

/// Builder for constructing a `WendaoIndexAdapter`.
#[derive(Debug, Default)]
pub struct WendaoIndexAdapterBuilder {
    file_sink: Option<FileWendaoSink>,
    memory_sink: Option<InMemoryWendaoSink>,
}

impl WendaoIndexAdapterBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the file-based sink.
    #[must_use]
    pub fn file_sink(mut self, sink: FileWendaoSink) -> Self {
        self.file_sink = Some(sink);
        self
    }

    /// Set the in-memory sink.
    #[must_use]
    pub fn memory_sink(mut self, sink: InMemoryWendaoSink) -> Self {
        self.memory_sink = Some(sink);
        self
    }

    /// Build the adapter.
    ///
    /// # Panics
    ///
    /// Panics if no file sink was configured.
    #[must_use]
    pub fn build(self) -> WendaoIndexAdapter {
        let Some(file_sink) = self.file_sink else {
            panic!("file_sink must be configured");
        };
        let memory_sink = self.memory_sink.unwrap_or_default();
        WendaoIndexAdapter::with_sinks(file_sink, memory_sink)
    }
}

#[cfg(test)]
#[path = "wendao_adapter.rs"]
mod tests;
