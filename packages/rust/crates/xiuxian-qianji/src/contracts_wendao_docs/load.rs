use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::error::QianjiError;
use xiuxian_wendao::analyzers::{DOCS_CONTRACT_IDS, docs_capability_contract_assets};

use super::WendaoDocsContract;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct WendaoDocsContractSnapshot {
    pub(super) id: String,
    pub(super) version: u32,
    pub(super) task_types: Vec<String>,
    pub(super) http: WendaoDocsHttpSurface,
    pub(super) cli: WendaoDocsCliSurface,
    pub(super) tool: WendaoDocsToolSurface,
    pub(super) params: Vec<WendaoDocsParam>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct WendaoDocsHttpSurface {
    pub(super) method: String,
    pub(super) path: String,
    pub(super) query: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct WendaoDocsCliSurface {
    pub(super) argv: Vec<String>,
    pub(super) flags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct WendaoDocsToolSurface {
    pub(super) name: String,
    pub(super) schema: String,
    #[serde(default)]
    pub(super) runtime_injected: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct WendaoDocsParam {
    pub(super) name: String,
    #[serde(rename = "type")]
    pub(super) value_type: String,
    #[serde(default)]
    pub(super) required: bool,
}

pub(super) fn load_wendao_docs_contract(
    contract_id: &str,
) -> Result<WendaoDocsContract, QianjiError> {
    let assets = docs_capability_contract_assets(contract_id).ok_or_else(|| {
        QianjiError::Topology(format!(
            "unknown Wendao docs contract `{contract_id}`; supported contracts: {}",
            DOCS_CONTRACT_IDS.join(", ")
        ))
    })?;
    let snapshot: WendaoDocsContractSnapshot =
        toml::from_str(assets.contract_toml).map_err(|error| {
            QianjiError::Topology(format!(
                "failed to parse Wendao contract snapshot `{contract_id}`: {error}"
            ))
        })?;
    let schema_json: Value = serde_json::from_str(assets.schema_json).map_err(|error| {
        QianjiError::Topology(format!(
            "failed to parse Wendao schema snapshot `{contract_id}`: {error}"
        ))
    })?;
    Ok(WendaoDocsContract {
        snapshot,
        schema_json,
    })
}
