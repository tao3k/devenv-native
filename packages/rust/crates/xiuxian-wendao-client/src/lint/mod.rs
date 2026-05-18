//! Markdown lint command surface, diagnostics, and contract assets.

mod command;
mod contract;
mod diagnostic;
mod discovery;
#[cfg(feature = "semantic-sql")]
mod lifecycle;
mod policy;
#[cfg(feature = "semantic-sql")]
mod projection_policy;
mod report;
mod run;
mod text_output;

pub use command::{LintCommand, MarkdownLintArgs};
#[cfg(feature = "semantic-sql")]
pub use command::{
    SemanticLintArgs, SemanticLintProjectionValidationArgs, SemanticLintValidationArgs,
    SemanticLintWritebackArgs,
};
pub use contract::{
    MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS, MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID,
    MarkdownLintDiagnosticContractAssets, MarkdownLintDiagnosticContractId,
    markdown_lint_diagnostic_contract_assets, markdown_lint_diagnostic_contract_snapshot,
    markdown_lint_diagnostic_schema_snapshot,
};
pub use report::{MarkdownLintFileReport, MarkdownLintIssue, MarkdownLintReport};
pub(crate) use run::run_markdown_lint;
#[cfg(feature = "semantic-sql")]
pub(crate) use run::run_semantic_lint;
