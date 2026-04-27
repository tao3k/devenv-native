//! Command-line interface entrypoint for xiuxian-wendao link-graph operations.

use anyhow::{Result, anyhow};
use clap::Parser;

#[path = "wendao/execute.rs"]
mod execute;
#[path = "wendao/helpers/mod.rs"]
mod helpers;
#[path = "wendao/types.rs"]
mod types;

use execute::{can_execute_immediate, execute, execute_immediate};
use helpers::build_index;
use types::{AgenticCommand, Cli, Command};
use xiuxian_logging::init_from_cli;
use xiuxian_wendao::{LinkGraphIndex, set_link_graph_wendao_config_override};

fn main() -> Result<()> {
    let cli = Cli::parse();
    if can_execute_immediate(&cli.command) {
        return execute_immediate(&cli);
    }

    init_from_cli("xiuxian_wendao", &cli.logging).map_err(|err| anyhow!(err))?;

    let mut config_path = cli.config_file.clone();
    if config_path.is_none() {
        let local_toml = std::path::Path::new("wendao.toml");
        if local_toml.exists() {
            config_path = Some(local_toml.to_path_buf());
        }
    }

    if let Some(conf) = &config_path
        && let Some(path_str) = conf.to_str()
    {
        set_link_graph_wendao_config_override(path_str);
    }

    let needs_index = command_requires_index(&cli.command);
    if needs_index {
        let index = build_index(&cli)?;
        execute_with_runtime(&cli, Some(&index))
    } else {
        execute_with_runtime(&cli, None)
    }
}

fn execute_with_runtime(cli: &Cli, index: Option<&LinkGraphIndex>) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(execute(cli, index))
}

fn command_requires_index(command: &Command) -> bool {
    match command {
        Command::Audit(args) => args.template.is_none(),
        Command::Search(_)
        | Command::Attachments(_)
        | Command::Stats
        | Command::Toc(_)
        | Command::Neighbors(_)
        | Command::Related(_)
        | Command::Metadata(_)
        | Command::Resolve(_)
        | Command::Fix(_)
        | Command::Sentinel(_)
        | Command::Agentic {
            command: AgenticCommand::Plan { .. } | AgenticCommand::Run { .. },
        } => true,
        _ => false,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/bin/wendao/main.rs"]
mod tests;
