//! Command dispatch implementation for `wendao` CLI.

use crate::bin_support::wendao::types::{Cli, Command, OutputFormat};
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
        Command::Audit(args) if args.template.is_some() => super::audit::handle(cli, args, None),
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
        Command::Search(_) => super::search::handle(cli, index),
        Command::Audit(args) => super::audit::handle(cli, args, index),
        Command::Attachments(_) => super::attachments::handle(cli, index),
        Command::Stats
        | Command::Toc(_)
        | Command::Neighbors(_)
        | Command::Related(_)
        | Command::Metadata(_)
        | Command::Resolve(_) => super::graph::handle(cli, index),
        Command::Saliency { .. } => super::saliency::handle(cli),
        Command::Hmas { .. } => super::hmas::handle(cli),
        Command::Episteme { .. } => super::episteme::handle(cli),
        Command::Agentic { .. } => super::agentic::handle(cli, index),
        Command::Repo { .. } => super::repo::handle(cli),
        Command::Docs { .. } => super::docs::handle(cli),
        Command::Client(command) => {
            let outcome =
                xiuxian_wendao_client::run_command(command, &client_context_from_cli(cli))?;
            if outcome.exit_code() != 0 {
                std::process::exit(i32::from(outcome.exit_code()));
            }
            Ok(())
        }
        #[cfg(feature = "zhenfa-router")]
        Command::Query { .. } => super::query::handle(cli).await,
        Command::Fix(args) => super::fix::handle(cli, args, index),
        #[cfg(feature = "zhenfa-router")]
        Command::Gateway(args) => super::gateway::handle(cli, args, index).await,
        Command::Sentinel(args) => super::sentinel::handle(cli, args, index).await,
    }
}

pub(super) fn client_context_from_cli(cli: &Cli) -> EmbeddedClientContext {
    let output = match cli.embedded_client_output() {
        OutputFormat::Text => ClientOutputFormat::Text,
        OutputFormat::Json => ClientOutputFormat::Json,
        OutputFormat::Pretty => ClientOutputFormat::Pretty,
    };
    EmbeddedClientContext::new(cli.root.clone(), output).with_config_file(cli.config_file.clone())
}
