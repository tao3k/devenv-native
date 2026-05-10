use std::env;

use anyhow::{Context, Result};
use xiuxian_wendao::analyzers::{
    DocsNavigationOptions, DocsRetrievalContextOptions, DocsToolService,
};

use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::{Cli, Command, DocsCommand};

pub(super) fn handle(cli: &Cli) -> Result<()> {
    let Command::Docs { command } = &cli.command else {
        unreachable!("docs handler called with non-docs command");
    };
    let context = DocsCommandContext::new(cli)?;
    match command {
        DocsCommand::Page(args) => handle_page(&context, args),
        DocsCommand::Tree(args) => handle_tree(&context, args),
        DocsCommand::PageIndexOutline(args) => handle_page_index_outline(&context, args),
        DocsCommand::PageIndex(args) => handle_page_index(&context, args),
        DocsCommand::Segment(args) => handle_segment(&context, args),
        DocsCommand::Search(args) => handle_search(&context, args),
        DocsCommand::SearchPageIndex(args) => handle_search_page_index(&context, args),
        DocsCommand::Node(args) => handle_node(&context, args),
        DocsCommand::Toc(args) => handle_toc(&context, args),
        DocsCommand::Navigation(args) => handle_navigation(&context, args),
        DocsCommand::Context(args) => handle_context(&context, args),
    }
}

struct DocsCommandContext<'a> {
    cli: &'a Cli,
    cwd: std::path::PathBuf,
}

impl<'a> DocsCommandContext<'a> {
    fn new(cli: &'a Cli) -> Result<Self> {
        Ok(Self {
            cli,
            cwd: env::current_dir()?,
        })
    }

    fn service(&self, repo: String) -> DocsToolService {
        DocsToolService::from_project_root(self.cwd.clone(), repo)
            .with_optional_config_path(self.cli.config_file.clone())
    }
}

fn handle_page(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsPageArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_document(&args.page_id)
        .with_context(|| format!("failed to open docs page `{}`", args.page_id))?;
    emit(&result, context.cli.output_or_json())
}

fn handle_tree(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsTreeArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_page_index_tree(&args.page_id)
        .with_context(|| {
            format!(
                "failed to open docs page-index tree for page `{}`",
                args.page_id
            )
        })?;
    emit(&result, context.cli.output_or_json())
}

fn handle_page_index_outline(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsPageIndexOutlineArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_page_index_outline(&args.page_id)
        .with_context(|| {
            format!(
                "failed to open docs text-free page-index tree for page `{}`",
                args.page_id
            )
        })?;
    emit(&result, context.cli.output_or_json())
}

fn handle_page_index(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsPageIndexArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_page_index()
        .with_context(|| {
            format!(
                "failed to open docs text-free page-index catalog for repo `{}`",
                args.repo
            )
        })?;
    emit(&result, context.cli.output_or_json())
}

fn handle_segment(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsSegmentArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_document_segment(&args.page_id, args.line_start, args.line_end)
        .with_context(|| {
            format!(
                "failed to open docs segment {}-{} for page `{}`",
                args.line_start, args.line_end, args.page_id
            )
        })?;
    emit(&result, context.cli.output_or_json())
}

fn handle_search(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsSearchArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .search_documents(&args.query, args.kind.map(Into::into), args.limit)
        .with_context(|| format!("failed to search docs pages for query `{}`", args.query))?;
    emit(&result, context.cli.output_or_json())
}

fn handle_search_page_index(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsSearchPageIndexArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .search_page_index(&args.query, args.kind.map(Into::into), args.limit)
        .with_context(|| {
            format!(
                "failed to search docs page-index for query `{}`",
                args.query
            )
        })?;
    emit(&result, context.cli.output_or_json())
}

fn handle_node(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsNodeArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_document_node(&args.page_id, &args.node_id)
        .with_context(|| {
            format!(
                "failed to open docs page-index node `{}` for page `{}`",
                args.node_id, args.page_id
            )
        })?;
    emit(&result, context.cli.output_or_json())
}

fn handle_toc(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsTocArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_toc_documents()
        .with_context(|| {
            format!(
                "failed to open docs markdown TOC documents for repo `{}`",
                args.repo
            )
        })?;
    emit(&result, context.cli.output_or_json())
}

fn handle_navigation(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsNavigationArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_navigation_with_options(
            &args.page_id,
            DocsNavigationOptions {
                node_id: args.node_id.clone(),
                family_kind: args.family_kind.map(Into::into),
                related_limit: args.related_limit,
                family_limit: args.family_limit,
            },
        )
        .with_context(|| {
            format!(
                "failed to open docs navigation bundle for page `{}`",
                args.page_id
            )
        })?;
    emit(&result, context.cli.output_or_json())
}

fn handle_context(
    context: &DocsCommandContext<'_>,
    args: &crate::bin_support::wendao::types::DocsContextArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_retrieval_context_with_options(
            &args.page_id,
            DocsRetrievalContextOptions {
                node_id: args.node_id.clone(),
                related_limit: args.related_limit,
            },
        )
        .with_context(|| {
            format!(
                "failed to open docs retrieval context for page `{}`",
                args.page_id
            )
        })?;
    emit(&result, context.cli.output_or_json())
}

#[cfg(test)]
#[path = "../../../../tests/unit/bin/wendao/execute/docs.rs"]
mod tests;
