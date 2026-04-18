mod command;
mod contract;
mod diagnostic;
mod discovery;
mod policy;
mod report;
mod run;

pub use command::{LintCommand, MarkdownLintArgs};
pub use contract::{
    MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS, MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID,
    MarkdownLintDiagnosticContractAssets, markdown_lint_diagnostic_contract_assets,
    markdown_lint_diagnostic_contract_snapshot, markdown_lint_diagnostic_schema_snapshot,
};
pub use report::{MarkdownLintFileReport, MarkdownLintIssue, MarkdownLintReport};
pub(crate) use run::run_markdown_lint;
