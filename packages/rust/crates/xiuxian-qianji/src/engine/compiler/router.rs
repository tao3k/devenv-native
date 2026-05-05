use crate::contracts::NodeDefinition;
use crate::error::QianjiError;
use serde_json::Value;

const DEFAULT_SEMANTIC_GUARD_ROUTE_KEY: &str = "semanticScopeGuardRoute";

pub(super) struct RouterConfig {
    pub(super) branches: Vec<(String, f32)>,
    pub(super) semantic_guard_route_key: Option<String>,
}

pub(super) fn config(node_def: &NodeDefinition) -> Result<RouterConfig, QianjiError> {
    Ok(RouterConfig {
        branches: branches(node_def)?,
        semantic_guard_route_key: semantic_guard_route_key(node_def)?,
    })
}

fn branches(node_def: &NodeDefinition) -> Result<Vec<(String, f32)>, QianjiError> {
    let mut branches = Vec::new();
    if let Some(branches_config) = node_def.params["branches"].as_array() {
        for item in branches_config {
            let Some(branch) = item.as_array() else {
                continue;
            };
            let Some(name) = branch.first().and_then(serde_json::Value::as_str) else {
                continue;
            };
            let Some(weight) = branch.get(1) else {
                continue;
            };
            branches.push((name.to_string(), branch_weight(weight)?));
        }
    }
    Ok(branches)
}

fn semantic_guard_route_key(node_def: &NodeDefinition) -> Result<Option<String>, QianjiError> {
    if let Some(key) = explicit_semantic_guard_route_key(&node_def.params)? {
        return Ok(Some(key));
    }
    if semantic_guard_route_enabled(&node_def.params)? {
        return Ok(Some(DEFAULT_SEMANTIC_GUARD_ROUTE_KEY.to_string()));
    }
    Ok(None)
}

fn explicit_semantic_guard_route_key(params: &Value) -> Result<Option<String>, QianjiError> {
    let Some(value) = params.get("semantic_guard_route_key") else {
        return Ok(None);
    };
    match value {
        Value::Null => Ok(None),
        Value::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(QianjiError::Topology(
                    "Router semantic_guard_route_key must not be empty".to_string(),
                ));
            }
            Ok(Some(trimmed.to_string()))
        }
        _ => Err(QianjiError::Topology(
            "Router semantic_guard_route_key must be a string".to_string(),
        )),
    }
}

fn semantic_guard_route_enabled(params: &Value) -> Result<bool, QianjiError> {
    let Some(value) = params.get("semantic_guard_route") else {
        return Ok(false);
    };
    match value {
        Value::Bool(enabled) => Ok(*enabled),
        Value::Null => Ok(false),
        _ => Err(QianjiError::Topology(
            "Router semantic_guard_route must be a boolean".to_string(),
        )),
    }
}

fn branch_weight(weight: &serde_json::Value) -> Result<f32, QianjiError> {
    let weight = serde_json::from_value::<f32>(weight.clone()).map_err(|_error| {
        QianjiError::Topology(
            "Router branch weight must be a finite number within f32 range".to_string(),
        )
    })?;
    if !weight.is_finite() {
        return Err(QianjiError::Topology(
            "Router branch weight must be a finite number".to_string(),
        ));
    }
    Ok(weight)
}
