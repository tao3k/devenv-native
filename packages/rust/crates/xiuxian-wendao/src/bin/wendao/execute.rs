//! Command dispatch implementation for `wendao` CLI.
//!
//! This module dispatches CLI commands to their respective handler modules.
//!
//! Each handler module implements the logic for a specific command.

#[path = "execute/agentic/mod.rs"]
mod agentic;
#[path = "execute/attachments.rs"]
mod attachments;
#[path = "execute/audit.rs"]
mod audit;
#[path = "execute/docs.rs"]
mod docs;
#[path = "execute/fix.rs"]
mod fix;
#[cfg(feature = "zhenfa-router")]
#[path = "execute/gateway/mod.rs"]
mod gateway;
#[path = "execute/graph.rs"]
mod graph;
#[path = "execute/hmas.rs"]
mod hmas;
#[cfg(feature = "zhenfa-router")]
#[path = "execute/query/mod.rs"]
mod query;
#[path = "execute/repo.rs"]
mod repo;
#[path = "execute/saliency.rs"]
mod saliency;
#[path = "execute/search.rs"]
mod search;
#[path = "execute/sentinel.rs"]
mod sentinel;

use crate::types::{Cli, Command, OutputFormat};
use anyhow::Result;
use xiuxian_wendao::LinkGraphIndex;
use xiuxian_wendao_client::{
    ClientContext as EmbeddedClientContext, OutputFormat as ClientOutputFormat,
};

pub(crate) fn can_execute_immediate(command: &Command) -> bool {
    matches!(command, Command::Audit(args) if args.template.is_some())
}

pub(crate) fn execute_immediate(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Audit(args) if args.template.is_some() => audit::handle(cli, args, None),
        _ => anyhow::bail!("command cannot execute without runtime"),
    }
}

/// Execute the CLI command.
///
/// This function dispatches the command into its respective handler.
/// All handlers take the CLI, an optional link graph index,
/// and return a Result indicating success or failure.
pub(crate) async fn execute(cli: &Cli, index: Option<&LinkGraphIndex>) -> Result<()> {
    match &cli.command {
        Command::Search(_) => search::handle(cli, index),
        Command::Audit(args) => audit::handle(cli, args, index),
        Command::Attachments(_) => attachments::handle(cli, index),
        Command::Stats
        | Command::Toc(_)
        | Command::Neighbors(_)
        | Command::Related(_)
        | Command::Metadata(_)
        | Command::Resolve(_) => graph::handle(cli, index),
        Command::Saliency { .. } => saliency::handle(cli),
        Command::Hmas { .. } => hmas::handle(cli),
        Command::Agentic { .. } => agentic::handle(cli, index),
        Command::Repo { .. } => repo::handle(cli),
        Command::Docs { .. } => docs::handle(cli),
        Command::Client(command) => {
            let outcome =
                xiuxian_wendao_client::run_command(command, &client_context_from_cli(cli))?;
            if outcome.exit_code() != 0 {
                std::process::exit(i32::from(outcome.exit_code()));
            }
            Ok(())
        }
        #[cfg(feature = "zhenfa-router")]
        Command::Query { .. } => query::handle(cli).await,
        Command::Fix(args) => fix::handle(cli, args, index),
        #[cfg(feature = "zhenfa-router")]
        Command::Gateway(args) => gateway::handle(cli, args, index).await,
        Command::Sentinel(args) => sentinel::handle(cli, args, index).await,
    }
}

fn client_context_from_cli(cli: &Cli) -> EmbeddedClientContext {
    let output = match cli.embedded_client_output() {
        OutputFormat::Text => ClientOutputFormat::Text,
        OutputFormat::Json => ClientOutputFormat::Json,
        OutputFormat::Pretty => ClientOutputFormat::Pretty,
    };
    EmbeddedClientContext::new(cli.root.clone(), output).with_config_file(cli.config_file.clone())
}

#[cfg(test)]
#[path = "../../../tests/unit/bin/wendao/execute.rs"]
mod tests;
