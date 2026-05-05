//! Streaming LLM analyzer feature seam. Start in `api`.

#[path = "api.rs"]
mod api;
#[path = "execution.rs"]
mod execution;
#[path = "output.rs"]
mod output;

pub use api::{
    OutputFlags, PipelineFlags, StreamingLlmAnalyzer, StreamingLlmAnalyzerBuilder,
    StreamingPipelineSettings,
};

#[cfg(test)]
#[path = "../../tests/unit/executors/llm/streaming.rs"]
mod tests;
