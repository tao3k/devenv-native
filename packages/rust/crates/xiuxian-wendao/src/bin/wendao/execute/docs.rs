use std::env;

use anyhow::{Context, Result};
use xiuxian_wendao::analyzers::{
    DocsNavigationOptions, DocsRetrievalContextOptions, DocsToolService,
};

use crate::helpers::emit;
use crate::types::{Cli, Command, DocsCommand};

pub(super) fn handle(cli: &Cli) -> Result<()> {
    let Command::Docs { command } = &cli.command else {
        unreachable!("docs handler called with non-docs command");
    };
    let context = DocsCommandContext::new(cli)?;
    match command {
        DocsCommand::Page(args) => handle_page(&context, args),
        DocsCommand::Tree(args) => handle_tree(&context, args),
        DocsCommand::TreeOutline(args) => handle_tree_outline(&context, args),
        DocsCommand::StructureCatalog(args) => handle_structure_catalog(&context, args),
        DocsCommand::Segment(args) => handle_segment(&context, args),
        DocsCommand::SearchStructure(args) => handle_search_structure(&context, args),
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

fn handle_page(context: &DocsCommandContext<'_>, args: &crate::types::DocsPageArgs) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_document(&args.page_id)
        .with_context(|| format!("failed to open docs page `{}`", args.page_id))?;
    emit(&result, context.cli.output)
}

fn handle_tree(context: &DocsCommandContext<'_>, args: &crate::types::DocsTreeArgs) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_document_structure(&args.page_id)
        .with_context(|| {
            format!(
                "failed to open docs page-index tree for page `{}`",
                args.page_id
            )
        })?;
    emit(&result, context.cli.output)
}

fn handle_tree_outline(
    context: &DocsCommandContext<'_>,
    args: &crate::types::DocsTreeOutlineArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_document_structure_outline(&args.page_id)
        .with_context(|| {
            format!(
                "failed to open docs text-free page-index tree for page `{}`",
                args.page_id
            )
        })?;
    emit(&result, context.cli.output)
}

fn handle_structure_catalog(
    context: &DocsCommandContext<'_>,
    args: &crate::types::DocsStructureCatalogArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_document_structure_catalog()
        .with_context(|| {
            format!(
                "failed to open docs text-free structure catalog for repo `{}`",
                args.repo
            )
        })?;
    emit(&result, context.cli.output)
}

fn handle_segment(
    context: &DocsCommandContext<'_>,
    args: &crate::types::DocsSegmentArgs,
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
    emit(&result, context.cli.output)
}

fn handle_search_structure(
    context: &DocsCommandContext<'_>,
    args: &crate::types::DocsSearchStructureArgs,
) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .search_document_structure(&args.query, args.kind.map(Into::into), args.limit)
        .with_context(|| {
            format!(
                "failed to search docs page-index structure for query `{}`",
                args.query
            )
        })?;
    emit(&result, context.cli.output)
}

fn handle_node(context: &DocsCommandContext<'_>, args: &crate::types::DocsNodeArgs) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_document_node(&args.page_id, &args.node_id)
        .with_context(|| {
            format!(
                "failed to open docs page-index node `{}` for page `{}`",
                args.node_id, args.page_id
            )
        })?;
    emit(&result, context.cli.output)
}

fn handle_toc(context: &DocsCommandContext<'_>, args: &crate::types::DocsTocArgs) -> Result<()> {
    let result = context
        .service(args.repo.clone())
        .get_toc_documents()
        .with_context(|| {
            format!(
                "failed to open docs markdown TOC documents for repo `{}`",
                args.repo
            )
        })?;
    emit(&result, context.cli.output)
}

fn handle_navigation(
    context: &DocsCommandContext<'_>,
    args: &crate::types::DocsNavigationArgs,
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
    emit(&result, context.cli.output)
}

fn handle_context(
    context: &DocsCommandContext<'_>,
    args: &crate::types::DocsContextArgs,
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
    emit(&result, context.cli.output)
}

#[cfg(test)]
#[path = "../../../../tests/unit/bin/wendao/execute/docs.rs"]
mod tests;
