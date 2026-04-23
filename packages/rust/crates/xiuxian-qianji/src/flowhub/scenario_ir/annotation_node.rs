use super::scenario_ir_annotation_model::{AnnotationValue, FlowhubGraphAnnotations};
use super::scenario_ir_annotation_support::{expect_list, expect_scalar};
use super::scenario_ir_annotations::FlowhubGraphAnnotationError;

pub(super) fn apply_node_annotation(
    annotations: &mut FlowhubGraphAnnotations,
    key: &str,
    value: AnnotationValue,
) -> Result<(), FlowhubGraphAnnotationError> {
    let remainder = key.strip_prefix("qianji.node.").ok_or_else(|| {
        FlowhubGraphAnnotationError::new(format!(
            "unsupported Flowhub Mermaid node annotation `{key}`"
        ))
    })?;
    let Some((node_ref, field)) = remainder.rsplit_once('.') else {
        return Err(FlowhubGraphAnnotationError::new(format!(
            "Flowhub Mermaid node annotation `{key}` must use `qianji.node.<node_ref>.<field>`"
        )));
    };
    if node_ref.trim().is_empty() {
        return Err(FlowhubGraphAnnotationError::new(format!(
            "Flowhub Mermaid node annotation `{key}` has an empty `<node_ref>`"
        )));
    }

    let entry = annotations.nodes.entry(node_ref.to_string()).or_default();
    match field {
        "kind" => entry.kind = Some(expect_scalar(key, value)?),
        "role" => entry.role = Some(expect_scalar(key, value)?),
        "agent_action" => entry.agent_action = Some(expect_scalar(key, value)?),
        "checkpoint" => entry.checkpoint = Some(expect_scalar(key, value)?),
        "writes" => entry.writes = expect_list(key, value)?,
        "merge_target" => entry.merge_target = expect_list(key, value)?,
        _ => {
            return Err(FlowhubGraphAnnotationError::new(format!(
                "unsupported Flowhub Mermaid node annotation field `{field}` in `{key}`"
            )));
        }
    }

    Ok(())
}
