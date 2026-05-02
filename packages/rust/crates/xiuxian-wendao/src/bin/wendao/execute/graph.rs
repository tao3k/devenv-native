//! Graph traversal and metadata command execution.

#[path = "graph/metadata_resolve.rs"]
mod metadata_resolve;
#[path = "graph/neighbors_related.rs"]
mod neighbors_related;
#[path = "graph/stats_toc.rs"]
mod stats_toc;

use crate::LinkGraphIndex;
use crate::bin_support::wendao::types::{Cli, Command};
use anyhow::Result;

pub(super) fn handle(cli: &Cli, index: Option<&LinkGraphIndex>) -> Result<()> {
    match &cli.command {
        Command::Stats => stats_toc::handle_stats(cli, index),
        Command::Toc(args) => stats_toc::handle_toc(cli, index, args.limit),
        Command::Neighbors(args) => neighbors_related::handle_neighbors(
            cli,
            index,
            &args.stem,
            &args.direction,
            args.hops,
            args.limit,
            args.verbose,
        ),
        Command::Related(args) => {
            let related_args = neighbors_related::RelatedArgs {
                stem: &args.stem,
                max_distance: args.max_distance,
                limit: args.limit,
                verbose: args.verbose,
                ppr_alpha: args.ppr_alpha,
                ppr_max_iter: args.ppr_max_iter,
                ppr_tol: args.ppr_tol,
                ppr_subgraph_mode: args.ppr_subgraph_mode,
            };
            neighbors_related::handle_related(cli, index, &related_args)
        }
        Command::Metadata(args) => metadata_resolve::handle_metadata(cli, index, &args.stem),
        Command::Resolve(args) => {
            metadata_resolve::handle_resolve(cli, index, &args.alias, args.limit)
        }
        _ => unreachable!("graph handler must be called with graph command"),
    }
}
