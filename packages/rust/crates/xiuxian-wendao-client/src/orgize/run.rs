//! Orgize-backed client command execution.

#[cfg(feature = "orgize-agent-read-model")]
use crate::orgize::read_model::{run_read_model, run_task_archive, run_task_list, run_task_report};
use crate::orgize::{
    OrgizeAgentPlanningArgs, OrgizeCommand, OrgizeFormatArgs, OrgizeLintArgs, OrgizeLintFormatArg,
    OrgizeSparseTreeArgs,
};
use crate::{ClientContext, CommandOutcome};
use anyhow::Result;
use std::path::PathBuf;
use xiuxian_wendao_parsers::{
    OrgizeAgentPlanningRequest, OrgizeFormatRequest, OrgizeLintOutputFormat, OrgizeLintRequest,
    OrgizeSparseTreeRenderOptions, OrgizeSparseTreeRequest, OrgizeSparseTreeVisibility,
    format_org_files, lint_org_files, render_agent_planning, render_sparse_tree,
};

/// Run one Orgize-backed client command.
///
/// # Errors
///
/// Returns an error when the selected Orgize operation fails.
pub(crate) fn run_command(
    command: &OrgizeCommand,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    match command {
        OrgizeCommand::Fmt(args) => run_format(args, context),
        OrgizeCommand::Lint(args) => run_lint(args, context),
        OrgizeCommand::AgentPlanning(args) => run_agent_planning(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::ReadModel(args) => run_read_model(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskList(args) => run_task_list(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskReport(args) => run_task_report(args, context),
        #[cfg(feature = "orgize-agent-read-model")]
        OrgizeCommand::TaskArchive(args) => run_task_archive(args, context),
        OrgizeCommand::SparseTree(args) => run_sparse_tree(args, context),
    }
}

fn run_format(args: &OrgizeFormatArgs, context: &ClientContext) -> Result<CommandOutcome> {
    let report = format_org_files(&OrgizeFormatRequest {
        paths: resolve_paths(&args.paths, context),
        check: args.check,
    })?;
    if args.check {
        for path in &report.changed_paths {
            eprintln!("{}: needs formatting", display_path(path, context));
        }
    }
    Ok(if args.check && report.changed() {
        CommandOutcome::failure(1)
    } else {
        CommandOutcome::success()
    })
}

fn run_lint(args: &OrgizeLintArgs, context: &ClientContext) -> Result<CommandOutcome> {
    let output_format = if args.json {
        OrgizeLintOutputFormat::Json
    } else {
        lint_output_format(args.format)
    };
    let report = lint_org_files(&OrgizeLintRequest {
        paths: resolve_paths(&args.paths, context),
        output_format,
        priority_highest: args.priority_highest.clone(),
        priority_lowest: args.priority_lowest.clone(),
        priority_default: args.priority_default.clone(),
    })?;
    print!("{}", report.render(output_format));
    Ok(if report.is_clean() {
        CommandOutcome::success()
    } else {
        CommandOutcome::failure(1)
    })
}

fn run_agent_planning(
    args: &OrgizeAgentPlanningArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let rendered = render_agent_planning(&OrgizeAgentPlanningRequest {
        paths: resolve_paths(&args.paths, context),
        start_date: args.date.clone(),
        end_date: args.end.clone(),
        include_done: args.include_done,
        include_archived: args.include_archived,
        include_comments: args.include_comments,
        match_expression: args.match_expression.clone(),
    })?;
    print!("{rendered}");
    Ok(CommandOutcome::success())
}

fn run_sparse_tree(args: &OrgizeSparseTreeArgs, context: &ClientContext) -> Result<CommandOutcome> {
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

fn resolve_paths(paths: &[PathBuf], context: &ClientContext) -> Vec<PathBuf> {
    let paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };
    paths
        .into_iter()
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                context.root().join(path)
            }
        })
        .collect()
}

fn display_path(path: &std::path::Path, context: &ClientContext) -> String {
    path.strip_prefix(context.root()).map_or_else(
        |_| path.display().to_string(),
        |path| path.display().to_string(),
    )
}

fn lint_output_format(format: OrgizeLintFormatArg) -> OrgizeLintOutputFormat {
    match format {
        OrgizeLintFormatArg::Compact => OrgizeLintOutputFormat::Compact,
        OrgizeLintFormatArg::Text => OrgizeLintOutputFormat::Text,
        OrgizeLintFormatArg::Json => OrgizeLintOutputFormat::Json,
    }
}
