//! `qianji-server` command implementation.

mod cli;
#[path = "facade.rs"]
mod facade;
mod flowhub;
pub(crate) mod flowhub_worker;
mod health;
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
pub(crate) mod llm_worker;
mod run;
#[cfg(test)]
#[path = "../../tests/unit/bin/qianji_server/mod.rs"]
mod tests;

pub use facade::{QianjiServerCliError, run_qianji_server_cli};
