//! `qianji-server` command implementation.

mod cli;
#[path = "facade.rs"]
mod facade;
mod health;
mod run;
#[cfg(test)]
#[path = "../../tests/unit/bin/qianji_server/mod.rs"]
mod tests;

pub use facade::run_qianji_server_cli;
