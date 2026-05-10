//! Metadata and alias resolution command handlers.

use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::Cli;
use anyhow::{Context, Result};
use serde_json::json;
use xiuxian_wendao::LinkGraphIndex;

pub(super) fn handle_metadata(cli: &Cli, index: Option<&LinkGraphIndex>, stem: &str) -> Result<()> {
    let index = index.context("link_graph index is required for metadata command")?;
    let candidates = index.resolve_metadata_candidates(stem);
    match candidates.len() {
        0 => emit(
            &Option::<xiuxian_wendao::LinkGraphMetadata>::None,
            cli.output_or_json(),
        ),
        1 => {
            let resolved = candidates
                .into_iter()
                .next()
                .context("metadata candidate unexpectedly missing")?;
            emit(&index.metadata(&resolved.path), cli.output_or_json())
        }
        _ => {
            let payload = json!({
                "error": "ambiguous_stem",
                "query": stem,
                "count": candidates.len(),
                "message": "multiple documents matched this stem/id/path; use full id or path",
                "candidates": candidates,
            });
            emit(&payload, cli.output_or_json())
        }
    }
}

pub(super) fn handle_resolve(
    cli: &Cli,
    index: Option<&LinkGraphIndex>,
    alias: &str,
    limit: usize,
) -> Result<()> {
    let index = index.context("link_graph index is required for resolve command")?;
    let candidates = index.resolve_metadata_candidates(alias);
    let bounded_limit = limit.max(1);
    let results: Vec<_> = candidates.into_iter().take(bounded_limit).collect();
    emit(&results, cli.output_or_json())
}
