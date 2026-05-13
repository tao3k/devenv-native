//! Zhenfa Pipeline: The Sovereign Encapsulation for xiuxian-qianji.
//!
//! This module provides a unified pipeline that integrates all streaming
//! defense mechanisms into a single coherent interface:
//!
//! 1. **`LogicGate`**: Incremental XSD validation against physical laws
//! 2. **`CognitiveSupervisor`**: Real-time cognitive state monitoring
//! 3. **`StreamingTransmuter`**: Multi-provider stream parsing
//! 4. **`ExternalSignalFusion`**: Heterogeneous event injection (Phase 7.3)
//!
//! # Architecture
//!
//! ```text
//! Raw Stream → Parser → LogicGate → CognitiveSupervisor → Output
//!                ↓           ↓              ↓
//!            Provider    XSD Rules    Coherence Score
//!                ↓
//!      External Signals (from Sentinel/ObservationBus)
//!                ↓
//!         Fusion Engine → PipelineOutput.external_signals
//! ```
//!
//! This is the primary interface for xiuxian-qianji to consume streaming
//! output with full cognitive sovereignty protection.

use tokio::sync::mpsc::UnboundedReceiver;

use super::logic_gate::{LogicGate, LogicGateError, LogicGateEvent};
use super::supervisor::{CognitiveDimension, CognitiveEvent, CognitiveSupervisor};
use super::traits::StreamingTransmuter;
use super::{ClaudeStreamingParser, CodexStreamingParser, GeminiStreamingParser};
use super::{StreamingOutcome, ZhenfaStreamingEvent};
use crate::ZhenfaSignalType;

/// External signal that can be injected into the pipeline.
///
/// These signals come from external sources like Sentinel's `ObservationBus`
/// and allow the pipeline to respond to real-time semantic drift events.
#[derive(Debug, Clone)]
pub struct ExternalSignal {
    /// Signal source identifier.
    pub source: String,
    /// Signal type (e.g., `semantic_drift`, `observation_stale`).
    pub signal_type: ZhenfaSignalType,
    /// Human-readable summary.
    pub summary: String,
    /// Confidence level (0.0 to 1.0).
    pub confidence: f32,
    /// Affected document IDs.
    pub affected_docs: Vec<String>,
    /// Whether auto-fix is available.
    pub auto_fix_available: bool,
    /// Timestamp of the signal.
    pub timestamp: String,
}

/// Provider type for the streaming pipeline.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum StreamProvider {
    /// Claude Code CLI (Anthropic).
    #[default]
    Claude,
    /// Gemini CLI (Google).
    Gemini,
    /// Codex / OpenAI-style agents.
    Codex,
}

/// Construction options for a streaming pipeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZhenfaPipelineOptions {
    /// Provider parser to use.
    pub provider: StreamProvider,
    /// Whether XML/XSD hot validation is enabled.
    pub validate_xsd: bool,
    /// Whether cognitive monitoring is enabled.
    pub monitor_cognitive: bool,
    /// Coherence threshold for early halt; zero disables early halt.
    pub early_halt_threshold: f32,
}

impl ZhenfaPipelineOptions {
    /// Create default options for one provider.
    #[must_use]
    pub const fn new(provider: StreamProvider) -> Self {
        Self {
            provider,
            validate_xsd: true,
            monitor_cognitive: true,
            early_halt_threshold: 0.3,
        }
    }

    /// Set XML/XSD validation mode.
    #[must_use]
    pub const fn with_xsd_validation(mut self, enabled: bool) -> Self {
        self.validate_xsd = enabled;
        self
    }

    /// Set cognitive monitoring mode.
    #[must_use]
    pub const fn with_cognitive_monitoring(mut self, enabled: bool) -> Self {
        self.monitor_cognitive = enabled;
        self
    }

    /// Set early-halt coherence threshold.
    #[must_use]
    pub const fn with_early_halt_threshold(mut self, threshold: f32) -> Self {
        self.early_halt_threshold = threshold;
        self
    }
}

