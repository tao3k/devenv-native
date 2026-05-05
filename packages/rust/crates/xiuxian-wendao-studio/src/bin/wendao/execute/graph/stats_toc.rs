//! Stats and table-of-contents command handlers.

use crate::bin_support::wendao::helpers::emit;
use crate::bin_support::wendao::types::Cli;
use anyhow::{Context, Result};
use xiuxian_wendao::LinkGraphIndex;

pub(super) fn handle_stats(cli: &Cli, index: Option<&LinkGraphIndex>) -> Result<()> {
    let index = index.context("link_graph index is required for stats command")?;
    emit(&index.stats(), cli.output_or_json())
}

pub(super) fn handle_toc(cli: &Cli, index: Option<&LinkGraphIndex>, limit: usize) -> Result<()> {
    let index = index.context("link_graph index is required for toc command")?;
    emit(&index.toc(limit.max(1)), cli.output_or_json())
}
