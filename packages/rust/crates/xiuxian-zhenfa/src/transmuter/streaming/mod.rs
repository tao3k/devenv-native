//! Unified streaming parser for multi-agent CLI outputs.
//!
//! This module provides a common abstraction for parsing streaming output
//! from various LLM CLI tools (Claude Code, Gemini CLI, Codex) into a unified
//! event stream that can be consumed by Qianji nodes.
//!
//! # Zero-Copy Architecture
//!
//! All text content uses `Arc<str>` for zero-copy sharing across consumers,
//! eliminating heap allocations for each text delta in high-throughput scenarios.
//!
//! # Pipeline Architecture
//!
//! The `ZhenfaPipeline` provides the sovereign encapsulation for xiuxian-qianji:
//!
//! ```text
//! Raw Stream -> Parser -> LogicGate -> CognitiveSupervisor -> Output
//! ```

mod claude;
mod codex;
mod event;
mod gemini;
mod logic_gate;
mod pipeline;
mod supervisor;
mod traits;

#[cfg(test)]
#[path = "../../../tests/unit/transmuter/streaming/mod.rs"]
mod tests;

pub use claude::ClaudeStreamingParser;
pub use codex::CodexStreamingParser;
pub use event::ZhenfaStreamingEvent;
pub use gemini::GeminiStreamingParser;
pub use pipeline::{
    CognitiveDistribution, ExternalSignal, PipelineError, PipelineOutput, StreamProvider,
    ZhenfaPipeline,
};
pub use traits::{StreamingOutcome, StreamingTransmuter, TokenUsage};
