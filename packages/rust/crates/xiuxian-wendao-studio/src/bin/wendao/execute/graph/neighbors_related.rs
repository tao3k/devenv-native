//! Neighbor and related-traversal command handlers.

use crate::bin_support::wendao::cli_support::{
    build_optional_related_ppr_options, build_related_monitor_phases, emit,
};
use crate::bin_support::wendao::types::{Cli, RelatedPprSubgraphModeArg};
use anyhow::{Context, Result};
use serde_json::json;
use xiuxian_wendao::{LinkGraphDirection, LinkGraphIndex};

pub(crate) struct RelatedArgs<'a> {
    pub stem: &'a str,
    pub max_distance: usize,
    pub limit: usize,
    pub verbose: bool,
    pub ppr_alpha: Option<f64>,
    pub ppr_max_iter: Option<usize>,
    pub ppr_tol: Option<f64>,
    pub ppr_subgraph_mode: Option<RelatedPprSubgraphModeArg>,
}

pub(super) fn handle_neighbors(
    cli: &Cli,
    index: Option<&LinkGraphIndex>,
    stem: &str,
    direction: &str,
    hops: usize,
    limit: usize,
    _verbose: bool,
) -> Result<()> {
    let index = index.context("link_graph index is required for neighbors command")?;
    let candidates = index.resolve_metadata_candidates(stem);
    match candidates.len() {
        0 => emit(&Vec::<serde_json::Value>::new(), cli.output_or_json()),
        1 => {
            let resolved = &candidates[0].path;
            let payload = index.neighbors(
                resolved,
                LinkGraphDirection::from_alias(direction),
                hops.max(1),
                limit.max(1),
            );
            emit(&payload, cli.output_or_json())
        }
        _ => {
            let payload = json!({
                "error": "ambiguous_stem",
                "command": "neighbors",
                "query": stem,
                "count": candidates.len(),
                "message": "multiple documents matched this stem/id/path; use full id or path",
                "candidates": candidates,
            });
            emit(&payload, cli.output_or_json())
        }
    }
}

pub(super) fn handle_related(
    cli: &Cli,
    index: Option<&LinkGraphIndex>,
    args: &RelatedArgs<'_>,
) -> Result<()> {
    let index = index.context("link_graph index is required for related command")?;
    let ppr = build_optional_related_ppr_options(
        args.ppr_alpha,
        args.ppr_max_iter,
        args.ppr_tol,
        args.ppr_subgraph_mode,
    );
    let candidates = index.resolve_metadata_candidates(args.stem);
    if candidates.is_empty() {
        return emit(&Vec::<serde_json::Value>::new(), cli.output_or_json());
    }
    if candidates.len() > 1 {
        let payload = json!({
            "error": "ambiguous_stem",
            "command": "related",
            "query": args.stem,
            "count": candidates.len(),
            "message": "multiple documents matched this stem/id/path; use full id or path",
            "candidates": candidates,
        });
        return emit(&payload, cli.output_or_json());
    }
    let resolved = &candidates[0].path;
    let bounded_distance = args.max_distance.max(1);
    let bounded_limit = args.limit.max(1);
    if args.verbose {
        let (results, diagnostics) =
            index.related_with_diagnostics(resolved, bounded_distance, bounded_limit, ppr.as_ref());
        let phases = build_related_monitor_phases(diagnostics);
        let payload = json!({
            "stem": args.stem,
            "max_distance": bounded_distance,
            "limit": bounded_limit,
            "ppr": ppr,
            "diagnostics": diagnostics,
            "phases": phases,
            "total": results.len(),
            "results": results,
        });
        emit(&payload, cli.output_or_json())
    } else {
        let (results, _) =
            index.related_with_diagnostics(resolved, bounded_distance, bounded_limit, ppr.as_ref());
        emit(&results, cli.output_or_json())
    }
}
