#[path = "dmn_parse_xml_decode.rs"]
mod decode;

pub(crate) use self::decode::{
    append_capture_reference, append_capture_text, attribute_value, local_name, required_attribute,
};
use super::state::{
    CaptureTarget, TempDecision, TempInput, TempOutput, TempRule, TempTable, finalize_input,
    finalize_input_entry, finalize_output, finalize_output_entry, finalize_rule,
    hit_policy_from_attr,
};
use crate::dmn_model_api::DmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    decision: &mut Option<TempDecision>,
    current_table: &mut Option<TempTable>,
    current_input: &mut Option<TempInput>,
    current_output: &mut Option<TempOutput>,
    current_rule: &mut Option<TempRule>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<()> {
    let event_name = event.name();
    let tag = local_name(event_name.as_ref());
    if handle_decision_start_tag(source, reader, event, tag, decision, current_table)? {
        return Ok(());
    }
    if handle_table_start_tag(
        source,
        reader,
        event,
        tag,
        current_table,
        current_input,
        current_output,
        current_rule,
        is_empty,
    )? {
        return Ok(());
    }
    handle_capture_start_tag(
        source,
        tag,
        current_rule,
        capture_target,
        capture_buffer,
        is_empty,
    )
}

fn handle_decision_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    decision: &mut Option<TempDecision>,
    current_table: &mut Option<TempTable>,
) -> Result<bool> {
    match tag {
        "decision" => {
            if decision.is_some() {
                return Err(BpmnEngineError::UnsupportedDmnDecisionCount {
                    source_id: source.source_id.clone(),
                    count: 2,
                });
            }
            *decision = Some(TempDecision {
                decision_id: required_attribute(source, reader, event, "decision", "id")?,
                name: attribute_value(source, reader, event, "name")?,
                table: None,
            });
            Ok(true)
        }
        "decisionTable" => {
            let Some(decision) = decision.as_ref() else {
                return Ok(true);
            };
            if decision.table.is_some() || current_table.is_some() {
                return Err(BpmnEngineError::UnsupportedDmnDecisionTableCount {
                    decision_id: decision.decision_id.clone(),
                    count: 2,
                });
            }
            *current_table = Some(TempTable {
                table_id: required_attribute(source, reader, event, "decisionTable", "id")?,
                name: attribute_value(source, reader, event, "name")?,
                hit_policy: hit_policy_from_attr(
                    source,
                    decision.decision_id.as_str(),
                    attribute_value(source, reader, event, "hitPolicy")?.as_deref(),
                )?,
                inputs: Vec::new(),
                outputs: Vec::new(),
                rules: Vec::new(),
            });
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_table_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    current_table: &mut Option<TempTable>,
    current_input: &mut Option<TempInput>,
    current_output: &mut Option<TempOutput>,
    current_rule: &mut Option<TempRule>,
    is_empty: bool,
) -> Result<bool> {
    match tag {
        "input" => {
            if current_table.is_none() {
                return Ok(true);
            }
            *current_input = Some(TempInput {
                input_id: required_attribute(source, reader, event, "input", "id")?,
                label: attribute_value(source, reader, event, "label")?,
                name: attribute_value(source, reader, event, "name")?,
                expression: None,
            });
            if is_empty {
                finalize_input(current_table, current_input);
            }
            Ok(true)
        }
        "output" => {
            if current_table.is_none() {
                return Ok(true);
            }
            *current_output = Some(TempOutput {
                output_id: required_attribute(source, reader, event, "output", "id")?,
                label: attribute_value(source, reader, event, "label")?,
                name: attribute_value(source, reader, event, "name")?,
            });
            if is_empty {
                finalize_output(current_table, current_output);
            }
            Ok(true)
        }
        "rule" => {
            if current_table.is_none() {
                return Ok(true);
            }
            *current_rule = Some(TempRule {
                rule_id: required_attribute(source, reader, event, "rule", "id")?,
                description: None,
                input_entries: Vec::new(),
                output_entries: Vec::new(),
            });
            if is_empty {
                finalize_rule(source, current_table, current_rule)?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn handle_capture_start_tag(
    source: &DmnSourceFile,
    tag: &str,
    current_rule: &mut Option<TempRule>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<()> {
    match tag {
        "inputExpression" => {
            *capture_target = Some(CaptureTarget::InputExpression);
            capture_buffer.clear();
        }
        "description" if current_rule.is_some() => {
            *capture_target = Some(CaptureTarget::RuleDescription);
            capture_buffer.clear();
        }
        "inputEntry" => {
            *capture_target = Some(CaptureTarget::InputEntry);
            capture_buffer.clear();
            if is_empty {
                finalize_input_entry(source, current_rule, capture_buffer)?;
                *capture_target = None;
                capture_buffer.clear();
            }
        }
        "outputEntry" => {
            *capture_target = Some(CaptureTarget::OutputEntry);
            capture_buffer.clear();
            if is_empty {
                finalize_output_entry(source, current_rule, capture_buffer)?;
                *capture_target = None;
                capture_buffer.clear();
            }
        }
        _ => {}
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_end_tag(
    source: &DmnSourceFile,
    tag: &str,
    decision: &mut Option<TempDecision>,
    current_table: &mut Option<TempTable>,
    current_input: &mut Option<TempInput>,
    current_output: &mut Option<TempOutput>,
    current_rule: &mut Option<TempRule>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
) -> Result<()> {
    match tag {
        "inputExpression" if capture_target == &Some(CaptureTarget::InputExpression) => {
            if let Some(input) = current_input.as_mut() {
                let text = capture_buffer.trim();
                if !text.is_empty() {
                    input.expression = Some(text.to_string());
                }
            }
            *capture_target = None;
            capture_buffer.clear();
        }
        "description" if capture_target == &Some(CaptureTarget::RuleDescription) => {
            if let Some(rule) = current_rule.as_mut() {
                let text = capture_buffer.trim();
                if !text.is_empty() {
                    rule.description = Some(text.to_string());
                }
            }
            *capture_target = None;
            capture_buffer.clear();
        }
        "inputEntry" if capture_target == &Some(CaptureTarget::InputEntry) => {
            finalize_input_entry(source, current_rule, capture_buffer)?;
            *capture_target = None;
            capture_buffer.clear();
        }
        "outputEntry" if capture_target == &Some(CaptureTarget::OutputEntry) => {
            finalize_output_entry(source, current_rule, capture_buffer)?;
            *capture_target = None;
            capture_buffer.clear();
        }
        "input" => finalize_input(current_table, current_input),
        "output" => finalize_output(current_table, current_output),
        "rule" => finalize_rule(source, current_table, current_rule)?,
        "decisionTable" => {
            let Some(table) = current_table.take() else {
                return Ok(());
            };
            let Some(decision) = decision.as_mut() else {
                return Ok(());
            };
            decision.table = Some(table);
        }
        _ => {}
    }
    Ok(())
}
