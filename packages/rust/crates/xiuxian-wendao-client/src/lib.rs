//! Lightweight Wendao client CLI surfaces for local document tooling.

#[cfg(test)]
#[path = "../tests/unit/lib_policy.rs"]
mod rust_project_harness_gate;

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = rust_project_harness_gate::client_rust_harness_config()
);

mod cli;
mod context;
mod execute;
mod get;
mod lint;
mod orgize;
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
#[cfg(feature = "orgize-agent-read-model")]
pub use orgize::OrgizeOgridShowArgs;
#[cfg(feature = "orgize-agent-read-model")]
pub use orgize::OrgizeReadModelArgs;
#[cfg(feature = "orgize-agent-read-model")]
pub use orgize::OrgizeTaskProbeArgs;
#[cfg(feature = "orgize-agent-read-model")]
pub use orgize::OrgizeTaskRecoverArgs;
#[cfg(feature = "orgize-agent-read-model")]
pub use orgize::OrgizeTaskSddArgs;
#[cfg(all(feature = "performance", feature = "orgize-agent-read-model"))]
pub use orgize::perf_support as orgize_perf_support;
pub use orgize::{
    OrgizeAgentPlanningArgs, OrgizeCommand, OrgizeEvalCommand, OrgizeFormatArgs, OrgizeLintArgs,
    OrgizeLintFormatArg, OrgizeSddCommand, OrgizeSddGraphDiffArgs, OrgizeSddStatusArgs,
    OrgizeSparseTreeArgs,
};
pub use output::OutputFormat;
#[cfg(feature = "semantic-sql")]
pub use semantic::{
    SemanticCheckReadModelSnapshotArgs, SemanticCommand, SemanticDescribeReadModelArgs,
    SemanticPlanReadModelMaterializationArgs, SemanticPreflightReadModelMaterializationArgs,
    SemanticReadModelQueryArgs, SemanticRefreshProjectionsArgs, SemanticSnapshotReadModelArgs,
};
