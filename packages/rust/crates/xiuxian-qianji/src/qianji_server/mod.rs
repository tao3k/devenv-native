//! `qianji-server` runtime helpers.
//!
//! CLI parsing and process startup stay under `qianji_server_cli`. This module
//! owns reusable server-side worker bridges used by HTTP routes and tests.

pub mod flowhub_worker;
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
pub mod llm_worker;
