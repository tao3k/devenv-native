//! `qianji-server` command implementation.

mod cli;
#[path = "facade.rs"]
mod facade;
mod flowhub;
mod health;
mod run;
mod security;
#[cfg(test)]
#[path = "../../tests/unit/bin/qianji_server/mod.rs"]
mod tests;

pub use facade::{QianjiServerCliError, run_qianji_server_cli};
