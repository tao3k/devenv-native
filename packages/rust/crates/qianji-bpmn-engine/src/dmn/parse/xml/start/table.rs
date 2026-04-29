use super::{
    BpmnEngineError, BytesStart, CaptureTarget, DmnSourceFile, PeerSurfaceState, Reader, Result,
    SurfaceStartState, TempInput, TempLiteralExpression, TempOutput, TempRule, TempTable,
    attribute_value, finalize_input, finalize_input_entry, finalize_output, finalize_output_entry,
    finalize_rule, hit_policy_from_attr, required_attribute,
};

pub(super) fn start_decision_table(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    surface: SurfaceStartState<'_>,
    current_table: &mut Option<TempTable>,
    peers: PeerSurfaceState<'_>,
) -> Result<bool> {
    let Some(decision) = surface.decision else {
        return Ok(true);
    };
    if decision.table.is_some()
        || decision.literal_expression.is_some()
        || decision.list_expression.is_some()
        || decision.context_expression.is_some()
        || decision.relation_expression.is_some()
        || decision.invocation.is_some()
        || surface.literal.is_some()
        || surface.invocation.is_some()
        || surface.table.is_some()
        || current_table.is_some()
        || peers.list.is_some()
        || peers.context.is_some()
        || peers.relation.is_some()
    {
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

pub(super) fn handle_literal_expression_text_start_tag(
    tag: &str,
    current_literal: Option<&TempLiteralExpression>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
) -> bool {
    if tag != "text" || current_literal.is_none() || capture_target.is_some() {
        return false;
    }
    *capture_target = Some(CaptureTarget::LiteralExpression);
    capture_buffer.clear();
    true
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_table_start_tag(
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
                type_ref: None,
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
                type_ref: attribute_value(source, reader, event, "typeRef")?,
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

pub(super) fn handle_input_expression_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    current_input: &mut Option<TempInput>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
) -> Result<bool> {
    if tag != "inputExpression" {
        return Ok(false);
    }
    if let Some(input) = current_input.as_mut() {
        input.type_ref = attribute_value(source, reader, event, "typeRef")?;
    }
    *capture_target = Some(CaptureTarget::InputExpression);
    capture_buffer.clear();
    Ok(true)
}

pub(super) fn handle_capture_start_tag(
    source: &DmnSourceFile,
    tag: &str,
    current_rule: &mut Option<TempRule>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<()> {
    match tag {
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