/// Output from the `ZhenfaPipeline` after processing a chunk.
#[derive(Debug, Clone)]
pub struct PipelineOutput {
    /// Original streaming event.
    pub event: ZhenfaStreamingEvent,
    /// Cognitive classification (if applicable).
    pub cognitive: Option<CognitiveEvent>,
    /// XSD validation events (if any).
    pub validation: Vec<LogicGateEvent>,
    /// Current coherence score.
    pub coherence_score: f32,
    /// Whether early halt should be triggered.
    pub should_halt: bool,
    /// External signals injected from `ObservationBus` (Phase 7.3).
    ///
    /// These signals are polled non-blockingly before each `process_line`
    /// and allow the pipeline to respond to real-time semantic drift events.
    pub external_signals: Vec<ExternalSignal>,
}

/// Errors that can occur during pipeline processing.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    /// Parsing error from the streaming provider.
    #[error("parse error: {0}")]
    ParseError(String),
    /// XSD validation violation.
    #[error("validation error: {0}")]
    ValidationError(LogicGateError),
    /// Early halt triggered due to low coherence.
    #[error("early halt triggered: coherence score {score} below threshold {threshold}")]
    EarlyHaltTriggered {
        /// Current coherence score.
        score: f32,
        /// Threshold that triggered the halt.
        threshold: f32,
    },
}

/// The sovereign streaming pipeline for xiuxian-qianji.
///
/// This struct encapsulates all streaming defense mechanisms and provides
/// a unified interface for processing streaming output with full protection.
///
/// # Phase 7.3: Heterogeneous Event Fusion
///
/// The pipeline can receive external signals from the `ObservationBus` via
/// an `mpsc::UnboundedReceiver<ExternalSignal>`. These signals are polled
/// non-blockingly before each `process_line` call, allowing real-time
/// semantic drift notifications to be injected into the stream.
pub struct ZhenfaPipeline {
    /// The streaming parser for the current provider.
    parser: Box<dyn StreamingTransmuter>,
    /// Logic gate for XSD validation.
    logic_gate: LogicGate,
    /// Cognitive supervisor for state monitoring.
    supervisor: CognitiveSupervisor,
    /// Current provider.
    provider: StreamProvider,
    /// Whether XSD validation is enabled.
    validate_xsd: bool,
    /// Whether cognitive monitoring is enabled.
    monitor_cognitive: bool,
    /// External signal receiver for heterogeneous event fusion (Phase 7.3).
    ///
    /// This receiver is polled non-blockingly in `process_line` to inject
    /// external signals from the `ObservationBus` into the pipeline output.
    signal_rx: Option<UnboundedReceiver<ExternalSignal>>,
}

impl std::fmt::Debug for ZhenfaPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZhenfaPipeline")
            .field("provider", &self.provider)
            .field("validate_xsd", &self.validate_xsd)
            .field("monitor_cognitive", &self.monitor_cognitive)
            .field("has_signal_rx", &self.signal_rx.is_some())
            .field("logic_gate", &self.logic_gate)
            .field("supervisor", &self.supervisor)
            .finish_non_exhaustive()
    }
}

impl ZhenfaPipeline {
    /// Create a new pipeline for the given provider.
    #[must_use]
    pub fn new(provider: StreamProvider) -> Self {
        Self::with_options(ZhenfaPipelineOptions::new(provider))
    }

    /// Create a pipeline with custom options.
    #[must_use]
    pub fn with_options(options: ZhenfaPipelineOptions) -> Self {
        let parser: Box<dyn StreamingTransmuter> = match options.provider {
            StreamProvider::Claude => Box::new(ClaudeStreamingParser::new()),
            StreamProvider::Gemini => Box::new(GeminiStreamingParser::new()),
            StreamProvider::Codex => Box::new(CodexStreamingParser::new()),
        };

        let supervisor = if options.early_halt_threshold > 0.0 {
            CognitiveSupervisor::with_early_halt_threshold(options.early_halt_threshold)
        } else {
            CognitiveSupervisor::new()
        };

        Self {
            parser,
            logic_gate: LogicGate::new(),
            supervisor,
            provider: options.provider,
            validate_xsd: options.validate_xsd,
            monitor_cognitive: options.monitor_cognitive,
            signal_rx: None,
        }
    }

