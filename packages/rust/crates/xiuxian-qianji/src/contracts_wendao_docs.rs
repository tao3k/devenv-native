//! Wendao docs invocation-contract helpers.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::error::QianjiError;
use crate::markdown::{MarkdownShowSection, render_show_surface};
use xiuxian_wendao::analyzers::{DOCS_CONTRACT_IDS, docs_capability_contract_assets};

#[path = "contracts_wendao_docs/contract.rs"]
mod contract;

use self::contract::WendaoDocsContract;

/// One display-ready Wendao docs invocation contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoDocsContractShow {
    /// Stable contract id selected at the CLI.
    pub contract_id: String,
    /// Raw `contract.toml` snapshot.
    pub contract_toml: String,
    /// Raw `schema.json` snapshot.
    pub schema_json: String,
}

/// Resolve one Wendao docs invocation contract for bounded display.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the requested contract id is unknown
/// or the checked-in snapshots cannot be parsed.
pub fn show_wendao_docs_contract(
    contract_id: impl AsRef<str>,
) -> Result<WendaoDocsContractShow, QianjiError> {
    let contract_id = contract_id.as_ref();
    let assets = docs_capability_contract_assets(contract_id).ok_or_else(|| {
        QianjiError::Topology(format!(
            "unknown Wendao docs contract `{contract_id}`; supported contracts: {}",
            DOCS_CONTRACT_IDS.join(", ")
        ))
    })?;
    load_wendao_docs_contract(contract_id)?;
    Ok(WendaoDocsContractShow {
        contract_id: contract_id.to_string(),
        contract_toml: assets.contract_toml.to_string(),
        schema_json: assets.schema_json.to_string(),
    })
}

/// Render one Wendao docs contract into markdown.
#[must_use]
pub fn render_wendao_docs_contract_show(show: &WendaoDocsContractShow) -> String {
    render_show_surface(
        "Contract",
        &[
            format!("Name: {}", show.contract_id),
            "Kind: wendao-docs-invocation-contract".to_string(),
        ],
        &[
            render_code_section("Contract TOML", "toml", show.contract_toml.as_str()),
            render_code_section("Schema JSON", "json", show.schema_json.as_str()),
        ],
    )
}

pub(crate) fn load_wendao_docs_contract(
    contract_id: &str,
) -> Result<WendaoDocsContract, QianjiError> {
    contract::load_wendao_docs_contract(contract_id)
}

pub(crate) fn validate_http_call(
    contract: &WendaoDocsContract,
    method: &str,
    path: &str,
    query: &BTreeMap<String, Value>,
) -> Result<(), QianjiError> {
    contract.validate_http_call(method, path, query)
}

pub(crate) fn validate_cli_call(
    contract: &WendaoDocsContract,
    argv: &[String],
) -> Result<(), QianjiError> {
    contract.validate_cli_call(argv)
}

fn render_code_section<'a>(title: &'a str, lang: &str, raw: &'a str) -> MarkdownShowSection<'a> {
    let mut lines = vec![format!("```{lang}")];
    lines.extend(raw.lines().map(ToString::to_string));
    lines.push("```".to_string());
    MarkdownShowSection {
        title: title.into(),
        lines,
    }
}
