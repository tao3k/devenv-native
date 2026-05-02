//! Thin root target for structured LLM analyzer integration coverage.

#![cfg(feature = "llm")]

#[path = "llm_analyzer_suite.rs"]
mod suite;
