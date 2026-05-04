//! Embedded contract identifiers and snapshots for markdown lint diagnostics.

use std::fmt;

/// Stable markdown lint diagnostic contract identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MarkdownLintDiagnosticContractId(&'static str);

impl MarkdownLintDiagnosticContractId {
    /// Create a checked-in markdown lint diagnostic contract id.
    #[must_use]
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    /// Return the contract id as a string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl AsRef<str> for MarkdownLintDiagnosticContractId {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl fmt::Display for MarkdownLintDiagnosticContractId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl PartialEq<&str> for MarkdownLintDiagnosticContractId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Stable contract identifier for markdown lint diagnostic rendering.
pub const MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID: MarkdownLintDiagnosticContractId =
    MarkdownLintDiagnosticContractId::new("wendao.markdown_lint.diagnostics");
/// Ordered markdown lint diagnostic contracts exposed by the lightweight client.
pub const MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS: &[MarkdownLintDiagnosticContractId] =
    &[MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID];

#[cfg(test)]
const MARKDOWN_LINT_DIAGNOSTICS_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/manifests/wendao.markdown_lint.diagnostics.toml"
));
const MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.markdown_lint.diagnostics/contract.toml"
));
const MARKDOWN_LINT_DIAGNOSTICS_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.markdown_lint.diagnostics/schema.json"
));

/// Raw checked-in assets for one markdown lint diagnostic contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkdownLintDiagnosticContractAssets {
    /// Checked-in normalized `contract.toml` content.
    pub contract_toml: &'static str,
    /// Checked-in `schema.json` content for `MarkdownLintReport`.
    pub schema_json: &'static str,
}

/// Resolve the raw checked-in assets for one markdown lint contract id.
#[must_use]
pub fn markdown_lint_diagnostic_contract_assets(
    contract_id: MarkdownLintDiagnosticContractId,
) -> Option<MarkdownLintDiagnosticContractAssets> {
    match contract_id {
        MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID => Some(MarkdownLintDiagnosticContractAssets {
            contract_toml: MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_TOML,
            schema_json: MARKDOWN_LINT_DIAGNOSTICS_SCHEMA_JSON,
        }),
        _ => None,
    }
}

/// Resolve the raw checked-in `contract.toml` for one markdown lint contract id.
#[must_use]
pub fn markdown_lint_diagnostic_contract_snapshot(
    contract_id: MarkdownLintDiagnosticContractId,
) -> Option<&'static str> {
    markdown_lint_diagnostic_contract_assets(contract_id).map(|assets| assets.contract_toml)
}

/// Resolve the raw checked-in `schema.json` for one markdown lint contract id.
#[must_use]
pub fn markdown_lint_diagnostic_schema_snapshot(
    contract_id: MarkdownLintDiagnosticContractId,
) -> Option<&'static str> {
    markdown_lint_diagnostic_contract_assets(contract_id).map(|assets| assets.schema_json)
}

#[cfg(test)]
pub(super) fn markdown_lint_diagnostic_manifest(
    contract_id: MarkdownLintDiagnosticContractId,
) -> Option<&'static str> {
    match contract_id {
        MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID => Some(MARKDOWN_LINT_DIAGNOSTICS_MANIFEST),
        _ => None,
    }
}
