use std::fmt;

use super::scenario_ir_annotation_node::apply_node_annotation;
use super::scenario_ir_annotation_support::{
    collect_annotation_values, expect_list, expect_scalar, parse_topology,
};

pub(crate) use super::scenario_ir_annotation_model::FlowhubGraphAnnotations;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowhubGraphAnnotationError {
    message: String,
}

impl FlowhubGraphAnnotationError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for FlowhubGraphAnnotationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for FlowhubGraphAnnotationError {}

/// Parse `%% qianji.*` annotations from one Mermaid source.
pub(crate) fn parse_flowhub_graph_annotations(
    source: &str,
) -> Result<Option<FlowhubGraphAnnotations>, FlowhubGraphAnnotationError> {
    let values = collect_annotation_values(source)?;
    if values.is_empty() {
        return Ok(None);
    }

    let mut annotations = FlowhubGraphAnnotations::default();
    for (key, value) in values {
        match key.as_str() {
            "qianji.scenario.id" => annotations.scenario.id = Some(expect_scalar(&key, value)?),
            "qianji.scenario.name" => {
                annotations.scenario.name = Some(expect_scalar(&key, value)?);
            }
            "qianji.scenario.description" => {
                annotations.scenario.description = Some(expect_scalar(&key, value)?);
            }
            "qianji.scenario.note" => {
                annotations.scenario.note = Some(expect_scalar(&key, value)?);
            }
            "qianji.scenario.topology" => {
                let raw = expect_scalar(&key, value)?;
                annotations.scenario.topology = Some(parse_topology(&key, raw.as_str())?);
            }
            "qianji.scenario.workdir_root" => {
                annotations.scenario.workdir_root = Some(expect_scalar(&key, value)?);
            }
            "qianji.scenario.requires" => {
                annotations.scenario.requires = expect_list(&key, value)?;
            }
            "qianji.scenario.target_root" => {
                annotations.scenario.target_root = Some(expect_scalar(&key, value)?);
            }
            "qianji.scenario.target_paths" => {
                annotations.scenario.target_paths = expect_list(&key, value)?;
            }
            "qianji.done_gate.require" => {
                annotations.done_gate_require = expect_list(&key, value)?;
            }
            _ if key.starts_with("qianji.node.") => {
                apply_node_annotation(&mut annotations, &key, value)?;
            }
            _ => {
                return Err(FlowhubGraphAnnotationError::new(format!(
                    "unsupported Flowhub Mermaid annotation key `{key}`"
                )));
            }
        }
    }

    Ok(Some(annotations))
}

#[cfg(test)]
#[path = "../../../tests/unit/flowhub/scenario_ir/annotations.rs"]
mod tests;
