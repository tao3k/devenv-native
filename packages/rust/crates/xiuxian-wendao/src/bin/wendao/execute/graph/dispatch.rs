//! Dispatch for graph-oriented CLI commands.

use crate::LinkGraphIndex;
use crate::bin_support::wendao::types::{Cli, Command};
use anyhow::Result;

pub(in crate::bin_support::wendao::execute) fn handle(
    cli: &Cli,
    index: Option<&LinkGraphIndex>,
) -> Result<()> {
    match &cli.command {
        Command::Stats => super::stats_toc::handle_stats(cli, index),
        Command::Toc(args) => super::stats_toc::handle_toc(cli, index, args.limit),
        Command::Neighbors(args) => super::neighbors_related::handle_neighbors(
            cli,
            index,
            &args.stem,
            &args.direction,
            args.hops,
            args.limit,
            args.verbose,
        ),
        Command::Related(args) => {
            let related_args = super::neighbors_related::RelatedArgs {
                stem: &args.stem,
                max_distance: args.max_distance,
                limit: args.limit,
                verbose: args.verbose,
                ppr_alpha: args.ppr_alpha,
                ppr_max_iter: args.ppr_max_iter,
                ppr_tol: args.ppr_tol,
                ppr_subgraph_mode: args.ppr_subgraph_mode,
            };
            super::neighbors_related::handle_related(cli, index, &related_args)
        }
        Command::Metadata(args) => super::metadata_resolve::handle_metadata(cli, index, &args.stem),
        Command::Resolve(args) => {
            super::metadata_resolve::handle_resolve(cli, index, &args.alias, args.limit)
        }
        _ => unreachable!("graph handler must be called with graph command"),
    }
}
