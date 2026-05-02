//! Contract-backed diagnostic rendering assets for markdown lint output.

mod assets;
mod loader;
mod manifest;
mod rule;
mod schema;
mod snapshot;
mod strategy;
mod validation;

pub use assets::{
    MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS, MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID,
    MarkdownLintDiagnosticContractAssets, markdown_lint_diagnostic_contract_assets,
    markdown_lint_diagnostic_contract_snapshot, markdown_lint_diagnostic_schema_snapshot,
};
pub(in crate::lint) use loader::diagnostic_contract;

#[cfg(test)]
use manifest::{
    contract_snapshot_path, generate_snapshot_contract_toml, parse_manifest, schema_snapshot_path,
    snapshot_root_path,
};
#[cfg(test)]
use schema::generate_schema_json;
#[cfg(test)]
use snapshot::MarkdownLintDiagnosticContractSnapshot;

#[cfg(test)]
#[path = "../../../tests/unit/contract.rs"]
mod tests;
