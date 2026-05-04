//! Runtime memory option resolution module surface.

mod core;
mod embedding;
mod env_overrides;
mod runtime;

pub(crate) use core::resolve_runtime_memory_options;
