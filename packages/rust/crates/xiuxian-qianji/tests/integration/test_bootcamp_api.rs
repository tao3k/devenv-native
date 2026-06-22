//! Integration tests for Qianji bootcamp laboratory API.

#![cfg(feature = "wendao-integration")]

#[path = "test_bootcamp_api/agenda.rs"]
mod agenda;
#[path = "test_bootcamp_api/common.rs"]
mod common;
#[path = "test_bootcamp_api/core.rs"]
mod core;
#[path = "test_bootcamp_api/forge.rs"]
mod forge;
