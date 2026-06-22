use std::collections::BTreeMap;

use crate::contracts::NodeDefinition;
#[cfg(feature = "wendao-integration")]
use crate::contracts::{load_wendao_docs_contract, validate_cli_call, validate_http_call};
use crate::error::QianjiError;
use serde_json::Value;

pub(super) struct CommandMechanismConfig {
    pub(super) cmd: String,
    pub(super) output_key: String,
    pub(super) allow_fail: bool,
    pub(super) stop_on_empty_stdout: bool,
    pub(super) empty_reason: Option<String>,
}

pub(super) struct WriteFileMechanismConfig {
    pub(super) path: String,
    pub(super) content: String,
    pub(super) output_key: String,
}

pub(super) struct SuspendMechanismConfig {
    pub(super) reason: String,
    pub(super) prompt: String,
    pub(super) resume_key: Option<String>,
}

pub(super) struct HttpCallMechanismConfig {
    pub(super) contract: String,
    pub(super) method: String,
    pub(super) path: String,
    pub(super) base_url: Option<String>,
    pub(super) query: BTreeMap<String, Value>,
    pub(super) output_key: String,
}

pub(super) struct CliCallMechanismConfig {
    pub(super) contract: String,
    pub(super) argv: Vec<String>,
    pub(super) output_key: String,
}

pub(super) fn command_mechanism_config(node_def: &NodeDefinition) -> CommandMechanismConfig {
    CommandMechanismConfig {
        cmd: string_param(node_def, "cmd").unwrap_or_default(),
        output_key: string_param(node_def, "output_key").unwrap_or_else(|| "stdout".to_string()),
        allow_fail: bool_param(node_def, "allow_fail", false),
        stop_on_empty_stdout: bool_param(node_def, "stop_on_empty_stdout", false),
        empty_reason: string_param(node_def, "empty_reason"),
    }
}

pub(super) fn write_file_mechanism_config(node_def: &NodeDefinition) -> WriteFileMechanismConfig {
    WriteFileMechanismConfig {
        path: string_param(node_def, "path")
            .or_else(|| string_param(node_def, "target_path"))
            .unwrap_or_default(),
        content: string_param(node_def, "content").unwrap_or_default(),
        output_key: string_param(node_def, "output_key")
            .unwrap_or_else(|| "write_file_result".to_string()),
    }
}

pub(super) fn suspend_mechanism_config(node_def: &NodeDefinition) -> SuspendMechanismConfig {
    SuspendMechanismConfig {
        reason: string_param(node_def, "reason").unwrap_or_else(|| "suspended".to_string()),
        prompt: string_param(node_def, "prompt")
            .unwrap_or_else(|| "Waiting for input...".to_string()),
        resume_key: optional_string_param(node_def, "resume_key"),
    }
}

pub(super) fn http_call_mechanism_config(
    node_def: &NodeDefinition,
) -> Result<HttpCallMechanismConfig, QianjiError> {
    #[cfg(not(feature = "wendao-integration"))]
    {
        let _ = node_def;
        Err(wendao_contract_validation_disabled("http_call"))
    }
    #[cfg(feature = "wendao-integration")]
    {
        let contract = required_top_level_string(
            node_def.contract.as_deref(),
            node_def.id.as_str(),
            "contract",
        )?;
        let method =
            required_top_level_string(node_def.method.as_deref(), node_def.id.as_str(), "method")?;
        let path =
            required_top_level_string(node_def.path.as_deref(), node_def.id.as_str(), "path")?;
        let query = node_def.query.clone().ok_or_else(|| {
            QianjiError::Topology(format!(
                "node `{}` kind `http_call` requires `query`",
                node_def.id
            ))
        })?;
        let loaded_contract = load_wendao_docs_contract(contract.as_str())?;
        validate_http_call(&loaded_contract, &method, &path, &query)?;

        Ok(HttpCallMechanismConfig {
            contract,
            method,
            path,
            base_url: optional_top_level_string(node_def.base_url.as_deref()),
            query,
            output_key: node_def.id.clone(),
        })
    }
}

pub(super) fn cli_call_mechanism_config(
    node_def: &NodeDefinition,
) -> Result<CliCallMechanismConfig, QianjiError> {
    #[cfg(not(feature = "wendao-integration"))]
    {
        let _ = node_def;
        Err(wendao_contract_validation_disabled("cli_call"))
    }
    #[cfg(feature = "wendao-integration")]
    {
        let contract = required_top_level_string(
            node_def.contract.as_deref(),
            node_def.id.as_str(),
            "contract",
        )?;
        let argv = node_def.argv.clone().ok_or_else(|| {
            QianjiError::Topology(format!(
                "node `{}` kind `cli_call` requires `argv`",
                node_def.id
            ))
        })?;
        let loaded_contract = load_wendao_docs_contract(contract.as_str())?;
        validate_cli_call(&loaded_contract, &argv)?;

        Ok(CliCallMechanismConfig {
            contract,
            argv,
            output_key: node_def.id.clone(),
        })
    }
}

#[cfg(not(feature = "wendao-integration"))]
fn wendao_contract_validation_disabled(kind: &str) -> QianjiError {
    QianjiError::Topology(format!(
        "node kind `{kind}` requires the `wendao-integration` feature for Wendao docs contract validation"
    ))
}

fn bool_param(node_def: &NodeDefinition, key: &str, default: bool) -> bool {
    node_def
        .params
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(default)
}

fn string_param(node_def: &NodeDefinition, key: &str) -> Option<String> {
    node_def
        .params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn optional_string_param(node_def: &NodeDefinition, key: &str) -> Option<String> {
    node_def
        .params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

#[cfg(feature = "wendao-integration")]
fn required_top_level_string(
    value: Option<&str>,
    node_id: &str,
    field: &str,
) -> Result<String, QianjiError> {
    optional_top_level_string(value).ok_or_else(|| {
        QianjiError::Topology(format!("node `{node_id}` requires top-level `{field}`"))
    })
}

#[cfg(feature = "wendao-integration")]
fn optional_top_level_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}
