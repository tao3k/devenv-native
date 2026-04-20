//! Bounded DMN XML parsing for one decision and one decision table.

use super::{
    DmnComparisonOperator, DmnDateComparison, DmnDateRange, DmnDateRangeBound,
    DmnDecisionDefinition, DmnDecisionRef, DmnHitPolicy, DmnInputClause, DmnInputEntry,
    DmnNumericComparison, DmnNumericRange, DmnNumericRangeBound, DmnOutputClause, DmnOutputEntry,
    DmnRule, DmnSourceFile,
};
use crate::error::{BpmnEngineError, Result};
use chrono::NaiveDate;
use quick_xml::Reader;
use quick_xml::escape::{resolve_predefined_entity, unescape};
use quick_xml::events::{BytesRef, BytesStart, Event};
use std::borrow::Cow;

struct TempDecision {
    decision_id: String,
    name: Option<String>,
    table: Option<TempTable>,
}

struct TempTable {
    table_id: String,
    name: Option<String>,
    hit_policy: DmnHitPolicy,
    inputs: Vec<DmnInputClause>,
    outputs: Vec<DmnOutputClause>,
    rules: Vec<DmnRule>,
}

struct TempInput {
    input_id: String,
    label: Option<String>,
    name: Option<String>,
    expression: Option<String>,
}

struct TempOutput {
    output_id: String,
    label: Option<String>,
    name: Option<String>,
}

struct TempRule {
    rule_id: String,
    description: Option<String>,
    input_entries: Vec<DmnInputEntry>,
    output_entries: Vec<DmnOutputEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureTarget {
    InputExpression,
    RuleDescription,
    InputEntry,
    OutputEntry,
}

/// Parses one bounded DMN source into a single-decision definition.
///
/// # Errors
///
/// Returns typed DMN parse errors when the XML payload is malformed or when
/// the document exceeds the bounded single-decision and single-table slice.
pub fn parse_dmn_decision(source: &DmnSourceFile) -> Result<DmnDecisionDefinition> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);

    let mut saw_root = false;
    let mut decision: Option<TempDecision> = None;
    let mut current_table: Option<TempTable> = None;
    let mut current_input: Option<TempInput> = None;
    let mut current_output: Option<TempOutput> = None;
    let mut current_rule: Option<TempRule> = None;
    let mut capture_target: Option<CaptureTarget> = None;
    let mut capture_buffer = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                saw_root = true;
                handle_start_tag(
                    source,
                    &reader,
                    &event,
                    &mut decision,
                    &mut current_table,
                    &mut current_input,
                    &mut current_output,
                    &mut current_rule,
                    &mut capture_target,
                    &mut capture_buffer,
                    false,
                )?;
            }
            Ok(Event::Empty(event)) => {
                saw_root = true;
                handle_start_tag(
                    source,
                    &reader,
                    &event,
                    &mut decision,
                    &mut current_table,
                    &mut current_input,
                    &mut current_output,
                    &mut current_rule,
                    &mut capture_target,
                    &mut capture_buffer,
                    true,
                )?;
            }
            Ok(Event::Text(event)) => append_capture_text(
                source,
                capture_target.as_ref(),
                &mut capture_buffer,
                event.decode(),
            )?,
            Ok(Event::CData(event)) => append_capture_text(
                source,
                capture_target.as_ref(),
                &mut capture_buffer,
                event.decode(),
            )?,
            Ok(Event::GeneralRef(event)) => append_capture_reference(
                source,
                capture_target.as_ref(),
                &mut capture_buffer,
                &event,
            )?,
            Ok(Event::End(event)) => {
                handle_end_tag(
                    source,
                    local_name(event.name().as_ref()),
                    &mut decision,
                    &mut current_table,
                    &mut current_input,
                    &mut current_output,
                    &mut current_rule,
                    &mut capture_target,
                    &mut capture_buffer,
                )?;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                return Err(BpmnEngineError::InvalidDmnXml {
                    source_id: source.source_id.clone(),
                    message: error.to_string(),
                });
            }
        }
    }

    if !saw_root {
        return Err(BpmnEngineError::MissingDmnRootElement {
            source_id: source.source_id.clone(),
        });
    }

    finalize_decision_definition(source, decision)
}

