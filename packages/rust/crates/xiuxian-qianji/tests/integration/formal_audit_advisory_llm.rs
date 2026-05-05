//! Thin root target for LLM-backed formal-audit advisory coverage.

#![cfg(feature = "llm")]

#[path = "formal_audit_advisory_llm_suite.rs"]
mod suite;
