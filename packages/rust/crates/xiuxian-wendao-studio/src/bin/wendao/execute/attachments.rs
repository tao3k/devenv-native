//! Attachment search command execution.

use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::{Cli, Command};
use anyhow::{Context, Result};
use serde_json::json;
use xiuxian_wendao::{LinkGraphAttachmentKind, LinkGraphIndex};

pub(super) fn handle(cli: &Cli, index: Option<&LinkGraphIndex>) -> Result<()> {
    let Command::Attachments(args) = &cli.command else {
        unreachable!("attachments handler must be called with attachments command");
    };

    let index = index.context("link_graph index is required for attachments command")?;
    let attachment_kinds: Vec<LinkGraphAttachmentKind> =
        args.kinds.iter().copied().map(Into::into).collect();
    let search_query = args.query.clone().unwrap_or_default();
    let rows = index.search_attachments(
        &search_query,
        args.limit,
        &args.exts,
        &attachment_kinds,
        args.case_sensitive,
    );

    let payload = json!({
        "query": search_query,
        "limit": args.limit.max(1),
        "ext": args.exts,
        "kinds": attachment_kinds,
        "case_sensitive": args.case_sensitive,
        "total": rows.len(),
        "hits": rows,
    });
    emit(&payload, cli.output_or_json())
}