#[allow(clippy::too_many_arguments)]
fn handle_start_tag(
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

fn append_capture_text(
    source: &DmnSourceFile,
    capture_target: Option<&CaptureTarget>,
    capture_buffer: &mut String,
    decoded: std::result::Result<Cow<'_, str>, quick_xml::encoding::EncodingError>,
) -> Result<()> {
    if capture_target.is_none() {
        return Ok(());
    }
    let text = decoded.map_err(|error| BpmnEngineError::InvalidDmnXml {
        source_id: source.source_id.clone(),
        message: error.to_string(),
    })?;
    let text = unescape(text.as_ref()).map_err(|error| BpmnEngineError::InvalidDmnXml {
        source_id: source.source_id.clone(),
        message: error.to_string(),
    })?;
    capture_buffer.push_str(text.as_ref());
    Ok(())
}

fn append_capture_reference(
    source: &DmnSourceFile,
    capture_target: Option<&CaptureTarget>,
    capture_buffer: &mut String,
    reference: &BytesRef<'_>,
) -> Result<()> {
    if capture_target.is_none() {
        return Ok(());
    }
    if let Some(ch) =
        reference
            .resolve_char_ref()
            .map_err(|error| BpmnEngineError::InvalidDmnXml {
                source_id: source.source_id.clone(),
                message: error.to_string(),
            })?
    {
        capture_buffer.push(ch);
        return Ok(());
    }

    let reference = reference
        .decode()
        .map_err(|error| BpmnEngineError::InvalidDmnXml {
            source_id: source.source_id.clone(),
            message: error.to_string(),
        })?;
    let entity = resolve_predefined_entity(reference.as_ref()).ok_or_else(|| {
        BpmnEngineError::InvalidDmnXml {
            source_id: source.source_id.clone(),
            message: format!("unrecognized XML entity reference '&{reference};'"),
        }
    })?;
    capture_buffer.push_str(entity);
    Ok(())
}

fn finalize_decision_definition(
    source: &DmnSourceFile,
    decision: Option<TempDecision>,
) -> Result<DmnDecisionDefinition> {
    let decision = decision.ok_or_else(|| BpmnEngineError::MissingDmnDecision {
        source_id: source.source_id.clone(),
    })?;
    let table = decision
        .table
        .ok_or_else(|| BpmnEngineError::MissingDmnDecisionTable {
            decision_id: decision.decision_id.clone(),
        })?;
    Ok(DmnDecisionDefinition::new(
        &source.source_id,
        DmnDecisionRef::new(&decision.decision_id).with_source_id(&source.source_id),
        decision.name,
        table.into_definition(),
    ))
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
fn handle_end_tag(
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

fn finalize_input(current_table: &mut Option<TempTable>, current_input: &mut Option<TempInput>) {
    let Some(table) = current_table.as_mut() else {
        return;
    };
    let Some(input) = current_input.take() else {
        return;
    };
    table.inputs.push(DmnInputClause::new(
        input.input_id,
        input.label,
        input.name,
        input.expression,
    ));
}

fn finalize_output(current_table: &mut Option<TempTable>, current_output: &mut Option<TempOutput>) {
    let Some(table) = current_table.as_mut() else {
        return;
    };
    let Some(output) = current_output.take() else {
        return;
    };
    table.outputs.push(DmnOutputClause::new(
        output.output_id,
        output.label,
        output.name,
    ));
}

fn finalize_rule(
    source: &DmnSourceFile,
    current_table: &mut Option<TempTable>,
    current_rule: &mut Option<TempRule>,
) -> Result<()> {
    let Some(table) = current_table.as_mut() else {
        return Ok(());
    };
    let Some(rule) = current_rule.take() else {
        return Ok(());
    };
    if rule.input_entries.len() != table.inputs.len()
        || rule.output_entries.len() != table.outputs.len()
    {
        return Err(BpmnEngineError::InvalidDmnRuleArity {
            source_id: source.source_id.clone(),
            rule_id: rule.rule_id.clone(),
            expected_inputs: table.inputs.len(),
            actual_inputs: rule.input_entries.len(),
            expected_outputs: table.outputs.len(),
            actual_outputs: rule.output_entries.len(),
        });
    }
    table.rules.push(DmnRule::new(
        rule.rule_id,
        rule.description,
        rule.input_entries,
        rule.output_entries,
    ));
    Ok(())
}

fn finalize_input_entry(
    source: &DmnSourceFile,
    current_rule: &mut Option<TempRule>,
    capture_buffer: &str,
) -> Result<()> {
    let Some(rule) = current_rule.as_mut() else {
        return Ok(());
    };
    rule.input_entries.push(parse_input_entry(
        source.source_id.as_str(),
        capture_buffer.trim(),
    )?);
    Ok(())
}

fn finalize_output_entry(
    source: &DmnSourceFile,
    current_rule: &mut Option<TempRule>,
    capture_buffer: &str,
) -> Result<()> {
    let Some(rule) = current_rule.as_mut() else {
        return Ok(());
    };
    rule.output_entries.push(DmnOutputEntry::new(parse_literal(
        source.source_id.as_str(),
        capture_buffer.trim(),
    )?));
    Ok(())
}

fn hit_policy_from_attr(
    source: &DmnSourceFile,
    decision_id: &str,
    raw: Option<&str>,
) -> Result<DmnHitPolicy> {
    match raw.unwrap_or("UNIQUE").trim().to_ascii_uppercase().as_str() {
        "UNIQUE" => Ok(DmnHitPolicy::Unique),
        "COLLECT" => Ok(DmnHitPolicy::Collect),
        policy => Err(BpmnEngineError::UnsupportedDmnHitPolicy {
            source_id: source.source_id.clone(),
            decision_id: decision_id.to_string(),
            hit_policy: policy.to_string(),
        }),
    }
}

fn parse_input_entry(source_id: &str, raw: &str) -> Result<DmnInputEntry> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return Ok(DmnInputEntry::Any);
    }
    if let Ok(literal) = parse_literal(source_id, trimmed) {
        return Ok(DmnInputEntry::Equals(literal));
    }
    if let Some(parsed) = parse_date_comparison(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_date_question_mark_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_date_interval_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_numeric_comparison(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_numeric_question_mark_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    if let Some(parsed) = parse_numeric_interval_range(source_id, trimmed)? {
        return Ok(parsed);
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: trimmed.to_string(),
    })
}

fn parse_literal(source_id: &str, raw: &str) -> Result<serde_json::Value> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    if let Some(date) = parse_date_literal(source_id, trimmed)? {
        return Ok(serde_json::Value::String(date));
    }
    if let Some(value) = parse_quoted_string(trimmed) {
        return Ok(serde_json::Value::String(value));
    }
    match trimmed {
        "true" => return Ok(serde_json::Value::Bool(true)),
        "false" => return Ok(serde_json::Value::Bool(false)),
        "null" => return Ok(serde_json::Value::Null),
        _ => {}
    }
    if let Ok(number) = serde_json::from_str::<serde_json::Number>(trimmed) {
        return Ok(serde_json::Value::Number(number));
    }
    if is_bare_string_token(trimmed) {
        return Ok(serde_json::Value::String(trimmed.to_string()));
    }
    Err(BpmnEngineError::UnsupportedDmnLiteral {
        source_id: source_id.to_string(),
        literal: trimmed.to_string(),
    })
}

fn parse_date_literal(source_id: &str, raw: &str) -> Result<Option<String>> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("date(") {
        return Ok(None);
    }
    let Some(inner) = trimmed
        .strip_prefix("date(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        });
    };
    let Some(value) = parse_quoted_string(inner.trim()) else {
        return Err(BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        });
    };
    let date = NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|_| {
        BpmnEngineError::UnsupportedDmnLiteral {
            source_id: source_id.to_string(),
            literal: trimmed.to_string(),
        }
    })?;
    Ok(Some(date.to_string()))
}

