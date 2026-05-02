//! Integration tests for Qianji bootcamp laboratory API.

#[path = "test_bootcamp_api/agenda.rs"]
mod agenda;
#[path = "test_bootcamp_api/common.rs"]
mod common;
#[path = "test_bootcamp_api/core.rs"]
mod core;
#[cfg(feature = "llm")]
#[path = "test_bootcamp_api/forge.rs"]
mod forge;
