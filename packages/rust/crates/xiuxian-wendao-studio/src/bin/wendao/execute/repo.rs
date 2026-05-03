//! Repo Intelligence command execution handler.

use std::env;

use anyhow::Result;

use crate::bin_support::wendao::helpers::emit;
use crate::bin_support::wendao::types::{Cli, Command, RepoCommand, RepoSyncModeArg};
use xiuxian_wendao::{
    DocCoverageQuery, ExampleSearchQuery, ModuleSearchQuery, RepoOverviewQuery, RepoSyncMode,
    RepoSyncQuery, SymbolSearchQuery, doc_coverage_from_config, example_search_from_config,
    module_search_from_config, repo_overview_from_config, repo_sync_from_config,
    symbol_search_from_config,
};

pub(super) fn handle(cli: &Cli) -> Result<()> {
    let Command::Repo { command } = &cli.command else {
        unreachable!("repo handler called with non-repo command");
    };

    match command {
        RepoCommand::Sync(args) => {
            let cwd = env::current_dir()?;
            let query = RepoSyncQuery {
                repo_id: args.repo.clone(),
                mode: match args.mode {
                    RepoSyncModeArg::Ensure => RepoSyncMode::Ensure,
                    RepoSyncModeArg::Refresh => RepoSyncMode::Refresh,
                    RepoSyncModeArg::Status => RepoSyncMode::Status,
                },
            };
            let result = repo_sync_from_config(&query, cli.config_file.as_deref(), &cwd)?;
            emit(&result, cli.output_or_json())
        }
        RepoCommand::Overview(args) => {
            let cwd = env::current_dir()?;
            let query = RepoOverviewQuery {
                repo_id: args.repo.clone(),
            };
            let result = repo_overview_from_config(&query, cli.config_file.as_deref(), &cwd)?;
            emit(&result, cli.output_or_json())
        }
        RepoCommand::ModuleSearch(args) => {
            let cwd = env::current_dir()?;
            let query = ModuleSearchQuery {
                repo_id: args.repo.clone(),
                query: args.query.clone(),
                limit: args.limit,
            };
            let result = module_search_from_config(&query, cli.config_file.as_deref(), &cwd)?;
            emit(&result, cli.output_or_json())
        }
        RepoCommand::SymbolSearch(args) => {
            let cwd = env::current_dir()?;
            let query = SymbolSearchQuery {
                repo_id: args.repo.clone(),
                query: args.query.clone(),
                limit: args.limit,
            };
            let result = symbol_search_from_config(&query, cli.config_file.as_deref(), &cwd)?;
            emit(&result, cli.output_or_json())
        }
        RepoCommand::ExampleSearch(args) => {
            let cwd = env::current_dir()?;
            let query = ExampleSearchQuery {
                repo_id: args.repo.clone(),
                query: args.query.clone(),
                limit: args.limit,
            };
            let result = example_search_from_config(&query, cli.config_file.as_deref(), &cwd)?;
            emit(&result, cli.output_or_json())
        }
        RepoCommand::DocCoverage(args) => {
            let cwd = env::current_dir()?;
            let query = DocCoverageQuery {
                repo_id: args.repo.clone(),
                module_id: args.module.clone(),
            };
            let result = doc_coverage_from_config(&query, cli.config_file.as_deref(), &cwd)?;
            emit(&result, cli.output_or_json())
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/bin/wendao/execute/repo.rs"]
mod tests;
