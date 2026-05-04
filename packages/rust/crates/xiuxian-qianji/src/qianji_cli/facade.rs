use super::dispatch;

/// Runs the `qianji` command-line interface.
///
/// # Errors
/// Returns an error if argument parsing, environment resolution, compilation, or execution fails.
pub async fn run_qianji_cli() -> Result<(), Box<dyn std::error::Error>> {
    Box::pin(dispatch::run()).await
}
