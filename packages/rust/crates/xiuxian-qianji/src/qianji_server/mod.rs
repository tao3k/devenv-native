//! `qianji-server` runtime helpers.
//!
//! CLI parsing and process startup stay under `qianji_server_cli`. This module
//! owns reusable server-side worker bridges used by HTTP routes and tests.

pub mod flowhub_worker;
#[cfg(all(
    feature = "llm",
    any(
        all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
        test
    )
))]
pub(crate) mod llm_worker;
