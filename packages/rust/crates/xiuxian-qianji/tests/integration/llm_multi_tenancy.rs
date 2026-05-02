//! Thin root target for node-level LLM multi-tenancy coverage.

#![cfg(feature = "llm")]

#[path = "llm_multi_tenancy_suite.rs"]
mod suite;
