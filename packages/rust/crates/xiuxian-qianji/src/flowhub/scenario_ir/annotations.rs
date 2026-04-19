use std::collections::BTreeMap;
use std::fmt;

use crate::contracts::FlowhubGraphTopology;

/// Parsed `%% qianji.*` annotations from one Mermaid scenario-case source.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FlowhubGraphAnnotations {
    /// Scenario-level metadata.
    pub(crate) scenario: FlowhubGraphScenarioAnnotations,
    /// Node-level metadata keyed by the annotation node reference.
    pub(crate) nodes: BTreeMap<String, FlowhubGraphNodeAnnotations>,
    /// Canonical completion requirements for the done gate.
    pub(crate) done_gate_require: Vec<String>,
}

/// Scenario-level metadata owned by one graph.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FlowhubGraphScenarioAnnotations {
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) topology: Option<FlowhubGraphTopology>,
    pub(crate) workdir_root: Option<String>,
    pub(crate) requires: Vec<String>,
    pub(crate) target_root: Option<String>,
    pub(crate) target_paths: Vec<String>,
}

/// Node-level metadata owned by one graph.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FlowhubGraphNodeAnnotations {
    pub(crate) kind: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) agent_action: Option<String>,
    pub(crate) checkpoint: Option<String>,
    pub(crate) writes: Vec<String>,
    pub(crate) merge_target: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowhubGraphAnnotationError {
    message: String,
}

impl FlowhubGraphAnnotationError {
    fn new(message: impl Into<String>) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum AnnotationValue {
    Scalar(String),
    List(Vec<String>),
}

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

fn collect_annotation_values(
    source: &str,
) -> Result<BTreeMap<String, AnnotationValue>, FlowhubGraphAnnotationError> {
    let mut values = BTreeMap::<String, AnnotationValue>::new();
    let mut active_list_key: Option<String> = None;

    for (line_index, line) in source.lines().enumerate() {
        let Some(payload) = extract_annotation_payload(line) else {
            active_list_key = None;
            continue;
        };

        if payload.is_empty() {
            continue;
        }

        if payload.starts_with("qianji.") {
            let Some((key, raw_value)) = payload.split_once(':') else {
                return Err(FlowhubGraphAnnotationError::new(format!(
                    "line {}: Flowhub Mermaid annotation `{payload}` must use `key: value` syntax",
                    line_index + 1
                )));
            };
            let key = key.trim().to_string();
            let value = raw_value.trim().to_string();
            if value.is_empty() {
                insert_annotation_value(
                    &mut values,
                    key.clone(),
                    AnnotationValue::List(Vec::new()),
                    line_index + 1,
                )?;
                active_list_key = Some(key);
            } else {
                insert_annotation_value(
                    &mut values,
                    key,
                    AnnotationValue::Scalar(value),
                    line_index + 1,
                )?;
                active_list_key = None;
            }
            continue;
        }

        if let Some(list_key) = active_list_key.as_ref()
            && let Some(item) = payload.strip_prefix("- ")
        {
            let item = item.trim();
            if item.is_empty() {
                return Err(FlowhubGraphAnnotationError::new(format!(
                    "line {}: Flowhub Mermaid list annotation `{list_key}` cannot contain an empty item",
                    line_index + 1
                )));
            }
            push_list_item(&mut values, list_key, item.to_string(), line_index + 1)?;
            continue;
        }

        active_list_key = None;
    }

    Ok(values)
}

fn extract_annotation_payload(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let payload = trimmed.strip_prefix("%%")?.trim_start();
    Some(payload)
}

fn insert_annotation_value(
    values: &mut BTreeMap<String, AnnotationValue>,
    key: String,
    value: AnnotationValue,
    line_number: usize,
) -> Result<(), FlowhubGraphAnnotationError> {
    if values.contains_key(&key) {
        return Err(FlowhubGraphAnnotationError::new(format!(
            "line {line_number}: duplicate Flowhub Mermaid annotation `{key}`"
        )));
    }
    values.insert(key, value);
    Ok(())
}

fn push_list_item(
    values: &mut BTreeMap<String, AnnotationValue>,
    key: &str,
    item: String,
    line_number: usize,
) -> Result<(), FlowhubGraphAnnotationError> {
    match values.get_mut(key) {
        Some(AnnotationValue::List(entries)) => {
            entries.push(item);
            Ok(())
        }
        Some(AnnotationValue::Scalar(_)) => Err(FlowhubGraphAnnotationError::new(format!(
            "line {line_number}: Flowhub Mermaid annotation `{key}` is not a list"
        ))),
        None => Err(FlowhubGraphAnnotationError::new(format!(
            "line {line_number}: Flowhub Mermaid list item has no owning annotation key"
        ))),
    }
}

fn expect_scalar(key: &str, value: AnnotationValue) -> Result<String, FlowhubGraphAnnotationError> {
    match value {
        AnnotationValue::Scalar(value) => Ok(value),
        AnnotationValue::List(_) => Err(FlowhubGraphAnnotationError::new(format!(
            "Flowhub Mermaid annotation `{key}` must be a scalar value"
        ))),
    }
}

fn expect_list(
    key: &str,
    value: AnnotationValue,
) -> Result<Vec<String>, FlowhubGraphAnnotationError> {
    match value {
        AnnotationValue::List(entries) => Ok(entries),
        AnnotationValue::Scalar(_) => Err(FlowhubGraphAnnotationError::new(format!(
            "Flowhub Mermaid annotation `{key}` must be a list"
        ))),
    }
}

fn parse_topology(
    key: &str,
    value: &str,
) -> Result<FlowhubGraphTopology, FlowhubGraphAnnotationError> {
    match value {
        "dag" => Ok(FlowhubGraphTopology::Dag),
        "bounded_loop" => Ok(FlowhubGraphTopology::BoundedLoop),
        "open_loop" => Ok(FlowhubGraphTopology::OpenLoop),
        _ => Err(FlowhubGraphAnnotationError::new(format!(
            "Flowhub Mermaid annotation `{key}` must be one of `dag`, `bounded_loop`, or `open_loop`"
        ))),
    }
}

fn apply_node_annotation(
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

#[cfg(test)]
#[path = "../../../tests/unit/flowhub/scenario_ir/annotations.rs"]
mod tests;
