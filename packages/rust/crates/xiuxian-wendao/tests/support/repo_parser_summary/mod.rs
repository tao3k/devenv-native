//! Deterministic parser-summary test transport for repo-intelligence fixtures.

mod modelica;
mod parser;
mod rows;
mod schema;
mod service;

pub(crate) use service::{FakeParserSummaryServiceGuard, spawn_fake_julia_parser_summary_service};
