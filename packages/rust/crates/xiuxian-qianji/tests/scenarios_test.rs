//! Thin root target for Qianji scenario coverage.

#![cfg(feature = "qianji-full")]

#[path = "support/mod.rs"]
mod support;

#[path = "integration/scenarios_test/suite.rs"]
mod suite;
