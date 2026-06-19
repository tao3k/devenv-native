use crate::contracts::{NodeAnnotationExecutionMode, NodeDefinition};
use crate::engine::NodeExecutionAffinity;
use std::collections::HashSet;

pub(super) struct AnnotationMechanismConfig {
    pub(super) persona_id: String,
    pub(super) template_target: Option<String>,
    pub(super) execution_mode: NodeAnnotationExecutionMode,
    pub(super) input_keys: Vec<String>,
    pub(super) history_key: String,
    pub(super) output_key: String,
}

pub(super) fn mechanism_config(node_def: &NodeDefinition) -> AnnotationMechanismConfig {
    AnnotationMechanismConfig {
        persona_id: persona_id(node_def),
        template_target: template_target(node_def),
        execution_mode: execution_mode(node_def),
        input_keys: input_keys(node_def),
        history_key: history_key(node_def),
        output_key: output_key(node_def),
    }
}

pub(super) fn node_execution_affinity(node_def: &NodeDefinition) -> NodeExecutionAffinity {
    let agent_id = node_param_string(node_def, "agent_id")
        .or_else(|| node_param_string(node_def, "executor_agent_id"));
    let role_class = node_param_string(node_def, "role_class")
        .or_else(|| node_param_string(node_def, "agent_role"))
        .or_else(|| derive_role_class_from_persona(node_def));

    NodeExecutionAffinity {
        agent_id,
        role_class,
    }
}

fn persona_id(node_def: &NodeDefinition) -> String {
    non_empty(
        node_def
            .annotation
            .as_ref()
            .and_then(|binding| binding.persona_id.as_deref()),
    )
    .map_or_else(
        || "artisan-engineer".to_string(),
        |value| resolve_semantic_placeholder(&value),
    )
}

fn template_target(node_def: &NodeDefinition) -> Option<String> {
    non_empty(
        node_def
            .annotation
            .as_ref()
            .and_then(|binding| binding.template_target.as_deref()),
    )
    .map(|value| resolve_semantic_placeholder(&value))
}

fn execution_mode(node_def: &NodeDefinition) -> NodeAnnotationExecutionMode {
    node_def
        .annotation
        .as_ref()
        .map_or(NodeAnnotationExecutionMode::Isolated, |binding| {
            binding.execution_mode.clone()
        })
}

fn history_key(node_def: &NodeDefinition) -> String {
    non_empty(
        node_def
            .annotation
            .as_ref()
            .and_then(|binding| binding.history_key.as_deref()),
    )
    .unwrap_or_else(|| "annotation_history".to_string())
}

fn output_key(node_def: &NodeDefinition) -> String {
    non_empty(
        node_def
            .annotation
            .as_ref()
            .and_then(|binding| binding.output_key.as_deref()),
    )
    .unwrap_or_else(|| "annotated_prompt".to_string())
}

fn input_keys(node_def: &NodeDefinition) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut keys = node_def
        .annotation
        .as_ref()
        .map(|binding| {
            binding
                .input_keys
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .filter(|value| seen.insert((*value).to_string()))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if keys.is_empty() {
        keys.push("raw_facts".to_string());
    }
    keys
}

fn node_param_string(node_def: &NodeDefinition, key: &str) -> Option<String> {
    non_empty(node_def.params.get(key).and_then(serde_json::Value::as_str))
        .map(|value| resolve_semantic_placeholder(&value))
}

fn derive_role_class_from_persona(node_def: &NodeDefinition) -> Option<String> {
    let persona_id = non_empty(
        node_def
            .annotation
            .as_ref()
            .and_then(|binding| binding.persona_id.as_deref()),
    )?;

    let resolved = resolve_semantic_placeholder(&persona_id);
    let stripped = resolved
        .trim_start_matches('$')
        .trim_end_matches('/')
        .trim();
    if stripped.is_empty() {
        return None;
    }

    let file_name = stripped.rsplit('/').next().unwrap_or(stripped);
    let role_name = file_name.strip_suffix(".md").unwrap_or(file_name).trim();
    if role_name.is_empty() {
        None
    } else {
        Some(role_name.to_ascii_lowercase())
    }
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn resolve_semantic_placeholder(raw: &str) -> String {
    raw.trim().to_string()
}
