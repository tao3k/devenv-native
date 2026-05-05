//! System prompt injection branch for assembling and persisting overrides.

mod assembler;
mod builder;
mod normalization;
mod render;

pub(super) use normalization::normalize_messages_with_snapshot;