    /// Attach an external signal receiver for heterogeneous event fusion.
    ///
    /// This allows the pipeline to receive real-time signals from external
    /// sources like the `ObservationBus`, enabling semantic drift notifications
    /// to be injected into the streaming output.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use tokio::sync::mpsc;
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// pipeline.attach_signal_receiver(rx);
    ///
    /// // In another task:
    /// tx.send(ExternalSignal {
    ///     source: "sentinel".to_string(),
    ///     signal_type: "semantic_drift".to_string(),
    ///     summary: "Source file changed".to_string(),
    ///     confidence: 0.85,
    ///     affected_docs: vec!["docs/api".to_string()],
    ///     auto_fix_available: true,
    ///     timestamp: chrono::Utc::now().to_rfc3339(),
    /// }).unwrap();
    /// ```
    pub fn attach_signal_receiver(&mut self, rx: UnboundedReceiver<ExternalSignal>) {
        self.signal_rx = Some(rx);
    }

    /// Poll external signals non-blockingly.
    ///
    /// This method drains all pending signals from the receiver without blocking.
    /// It should be called before processing each line to ensure timely signal delivery.
    fn poll_external_signals(&mut self) -> Vec<ExternalSignal> {
        let Some(ref mut rx) = self.signal_rx else {
            return Vec::new();
        };

        let mut signals = Vec::new();
        while let Ok(signal) = rx.try_recv() {
            signals.push(signal);
        }
        signals
    }

    /// Process a line of streaming output.
    ///
    /// # Errors
    ///
    /// Returns `PipelineError` when parsing fails, validation fails, or
    /// early halt is triggered.
    pub fn process_line(&mut self, line: &str) -> Result<Vec<PipelineOutput>, PipelineError> {
        let external_signals = self.poll_external_signals();
        self.reject_when_early_halt_active()?;
        let events = self.parse_streaming_events(line)?;

        let mut outputs = Vec::with_capacity(events.len().max(1));
        if self.push_signal_only_output(&events, external_signals.as_slice(), &mut outputs) {
            return Ok(outputs);
        }

        for (index, event) in events.into_iter().enumerate() {
            let output = self.process_event(
                &event,
                if index == 0 {
                    external_signals.clone()
                } else {
                    Vec::new()
                },
            )?;
            outputs.push(output);
        }

        Ok(outputs)
    }

    fn reject_when_early_halt_active(&self) -> Result<(), PipelineError> {
        if self.supervisor.should_halt() {
            return Err(PipelineError::EarlyHaltTriggered {
                score: self.supervisor.coherence().score,
                threshold: self.supervisor.early_halt_threshold(),
            });
        }
        Ok(())
    }

    fn parse_streaming_events(
        &mut self,
        line: &str,
    ) -> Result<Vec<ZhenfaStreamingEvent>, PipelineError> {
        self.parser
            .parse_line(line)
            .map_err(PipelineError::ParseError)
    }

    fn push_signal_only_output(
        &self,
        events: &[ZhenfaStreamingEvent],
        external_signals: &[ExternalSignal],
        outputs: &mut Vec<PipelineOutput>,
    ) -> bool {
        if !events.is_empty() || external_signals.is_empty() {
            return false;
        }
        outputs.push(PipelineOutput {
            event: ZhenfaStreamingEvent::Status(std::sync::Arc::from("external_signal")),
            cognitive: None,
            validation: Vec::new(),
            coherence_score: self.supervisor.coherence().score,
            should_halt: false,
            external_signals: external_signals.to_vec(),
        });
        true
    }

    fn process_event(
        &mut self,
        event: &ZhenfaStreamingEvent,
        external_signals: Vec<ExternalSignal>,
    ) -> Result<PipelineOutput, PipelineError> {
        let mut output = PipelineOutput {
            event: event.clone(),
            cognitive: None,
            validation: Vec::new(),
            coherence_score: self.supervisor.coherence().score,
            should_halt: false,
            external_signals,
        };

        self.apply_cognitive_monitoring(event, &mut output);
        self.apply_hot_validation(event, &mut output)?;
        Ok(output)
    }

    fn apply_cognitive_monitoring(
        &mut self,
        event: &ZhenfaStreamingEvent,
        output: &mut PipelineOutput,
    ) {
        if !self.monitor_cognitive || !matches!(event, ZhenfaStreamingEvent::Thought(_)) {
            return;
        }
        let cognitive = self.supervisor.classify(event.clone());
        output.coherence_score = cognitive.coherence;
        output.should_halt = self.supervisor.should_halt();
        output.cognitive = Some(cognitive);
    }

