//! Lightweight Wendao client CLI surfaces for local document tooling.

xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");

mod cli;
mod context;
mod execute;
mod lint;
mod output;

pub use cli::{ClientCli, ClientCommand};
pub use context::ClientContext;
pub use execute::{CommandOutcome, run_command};
pub use lint::{
    LintCommand, MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS, MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID,
    MarkdownLintArgs, MarkdownLintDiagnosticContractAssets, MarkdownLintFileReport,
    MarkdownLintIssue, MarkdownLintReport, markdown_lint_diagnostic_contract_assets,
    markdown_lint_diagnostic_contract_snapshot, markdown_lint_diagnostic_schema_snapshot,
};
pub use output::OutputFormat;
