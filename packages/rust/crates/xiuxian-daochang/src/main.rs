//! omni-agent CLI: gateway, stdio, or repl mode.
//!
//! External tool servers are loaded from `.tool.json` by default.
//! Override with `--tool-config <path>`.
//!
//! Logging: set `RUST_LOG=omni_agent=info` (or `warn`, `debug`) to see agent logs on stderr.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    xiuxian_daochang::cli_runtime::run().await
}
