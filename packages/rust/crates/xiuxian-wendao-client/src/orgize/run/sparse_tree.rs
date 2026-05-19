//! Sparse-tree `orgize` command execution.

use anyhow::Result;
use xiuxian_wendao_parsers::{
    OrgizeSparseTreeRenderOptions, OrgizeSparseTreeRequest, OrgizeSparseTreeVisibility,
    render_sparse_tree,
};

use crate::orgize::OrgizeSparseTreeArgs;
use crate::{ClientContext, CommandOutcome};

use super::paths::resolve_paths;

pub(super) fn run_sparse_tree(
    args: &OrgizeSparseTreeArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let rendered = render_sparse_tree(&OrgizeSparseTreeRequest {
        paths: resolve_paths(&args.paths, context),
        text: args.text.clone(),
        match_expression: args.match_expression.clone(),
        visibility: OrgizeSparseTreeVisibility {
            exclude_done: args.visibility.exclude_done,
            exclude_archived: args.visibility.exclude_archived,
        },
        include_comments: args.visibility.include_comments,
        render: OrgizeSparseTreeRenderOptions {
            explain_skips: args.render.explain_skips,
        },
    })?;
    print!("{rendered}");
    Ok(CommandOutcome::success())
}
