//! Gateway subcommand for starting the Wendao API server.
//!
//! This command starts the Axum-based HTTP server that provides:
//! REST API endpoints for the Wendao knowledge graph and VFS operations.

use clap::{Args, Subcommand};

/// Arguments for the `gateway` subcommand.
#[derive(Debug, Args, Clone)]
pub(crate) struct GatewayArgs {
    #[command(subcommand)]
    pub(crate) command: GatewayCommand,
}

/// Gateway subcommands.
#[derive(Debug, Subcommand, Clone)]
pub(crate) enum GatewayCommand {
    /// Start the gateway server.
    Start(GatewayStartArgs),
}

/// Arguments for `gateway start`.
#[derive(Debug, Args, Clone, Default)]
pub(crate) struct GatewayStartArgs {
    /// Port to listen on. Overrides config file if specified.
    #[arg(short, long)]
    pub(crate) port: Option<u16>,
}

/// Create a Gateway command from arguments.
#[cfg(all(test, feature = "zhenfa-router"))]
pub(crate) fn gateway(args: &GatewayArgs) -> super::Command {
    super::Command::Gateway(args.clone())
}

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/types/commands/gateway.rs"]
mod tests;
