use crate::contracts::NodeDefinition;
use crate::error::QianjiError;

pub(super) fn retry_targets(node_def: &NodeDefinition) -> Vec<String> {
    node_def
        .params
        .get("retry_targets")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn uses_llm_controller(node_def: &NodeDefinition) -> bool {
    node_def.qianhuan.is_some() && node_def.llm.is_some()
}

pub(super) fn ensure_native_retry_budget_not_configured(
    node_def: &NodeDefinition,
) -> Result<(), QianjiError> {
    if node_def.params.get("max_retries").is_some() {
        return Err(QianjiError::Topology(
            "formal_audit.max_retries requires `[nodes.qianhuan] + [nodes.llm]`; native formal_audit only supports retry_targets.".to_string(),
        ));
    }
    Ok(())
}
