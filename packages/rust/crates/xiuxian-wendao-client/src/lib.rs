//! Lightweight Wendao client CLI surfaces for local document tooling.

mod cli;
mod context;
mod execute;
mod get;
mod lint;
mod output;
#[cfg(feature = "semantic-sql")]
mod semantic;

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
    MarkdownLintArgs, MarkdownLintDiagnosticContractAssets, MarkdownLintDiagnosticContractId,
    MarkdownLintFileReport, MarkdownLintIssue, MarkdownLintReport,
    markdown_lint_diagnostic_contract_assets, markdown_lint_diagnostic_contract_snapshot,
    markdown_lint_diagnostic_schema_snapshot,
};
#[cfg(feature = "semantic-sql")]
pub use lint::{
    SemanticLintArgs, SemanticLintProjectionValidationArgs, SemanticLintValidationArgs,
    SemanticLintWritebackArgs,
};
pub use output::OutputFormat;
#[cfg(feature = "semantic-sql")]
pub use semantic::{
    SemanticCheckReadModelSnapshotArgs, SemanticCommand, SemanticDescribeReadModelArgs,
    SemanticPlanReadModelMaterializationArgs, SemanticPreflightReadModelMaterializationArgs,
    SemanticReadModelQueryArgs, SemanticRefreshProjectionsArgs, SemanticSnapshotReadModelArgs,
};
