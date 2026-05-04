//! Agentic settings application module surface.

mod core;
mod execution;
mod expansion;
mod scalar;
mod search;
mod suggested;

pub(super) use core::{
    apply_execution_settings, apply_expansion_settings, apply_search_settings,
    apply_suggested_link_settings,
};