fn parse_date_comparison(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    let Some((operator, value_raw)) = parse_comparison_prefix(raw) else {
        return Ok(None);
    };
    if !value_raw.trim().starts_with("date(") {
        return Ok(None);
    }
    let value = parse_date_unary_value(source_id, value_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::DateComparison(DmnDateComparison::new(
        operator, value,
    ))))
}

fn parse_numeric_comparison(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    let Some((operator, value_raw)) = parse_comparison_prefix(raw) else {
        return Ok(None);
    };
    let value = parse_numeric_value(source_id, value_raw, raw)?;
    Ok(Some(DmnInputEntry::NumericComparison(
        DmnNumericComparison::new(operator, value),
    )))
}

fn parse_date_question_mark_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    if !raw.contains('?') {
        return Ok(None);
    }
    let Some((left_raw, right_raw)) = raw.split_once('?') else {
        return Ok(None);
    };
    if right_raw.contains('?') {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
            expression: raw.to_string(),
        });
    }
    if !(left_raw.contains("date(") || right_raw.contains("date(")) {
        return Ok(None);
    }

    let lower = parse_left_question_mark_date_bound(source_id, left_raw.trim(), raw)?;
    let upper = parse_right_question_mark_date_bound(source_id, right_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::DateRange(DmnDateRange::new(
        Some(lower),
        Some(upper),
    ))))
}

