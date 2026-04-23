use std::collections::BTreeMap;

use crate::contracts::FlowhubGraphTopology;

use super::scenario_ir_annotation_model::AnnotationValue;
use super::scenario_ir_annotations::FlowhubGraphAnnotationError;

pub(super) fn collect_annotation_values(
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

pub(super) fn expect_scalar(
    key: &str,
    value: AnnotationValue,
) -> Result<String, FlowhubGraphAnnotationError> {
    match value {
        AnnotationValue::Scalar(value) => Ok(value),
        AnnotationValue::List(_) => Err(FlowhubGraphAnnotationError::new(format!(
            "Flowhub Mermaid annotation `{key}` must be a scalar value"
        ))),
    }
}

pub(super) fn expect_list(
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

pub(super) fn parse_topology(
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
