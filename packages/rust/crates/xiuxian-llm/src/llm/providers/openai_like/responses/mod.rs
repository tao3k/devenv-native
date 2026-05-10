//! `OpenAI` Responses API payload building and stream parsing.

mod inspect;
mod payload;
mod stream;
mod types;

pub(crate) use inspect::{
    summarize_openai_responses_input, validate_openai_responses_input_tool_chain,
};
pub use payload::{build_openai_responses_payload, remap_openai_responses_tool_name};
pub use stream::parse_openai_responses_stream;
pub use types::{
    OpenAiResponsesAssistantOutput, OpenAiResponsesPayload, OpenAiResponsesToolCall,
    OpenAiResponsesToolType,
};
