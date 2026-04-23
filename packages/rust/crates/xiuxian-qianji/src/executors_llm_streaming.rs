//! Streaming LLM analyzer feature seam. Start in `api`.

#[path = "executors_llm_streaming/api.rs"]
mod api;
#[path = "executors_llm_streaming/execution.rs"]
mod execution;
#[path = "executors_llm_streaming/output.rs"]
mod output;

pub use api::{
    OutputFlags, PipelineFlags, StreamingLlmAnalyzer, StreamingLlmAnalyzerBuilder,
    StreamingPipelineSettings,
};

#[cfg(test)]
#[path = "../tests/unit/executors/llm/streaming.rs"]
mod tests;
