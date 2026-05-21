//! Thought Aggregator for Sovereign Memory (Blueprint V6.1).
//!
//! Captures streaming events during workflow execution and aggregates
//! them into `CognitiveTrace` artifacts for persistent storage in Wendao.

use std::sync::Arc;
use xiuxian_wendao_core::CognitiveTraceRecord;
use xiuxian_zhenfa::ZhenfaStreamingEvent;

/// Aggregates streaming events into a cognitive trace artifact.
///
/// This struct captures the reasoning flow during workflow execution,
/// enabling "historical sovereignty" - the ability to query the knowledge
/// graph for the reasoning chain that led to any decision or commit.
#[derive(Debug, Clone)]
pub struct ThoughtAggregator {
    /// Unique identifier for this trace.
    trace_id: String,
    /// Session identifier from Qianji execution.
    session_id: Option<String>,
    /// Node identifier from the compiled flow graph.
    node_id: String,
    /// The original user intent/prompt.
    intent: String,
    /// Aggregated reasoning content.
    reasoning_chunks: Vec<String>,
    /// Tool calls made during execution.
    tool_calls: Vec<ToolCallRecord>,
    /// Final outcome or conclusion.
    outcome: Option<String>,
    /// Cognitive coherence score during execution.
    coherence_score: Option<f32>,
    /// Whether early halt was triggered.
    early_halt_triggered: bool,
    /// Timestamp when aggregation started.
    start_timestamp_ms: u64,
}

/// Record of a tool call during execution.
#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    /// Tool call identifier.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Input parameters.
    pub input: serde_json::Value,
    /// Output result (if available).
    pub output: Option<serde_json::Value>,
}

impl ThoughtAggregator {
    /// Create a new thought aggregator for a workflow node.
    #[must_use]
    pub fn new(session_id: Option<String>, node_id: String, intent: String) -> Self {
        let trace_id = format!("trace-{}-{}", node_id, unix_timestamp_ms());
        Self {
            trace_id,
            session_id,
            node_id,
            intent,
            reasoning_chunks: Vec::new(),
            tool_calls: Vec::new(),
            outcome: None,
            coherence_score: None,
            early_halt_triggered: false,
            start_timestamp_ms: unix_timestamp_ms(),
        }
    }

    /// Process a streaming event and aggregate into the trace.
    pub fn process_event(&mut self, event: &ZhenfaStreamingEvent) {
        match event {
            ZhenfaStreamingEvent::Thought(text) => {
                self.reasoning_chunks.push(format!("[THOUGHT] {text}"));
            }
            ZhenfaStreamingEvent::TextDelta(text) => {
                self.reasoning_chunks.push(text.to_string());
            }
            ZhenfaStreamingEvent::ToolCall { id, name, input } => {
                self.tool_calls.push(ToolCallRecord {
                    id: id.to_string(),
                    name: name.to_string(),
                    input: input.clone(),
                    output: None,
                });
            }
            ZhenfaStreamingEvent::ToolResult { id, output } => {
                if let Some(tool_call) = self
                    .tool_calls
                    .iter_mut()
                    .rev()
                    .find(|tc| tc.id.as_str() == id.as_ref())
                {
                    tool_call.output = Some(output.clone());
                }
            }
            ZhenfaStreamingEvent::Status(text) => {
                self.reasoning_chunks.push(format!("[STATUS] {text}"));
            }
            ZhenfaStreamingEvent::Progress { message, percent } => {
                self.reasoning_chunks
                    .push(format!("[PROGRESS {percent}%] {message}"));
            }
            ZhenfaStreamingEvent::Finished(outcome) => {
                self.outcome = Some(outcome.final_text.as_ref().to_string());
            }
            ZhenfaStreamingEvent::Error { code, message } => {
                self.reasoning_chunks
                    .push(format!("[ERROR {code}] {message}"));
            }
        }
    }

    /// Set the cognitive coherence score.
    pub fn set_coherence_score(&mut self, score: f32) {
        self.coherence_score = Some(score);
    }

    /// Mark that early halt was triggered.
    pub fn set_early_halt(&mut self) {
        self.early_halt_triggered = true;
    }

    /// Set the final outcome.
    pub fn set_outcome(&mut self, outcome: String) {
        self.outcome = Some(outcome);
    }

    /// Build the final cognitive trace record.
    #[must_use]
    pub fn build(self) -> CognitiveTraceRecord {
        let reasoning = self.reasoning_chunks.join("\n");
        let reasoning_arc: Arc<str> = Arc::<str>::from(reasoning);
        let outcome_arc = self.outcome.map(Arc::<str>::from);

        CognitiveTraceRecord {
            trace_id: self.trace_id.into(),
            session_id: self.session_id.map(Into::into),
            node_id: self.node_id.into(),
            intent: self.intent,
            reasoning: reasoning_arc,
            outcome: outcome_arc,
            commit_sha: None,
            timestamp_ms: self.start_timestamp_ms,
            coherence_score: self.coherence_score,
            early_halt_triggered: self.early_halt_triggered,
        }
    }

    /// Get the current reasoning length (for budget tracking).
    #[must_use]
    pub fn reasoning_length(&self) -> usize {
        self.reasoning_chunks
            .iter()
            .map(std::string::String::len)
            .sum()
    }

    /// Check if the aggregator has captured any content.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reasoning_chunks.is_empty() && self.tool_calls.is_empty()
    }
}

/// Get current timestamp in milliseconds.
fn unix_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "thought_aggregator.rs"]
mod tests;
