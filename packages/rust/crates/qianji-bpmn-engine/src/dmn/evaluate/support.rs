//! Shared helpers for bounded DMN evaluation leaves.

use crate::{BpmnEngineError, DmnDecisionDefinition, DmnKnowledgeRequirementReference};
use serde_json::Value;

type Result<T> = std::result::Result<T, BpmnEngineError>;

pub(super) fn knowledge_requirement_href(
    decision: &DmnDecisionDefinition,
    requirement: &DmnKnowledgeRequirementReference,
) -> Result<String> {
    let href = requirement.href.as_deref().unwrap_or("<missing>");
    href.strip_prefix('#')
        .filter(|target| !target.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| BpmnEngineError::UnsupportedDmnKnowledgeRequirementHref {
            source_id: decision.source_id.to_string(),
            decision_id: decision.decision.decision_id.to_string(),
            href: href.to_string(),
        })
}

pub(super) fn merge_evaluation_output(variables: &mut Value, output: &Value) {
    let (Some(variables), Some(output)) = (variables.as_object_mut(), output.as_object()) else {
        return;
    };
    for (key, value) in output {
        variables.insert(key.clone(), value.clone());
    }
}

pub(super) fn is_simple_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
