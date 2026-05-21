use super::node::last_process_node_mut;
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::parser::import::attributes::attribute_value;
use crate::parser::import::{RawProcess, RawRepeatSpec};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

pub(in crate::parser::import) fn apply_multi_instance_loop_cardinality(
    process: &mut RawProcess,
    loop_cardinality: &str,
) -> Result<()> {
    let process_id = process.process_id.clone();
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_multi_instance_loop_cardinality_missing_node",
        })?;
    let node_id = node.bpmn_id.clone();
    let trimmed = loop_cardinality.trim();
    let parsed_loop_cardinality =
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.parse::<u32>().map_err(|_| {
                BpmnEngineError::UnsupportedLoopConfiguration {
                    process_id: process_id.into(),
                    node_id: node_id.into(),
                    detail: "invalid_loop_cardinality",
                }
            })?)
        };
    match node.repeat.as_mut() {
        Some(RawRepeatSpec::SequentialMultiInstance(loop_spec)) => {
            loop_spec.loop_cardinality = parsed_loop_cardinality;
        }
        Some(RawRepeatSpec::ParallelMultiInstance(loop_spec)) => {
            loop_spec.loop_cardinality = parsed_loop_cardinality;
        }
        _ => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "apply_multi_instance_loop_cardinality_missing_repeat_spec",
            });
        }
    }
    Ok(())
}

pub(in crate::parser::import) fn apply_multi_instance_loop_data_input_ref(
    process: &mut RawProcess,
    loop_data_input_ref: &str,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_multi_instance_loop_data_input_ref_missing_node",
        })?;
    let loop_data_input_ref =
        (!loop_data_input_ref.trim().is_empty()).then(|| loop_data_input_ref.trim().to_string());
    match node.repeat.as_mut() {
        Some(RawRepeatSpec::SequentialMultiInstance(loop_spec)) => {
            loop_spec.loop_data_input_ref = loop_data_input_ref;
        }
        Some(RawRepeatSpec::ParallelMultiInstance(loop_spec)) => {
            loop_spec.loop_data_input_ref = loop_data_input_ref;
        }
        _ => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "apply_multi_instance_loop_data_input_ref_missing_repeat_spec",
            });
        }
    }
    Ok(())
}

pub(in crate::parser::import) fn apply_multi_instance_loop_data_output_ref(
    process: &mut RawProcess,
    loop_data_output_ref: &str,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_multi_instance_loop_data_output_ref_missing_node",
        })?;
    let loop_data_output_ref =
        (!loop_data_output_ref.trim().is_empty()).then(|| loop_data_output_ref.trim().to_string());
    match node.repeat.as_mut() {
        Some(RawRepeatSpec::SequentialMultiInstance(loop_spec)) => {
            loop_spec.loop_data_output_ref = loop_data_output_ref;
        }
        Some(RawRepeatSpec::ParallelMultiInstance(loop_spec)) => {
            loop_spec.loop_data_output_ref = loop_data_output_ref;
        }
        _ => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "apply_multi_instance_loop_data_output_ref_missing_repeat_spec",
            });
        }
    }
    Ok(())
}

pub(in crate::parser::import) fn apply_multi_instance_input_data_item(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    process: &mut RawProcess,
    tag: &str,
) -> Result<()> {
    let process_id = process.process_id.clone();
    let node = last_process_node_mut(source, process)?;
    let input_data_item = multi_instance_data_item_identifier(source, reader, event, tag)?
        .trim()
        .to_string();
    if input_data_item.is_empty() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process_id.into(),
            node_id: (node.bpmn_id.clone()).into(),
            detail: "invalid_input_data_item",
        });
    }
    match node.repeat.as_mut() {
        Some(RawRepeatSpec::SequentialMultiInstance(loop_spec)) => {
            loop_spec.input_data_item = Some(input_data_item);
        }
        Some(RawRepeatSpec::ParallelMultiInstance(loop_spec)) => {
            loop_spec.input_data_item = Some(input_data_item);
        }
        _ => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "apply_multi_instance_input_data_item_missing_repeat_spec",
            });
        }
    }
    Ok(())
}

pub(in crate::parser::import) fn apply_multi_instance_output_data_item(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    process: &mut RawProcess,
    tag: &str,
) -> Result<()> {
    let process_id = process.process_id.clone();
    let node = last_process_node_mut(source, process)?;
    let output_data_item = multi_instance_data_item_identifier(source, reader, event, tag)?
        .trim()
        .to_string();
    if output_data_item.is_empty() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process_id.into(),
            node_id: (node.bpmn_id.clone()).into(),
            detail: "invalid_output_data_item",
        });
    }
    match node.repeat.as_mut() {
        Some(RawRepeatSpec::SequentialMultiInstance(loop_spec)) => {
            loop_spec.output_data_item = Some(output_data_item);
        }
        Some(RawRepeatSpec::ParallelMultiInstance(loop_spec)) => {
            loop_spec.output_data_item = Some(output_data_item);
        }
        _ => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "apply_multi_instance_output_data_item_missing_repeat_spec",
            });
        }
    }
    Ok(())
}

fn multi_instance_data_item_identifier(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
) -> Result<String> {
    attribute_value(reader, event, "id")?
        .or_else(|| attribute_value(reader, event, "name").ok().flatten())
        .ok_or_else(|| BpmnEngineError::MissingAttribute {
            source_id: (source.source_id.clone()).into(),
            element: tag.to_string(),
            attribute: "id".to_string(),
        })
}

pub(in crate::parser::import) fn apply_multi_instance_completion_condition(
    process: &mut RawProcess,
    completion_condition: &str,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_multi_instance_completion_condition_missing_node",
        })?;
    let completion_condition =
        (!completion_condition.trim().is_empty()).then(|| completion_condition.to_string());
    match node.repeat.as_mut() {
        Some(RawRepeatSpec::SequentialMultiInstance(loop_spec)) => {
            loop_spec.completion_condition = completion_condition;
        }
        Some(RawRepeatSpec::ParallelMultiInstance(loop_spec)) => {
            loop_spec.completion_condition = completion_condition;
        }
        _ => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "apply_multi_instance_completion_condition_missing_repeat_spec",
            });
        }
    }
    Ok(())
}
