//! Lightweight Wendao client CLI surfaces for local document tooling.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

mod cli;
mod context;
mod execute;
mod get;
mod lint;
mod output;

pub use cli::{ClientCli, ClientCommand};
pub use context::ClientContext;
pub use execute::{CommandOutcome, run_command};
#[cfg(feature = "performance")]
pub use get::perf_support;
pub use get::{
    DocsPageIndexDocumentsResult, DocsPageIndexTreesResult, GetCommand, GetScopeArgs,
    ProjectedPageIndexDocument, ProjectedPageIndexLink, ProjectedPageIndexNode,
    ProjectedPageIndexSection, ProjectedPageIndexTree, ProjectionPageKind,
};
pub use lint::{
    LintCommand, MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS, MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID,
    MarkdownLintArgs, MarkdownLintDiagnosticContractAssets, MarkdownLintFileReport,
    MarkdownLintIssue, MarkdownLintReport, markdown_lint_diagnostic_contract_assets,
    markdown_lint_diagnostic_contract_snapshot, markdown_lint_diagnostic_schema_snapshot,
};
pub use output::OutputFormat;
