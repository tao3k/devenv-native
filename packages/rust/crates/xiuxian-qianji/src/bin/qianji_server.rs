//! Qianji server binary entry point.

/// Main entry point for the Qianji HTTP service shell.
///
/// # Errors
/// Returns an error if argument parsing, socket binding, or HTTP serving fails.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Ok(xiuxian_qianji::run_qianji_server_cli(std::env::args().skip(1)).await?)
}
