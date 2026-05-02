//! `qianji-server` command implementation.

mod cli;
mod health;
mod run;

/// Runs the `qianji-server` command-line interface.
///
/// # Errors
/// Returns an error if argument parsing, socket binding, or HTTP serving fails.
pub async fn run_qianji_server_cli<I, S>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let command = cli::parse_qianji_server_args(args)?;
    run::run_qianji_server(command).await
}

#[cfg(test)]
#[path = "../tests/unit/bin/qianji_server/mod.rs"]
mod tests;