fn parse_numeric_question_mark_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    if !raw.contains('?') {
        return Ok(None);
    }
    let Some((left_raw, right_raw)) = raw.split_once('?') else {
        return Ok(None);
    };
    if right_raw.contains('?') {
        return Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
            expression: raw.to_string(),
        });
    }

    let lower = parse_left_question_mark_bound(source_id, left_raw.trim(), raw)?;
    let upper = parse_right_question_mark_bound(source_id, right_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::NumericRange(DmnNumericRange::new(
        Some(lower),
        Some(upper),
    ))))
}

fn parse_date_interval_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    let Some(first_char) = raw.chars().next() else {
        return Ok(None);
    };
    let Some(last_char) = raw.chars().last() else {
        return Ok(None);
    };
    if !matches!(first_char, '[' | '(') || !matches!(last_char, ']' | ')') {
        return Ok(None);
    }
    let Some(inner) = raw
        .strip_prefix(first_char)
        .and_then(|value| value.strip_suffix(last_char))
    else {
        return Ok(None);
    };
    let Some((lower_raw, upper_raw)) = inner.split_once("..") else {
        return Ok(None);
    };
    if !(lower_raw.contains("date(") || upper_raw.contains("date(")) {
        return Ok(None);
    }

    let lower = parse_date_unary_value(source_id, lower_raw.trim(), raw)?;
    let upper = parse_date_unary_value(source_id, upper_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::DateRange(DmnDateRange::new(
        Some(DmnDateRangeBound::new(lower, first_char == '[')),
        Some(DmnDateRangeBound::new(upper, last_char == ']')),
    ))))
}

fn parse_numeric_interval_range(source_id: &str, raw: &str) -> Result<Option<DmnInputEntry>> {
    let Some(first_char) = raw.chars().next() else {
        return Ok(None);
    };
    let Some(last_char) = raw.chars().last() else {
        return Ok(None);
    };
    if !matches!(first_char, '[' | '(') || !matches!(last_char, ']' | ')') {
        return Ok(None);
    }
    let Some(inner) = raw
        .strip_prefix(first_char)
        .and_then(|value| value.strip_suffix(last_char))
    else {
        return Ok(None);
    };
    let Some((lower_raw, upper_raw)) = inner.split_once("..") else {
        return Ok(None);
    };

    let lower = parse_numeric_value(source_id, lower_raw.trim(), raw)?;
    let upper = parse_numeric_value(source_id, upper_raw.trim(), raw)?;
    Ok(Some(DmnInputEntry::NumericRange(DmnNumericRange::new(
        Some(DmnNumericRangeBound::new(lower, first_char == '[')),
        Some(DmnNumericRangeBound::new(upper, last_char == ']')),
    ))))
}

