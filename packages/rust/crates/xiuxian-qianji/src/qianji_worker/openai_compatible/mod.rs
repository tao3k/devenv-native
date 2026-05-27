//! OpenAI-compatible activity executor side-effect adapter.

mod artifact;
mod episteme;
mod failure;
mod io_support;
mod protocol;
mod response;
mod run;
mod transport;
mod types;

pub(crate) use run::execute_openai_compatible_llm;
pub(crate) use types::OpenAiCompatibleLlmExecutionRequest;
