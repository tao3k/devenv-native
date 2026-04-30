use crate::error::{BpmnEngineError, Result};
use crate::parser::import::model::{RawHumanTaskChoiceSpec, RawHumanTaskFreeTextSpec};

pub(super) fn parse_choice_literal(value: &str) -> Result<Vec<RawHumanTaskChoiceSpec>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let parsed: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|_| BpmnEngineError::UnsupportedOperation {
            operation: "invalid_native_human_task_choices_json",
        })?;
    let Some(items) = parsed.as_array() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "invalid_native_human_task_choices_json",
        });
    };
    items
        .iter()
        .map(|item| {
            if let Some(value) = item.as_str() {
                return Ok(RawHumanTaskChoiceSpec {
                    value: value.to_string(),
                    label: None,
                });
            }
            let Some(object) = item.as_object() else {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "invalid_native_human_task_choices_json",
                });
            };
            let Some(value) = object.get("value").and_then(|value| value.as_str()) else {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "invalid_native_human_task_choices_json",
                });
            };
            Ok(RawHumanTaskChoiceSpec {
                value: value.to_string(),
                label: object
                    .get("label")
                    .and_then(|label| label.as_str())
                    .map(ToString::to_string),
            })
        })
        .collect()
}

pub(super) fn parse_free_text_literal(value: &str) -> Result<Vec<RawHumanTaskFreeTextSpec>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let parsed: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|_| BpmnEngineError::UnsupportedOperation {
            operation: "invalid_native_human_task_free_text_json",
        })?;
    match parsed {
        serde_json::Value::String(name) => Ok(vec![RawHumanTaskFreeTextSpec {
            name,
            optional: false,
        }]),
        serde_json::Value::Object(object) => Ok(vec![free_text_from_object(&object)?]),
        serde_json::Value::Array(items) => items
            .iter()
            .map(|item| {
                let Some(object) = item.as_object() else {
                    return Err(BpmnEngineError::UnsupportedOperation {
                        operation: "invalid_native_human_task_free_text_json",
                    });
                };
                free_text_from_object(object)
            })
            .collect(),
        _ => Err(BpmnEngineError::UnsupportedOperation {
            operation: "invalid_native_human_task_free_text_json",
        }),
    }
}

fn free_text_from_object(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<RawHumanTaskFreeTextSpec> {
    let Some(name) = object.get("name").and_then(|value| value.as_str()) else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "invalid_native_human_task_free_text_json",
        });
    };
    Ok(RawHumanTaskFreeTextSpec {
        name: name.to_string(),
        optional: object
            .get("optional")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    })
}
