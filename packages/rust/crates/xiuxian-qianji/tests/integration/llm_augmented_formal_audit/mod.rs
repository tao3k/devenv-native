//! Thin root target for LLM-augmented formal audit coverage.

#![cfg(feature = "llm")]
#![cfg(feature = "wendao-integration")]

#[path = "suite.rs"]
mod suite;
