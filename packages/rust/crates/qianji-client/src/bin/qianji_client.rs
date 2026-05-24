//! Qianji client binary entry point.

/// Main entry point for the Qianji client CLI.
///
/// # Errors
/// Returns an error when command parsing, materialization, or validation fails.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(qianji_client::run_qianji_client_cli()?)
}