fn parse_comparison_prefix(raw: &str) -> Option<(DmnComparisonOperator, &str)> {
    if let Some(value) = raw.strip_prefix("<=") {
        return Some((DmnComparisonOperator::LessThanOrEqual, value.trim()));
    }
    if let Some(value) = raw.strip_prefix(">=") {
        return Some((DmnComparisonOperator::GreaterThanOrEqual, value.trim()));
    }
    if let Some(value) = raw.strip_prefix('<') {
        return Some((DmnComparisonOperator::LessThan, value.trim()));
    }
    raw.strip_prefix('>')
        .map(|value| (DmnComparisonOperator::GreaterThan, value.trim()))
}

fn parse_left_question_mark_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnNumericRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_left_question_mark_date_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDateRangeBound> {
    if let Some(value_raw) = raw.strip_suffix("<=") {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_suffix('<') {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_right_question_mark_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnNumericRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnNumericRangeBound::new(
            parse_numeric_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_right_question_mark_date_bound(
    source_id: &str,
    raw: &str,
    expression: &str,
) -> Result<DmnDateRangeBound> {
    if let Some(value_raw) = raw.strip_prefix("<=") {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            true,
        ));
    }
    if let Some(value_raw) = raw.strip_prefix('<') {
        return Ok(DmnDateRangeBound::new(
            parse_date_unary_value(source_id, value_raw.trim(), expression)?,
            false,
        ));
    }
    Err(BpmnEngineError::UnsupportedDmnUnaryTest {
        source_id: source_id.to_string(),
        expression: expression.to_string(),
    })
}

fn parse_numeric_value(source_id: &str, raw: &str, expression: &str) -> Result<f64> {
    match parse_literal(source_id, raw)? {
        serde_json::Value::Number(number) => {
            number
                .as_f64()
                .ok_or_else(|| BpmnEngineError::UnsupportedDmnUnaryTest {
                    source_id: source_id.to_string(),
                    expression: expression.to_string(),
                })
        }
        _ => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
            expression: expression.to_string(),
        }),
    }
}

fn parse_date_unary_value(source_id: &str, raw: &str, expression: &str) -> Result<String> {
    match parse_date_literal(source_id, raw)? {
        Some(date) => Ok(date),
        None => Err(BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: source_id.to_string(),
            expression: expression.to_string(),
        }),
    }
}

fn parse_quoted_string(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return Some(raw[1..raw.len() - 1].to_string());
    }
    None
}

fn is_bare_string_token(raw: &str) -> bool {
    raw.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':'))
}

fn required_attribute(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    element: &str,
    attribute: &str,
) -> Result<String> {
    attribute_value(source, reader, event, attribute)?.ok_or_else(|| {
        BpmnEngineError::MissingDmnAttribute {
            source_id: source.source_id.clone(),
            element: element.to_string(),
            attribute: attribute.to_string(),
        }
    })
}

fn attribute_value(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Result<Option<String>> {
    for attribute in event.attributes() {
        let attribute = attribute.map_err(|error| BpmnEngineError::InvalidDmnXml {
            source_id: source.source_id.clone(),
            message: error.to_string(),
        })?;
        if local_name(attribute.key.as_ref()) == attribute_name {
            let value = attribute
                .decode_and_unescape_value(reader.decoder())
                .map_err(|error| BpmnEngineError::InvalidDmnXml {
                    source_id: source.source_id.clone(),
                    message: error.to_string(),
                })?;
            return Ok(Some(match value {
                Cow::Borrowed(value) => value.to_string(),
                Cow::Owned(value) => value,
            }));
        }
    }
    Ok(None)
}

fn local_name(name: &[u8]) -> &str {
    std::str::from_utf8(name)
        .ok()
        .map_or("", |raw| raw.rsplit(':').next().unwrap_or(raw))
}

impl TempTable {
    fn into_definition(self) -> super::DmnDecisionTable {
        super::DmnDecisionTable::new(
            self.table_id,
            self.name,
            self.hit_policy,
            self.inputs,
            self.outputs,
            self.rules,
        )
    }
}