    fn apply_hot_validation(
        &mut self,
        event: &ZhenfaStreamingEvent,
        output: &mut PipelineOutput,
    ) -> Result<(), PipelineError> {
        if !self.validate_xsd {
            return Ok(());
        }
        let Some(text) = event.text_content() else {
            return Ok(());
        };
        output.validation = self
            .logic_gate
            .hot_validate(text)
            .map_err(PipelineError::ValidationError)?;
        Ok(())
    }

    /// Finalize the stream and get the final outcome.
    ///
    /// # Errors
    ///
    /// Returns `PipelineError` if finalization fails.
    pub fn finalize(&mut self) -> Result<Option<StreamingOutcome>, PipelineError> {
        let event = self.parser.finalize().map_err(PipelineError::ParseError)?;

        // Convert ZhenfaStreamingEvent to StreamingOutcome
        let outcome = event.and_then(|e| match e {
            ZhenfaStreamingEvent::Finished(outcome) => Some(outcome),
            _ => None,
        });

        Ok(outcome)
    }

    /// Get the current accumulated text.
    #[must_use]
    pub fn accumulated_text(&self) -> &str {
        self.parser.accumulated_text()
    }

    /// Get the current coherence score.
    #[must_use]
    pub fn coherence_score(&self) -> f32 {
        self.supervisor.coherence().score
    }

    /// Check if early halt should be triggered.
    #[must_use]
    pub fn should_halt(&self) -> bool {
        self.supervisor.should_halt()
    }

    /// Get the cognitive dimension distribution.
    #[must_use]
    pub fn cognitive_distribution(&self) -> CognitiveDistribution {
        self.supervisor
            .history_slice(0, self.supervisor.history_len())
            .iter()
            .copied()
            .fold(CognitiveDistribution::default(), |mut distribution, dim| {
                distribution.increment(dim);
                distribution
            })
    }

    /// Reset the pipeline state for a new session.
    pub fn reset(&mut self) {
        self.parser.reset();
        self.logic_gate.reset();
        self.supervisor.reset();
    }

    /// Get the provider type.
    #[must_use]
    pub const fn provider(&self) -> StreamProvider {
        self.provider
    }
}

/// Distribution of cognitive dimensions in the current session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CognitiveDistribution {
    /// Number of Meta (planning/reflection) thoughts.
    pub meta: usize,
    /// Number of Operational (implementation) thoughts.
    pub operational: usize,
    /// Number of Epistemic (uncertainty/knowledge-gap) thoughts.
    pub epistemic: usize,
    /// Number of Instrumental (tool-use) thoughts.
    pub instrumental: usize,
    /// Number of System events.
    pub system: usize,
}

impl CognitiveDistribution {
    fn increment(&mut self, dim: CognitiveDimension) {
        match dim {
            CognitiveDimension::Meta => self.meta += 1,
            CognitiveDimension::Operational => self.operational += 1,
            CognitiveDimension::Epistemic => self.epistemic += 1,
            CognitiveDimension::Instrumental => self.instrumental += 1,
            CognitiveDimension::System => self.system += 1,
        }
    }

    /// Get the total number of events.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.meta + self.operational + self.epistemic + self.instrumental + self.system
    }

    /// Get the cognitive balance (ratio of meta to operational).
    #[must_use]
    pub fn balance(&self) -> f32 {
        let total = self.meta + self.operational;
        if total == 0 {
            return 0.5;
        }
        ratio_from_counts(self.meta, total)
    }

    /// Get the uncertainty ratio (epistemic / total).
    #[must_use]
    pub fn uncertainty_ratio(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }
        ratio_from_counts(self.epistemic, total)
    }
}

fn ratio_from_counts(numerator: usize, denominator: usize) -> f32 {
    debug_assert!(denominator > 0);
    saturating_usize_to_f32(numerator) / saturating_usize_to_f32(denominator)
}

fn saturating_usize_to_f32(value: usize) -> f32 {
    f32::from(u16::try_from(value).unwrap_or(u16::MAX))
}

#[cfg(test)]
#[path = "../../../tests/unit/transmuter/streaming/pipeline.rs"]
mod tests;
