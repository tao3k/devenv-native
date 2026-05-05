//! Runtime agent factory wiring for daochang channel entrypoints.

mod builder;
mod env_lookup;
mod inference;
mod logging;
mod memory;
mod session;
mod tools;
mod types;

pub use builder::build_agent;
pub(crate) use inference::{
    parse_embedding_backend_mode, resolve_inference_url, resolve_runtime_embedding_backend_mode,
    resolve_runtime_embedding_base_url, resolve_runtime_inference_url, resolve_runtime_model,
    validate_inference_url_origin,
};
pub(crate) use memory::resolve_runtime_memory_options;
pub(crate) use tools::{resolve_runtime_tool_options, resolve_runtime_tool_servers};
pub(crate) use types::MemoryRuntimeOptions;
