//! Qianji server binary entry seam.
//!
//! The server shell owns transport startup only; workflow control stays in the
//! library-owned BPMN HTTP router.

#[path = "qianji_server/cli.rs"]
mod cli;
#[path = "qianji_server/health.rs"]
mod health;
#[path = "qianji_server/run.rs"]
mod run;

/// Main entry point for the Qianji HTTP service shell.
///
/// # Errors
/// Returns an error if argument parsing, socket binding, or HTTP serving fails.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let command = cli::parse_qianji_server_args(std::env::args().skip(1))?;
    run::run_qianji_server(command).await
}

#[cfg(test)]
#[path = "../../tests/unit/bin/qianji_server/mod.rs"]
mod tests;
