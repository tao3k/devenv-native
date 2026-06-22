//! `zhenfa_router::native::sentinel::observations` owns Wendao native sentinel observations behavior.

use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use super::types::{DriftConfidence, SemanticDriftSignal};

/// Signal types for observation lifecycle events.
///
/// These signals are emitted when code observations need attention:
/// - `Stale`: The observed code may have changed, observation needs re-validation
/// - `Broken`: The observed code structure no longer matches the pattern
/// - `Orphaned`: The source file referenced by the observation no longer exists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservationSignal {
    /// Observation pattern may be outdated due to source changes.
    Stale {
        /// Document containing the observation.
        doc_id: String,
        /// The observation pattern that may need updating.
        observation: ObservationRef,
        /// Source file that triggered the stale signal.
        trigger_source: String,
        /// Confidence that this observation is affected.
        confidence: DriftConfidence,
    },
    /// Observation pattern no longer matches any code structure.
    Broken {
        /// Document containing the broken observation.
        doc_id: String,
        /// The broken observation pattern.
        observation: ObservationRef,
        /// Error message describing the breakage.
        error: String,
    },
    /// Source file referenced by observation no longer exists.
    Orphaned {
        /// Document containing the orphaned observation.
        doc_id: String,
        /// The orphaned observation pattern.
        observation: ObservationRef,
        /// Former source file location.
        former_source: String,
    },
}

/// Reference to a code observation within a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationRef {
    /// The observation pattern used by the configured source-query backend.
    pub pattern: String,
    /// Target language.
    pub language: String,
    /// Line number in the document.
    pub line_number: usize,
    /// Node ID where the observation is located.
    pub node_id: String,
}

impl ObservationSignal {
    /// Create a stale signal from a semantic drift detection.
    #[must_use]
    pub fn stale_from_drift(drift: &SemanticDriftSignal) -> Vec<Self> {
        drift
            .affected_docs
            .iter()
            .map(|doc| Self::Stale {
                doc_id: doc.doc_id.clone(),
                observation: ObservationRef {
                    pattern: doc.matching_pattern.clone(),
                    language: doc.language.clone(),
                    line_number: doc.line_number.unwrap_or(0),
                    node_id: doc.node_id.clone(),
                },
                trigger_source: drift.source_path.clone(),
                confidence: drift.confidence,
            })
            .collect()
    }

    /// Convert signal to a streaming-friendly status message.
    #[must_use]
    pub fn to_status_message(&self) -> String {
        match self {
            Self::Stale {
                doc_id,
                observation,
                trigger_source,
                confidence,
            } => {
                format!(
                    "⚠️ Stale observation in {}: '{}' may need update (triggered by {}, {:?} confidence)",
                    doc_id, observation.pattern, trigger_source, confidence
                )
            }
            Self::Broken {
                doc_id,
                observation,
                error,
            } => {
                format!(
                    "❌ Broken observation in {}: '{}' - {}",
                    doc_id, observation.pattern, error
                )
            }
            Self::Orphaned {
                doc_id,
                observation,
                former_source,
            } => {
                format!(
                    "🗑️ Orphaned observation in {}: '{}' (source {} no longer exists)",
                    doc_id, observation.pattern, former_source
                )
            }
        }
    }

    /// Get the affected document ID.
    #[must_use]
    pub fn doc_id(&self) -> &str {
        match self {
            Self::Stale { doc_id, .. }
            | Self::Broken { doc_id, .. }
            | Self::Orphaned { doc_id, .. } => doc_id,
        }
    }

    /// Check if this signal requires immediate attention.
    #[must_use]
    pub fn requires_attention(&self) -> bool {
        matches!(
            self,
            Self::Broken { .. }
                | Self::Stale {
                    confidence: DriftConfidence::High,
                    ..
                }
        )
    }
}

/// Global signal counter for unique IDs.
static SIGNAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Streaming bus for observation signals.
///
/// This struct manages the flow of observation signals from Sentinel
/// to agent consumers via an MPSC channel.
pub struct ObservationBus {
    /// Sender for observation signals.
    tx: Option<mpsc::UnboundedSender<ObservationSignal>>,
}

impl Default for ObservationBus {
    fn default() -> Self {
        Self::new()
    }
}

impl ObservationBus {
    /// Create a new observation bus.
    #[must_use]
    pub fn new() -> Self {
        Self { tx: None }
    }

    /// Connect the bus to a receiver channel.
    pub fn connect(&mut self, tx: mpsc::UnboundedSender<ObservationSignal>) {
        self.tx = Some(tx);
    }

    /// Emit a signal to connected consumers.
    ///
    /// Returns the signal ID if successfully emitted.
    pub fn emit(&self, signal: ObservationSignal) -> Option<u64> {
        let tx = self.tx.as_ref()?;
        let signal_id = SIGNAL_COUNTER.fetch_add(1, Ordering::SeqCst);

        if tx.send(signal).is_ok() {
            Some(signal_id)
        } else {
            None
        }
    }

    /// Emit multiple signals from a semantic drift detection.
    #[must_use]
    pub fn emit_drift_signals(&self, drift: &SemanticDriftSignal) -> Vec<u64> {
        let signals = ObservationSignal::stale_from_drift(drift);
        signals.into_iter().filter_map(|s| self.emit(s)).collect()
    }

    /// Check if the bus is connected.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.tx.is_some()
    }
}

/// Convert observation signals to a streaming status format.
///
/// This function transforms internal signals into a format suitable
/// for agent notification via the `ZhenfaStreamingEvent::Status` channel.
#[must_use]
pub fn signals_to_status_batch(signals: &[ObservationSignal]) -> String {
    let mut batch = String::new();
    batch.push_str("=== Observation Signal Batch ===\n");

    for (i, signal) in signals.iter().enumerate() {
        let _ = writeln!(batch, "{}. {}", i + 1, signal.to_status_message());
    }

    let _ = write!(
        batch,
        "\nTotal: {} signal(s), {} require immediate attention",
        signals.len(),
        signals.iter().filter(|s| s.requires_attention()).count()
    );

    batch
}
