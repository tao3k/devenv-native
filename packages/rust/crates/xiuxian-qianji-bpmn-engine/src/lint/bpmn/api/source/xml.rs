use super::{
    ActiveGatewayFlow, GatewayFlowDetail, MissingBranchConditionContext, OutgoingFlowSummary,
};
use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use std::borrow::Cow;

pub(in crate::lint::bpmn::api) fn find_unescaped_placeholder_span(
    contents: &str,
    parser_offset: Option<u64>,
) -> Option<(std::ops::Range<usize>, String)> {
    let mut cursor = usize::try_from(parser_offset?)
        .ok()?
        .min(contents.len().saturating_sub(1));
    loop {
        let start = contents.get(..=cursor)?.rfind('<')?;
        if let Some(placeholder) = xml_placeholder_tag_at(contents, start) {
            return Some(placeholder);
        }
        if start == 0 {
            return None;
        }
        cursor = start - 1;
    }
}

fn xml_placeholder_tag_at(
    contents: &str,
    start: usize,
) -> Option<(std::ops::Range<usize>, String)> {
    let end = contents.get(start..)?.find('>')? + start + 1;
    let tag_name = contents
        .get(start + 1..end - 1)?
        .trim()
        .trim_end_matches('/');
    if tag_name.is_empty()
        || tag_name.starts_with(['/', '?', '!'])
        || tag_name.contains(char::is_whitespace)
        || tag_name.contains(':')
        || is_known_xml_element_hint(tag_name)
        || !tag_name.bytes().all(is_xml_name_hint_byte)
    {
        return None;
    }
    Some((start..end, tag_name.to_string()))
}

fn is_known_xml_element_hint(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "definitions"
            | "process"
            | "documentation"
            | "extensionElements"
            | "startEvent"
            | "endEvent"
            | "intermediateCatchEvent"
            | "intermediateThrowEvent"
            | "task"
            | "serviceTask"
            | "userTask"
            | "manualTask"
            | "businessRuleTask"
            | "scriptTask"
            | "receiveTask"
            | "sendTask"
            | "exclusiveGateway"
            | "inclusiveGateway"
            | "parallelGateway"
            | "eventBasedGateway"
            | "sequenceFlow"
            | "conditionExpression"
            | "boundaryEvent"
            | "subProcess"
            | "transaction"
            | "callActivity"
            | "errorEventDefinition"
            | "messageEventDefinition"
            | "signalEventDefinition"
            | "timerEventDefinition"
            | "cancelEventDefinition"
            | "compensateEventDefinition"
            | "script"
            | "standardLoopCharacteristics"
            | "multiInstanceLoopCharacteristics"
            | "loopCardinality"
            | "completionCondition"
            | "loopDataInputRef"
            | "loopDataOutputRef"
            | "inputDataItem"
            | "outputDataItem"
            | "association"
            | "laneSet"
            | "lane"
            | "flowNodeRef"
    )
}

fn is_xml_name_hint_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':' | b'.')
}

pub(in crate::lint::bpmn::api) fn find_xml_error_token_span(
    contents: &str,
    parser_offset: Option<u64>,
) -> Option<std::ops::Range<usize>> {
    let offset = usize::try_from(parser_offset?)
        .ok()?
        .min(contents.len().saturating_sub(1));
    let start = contents
        .get(..=offset)?
        .rfind('<')
        .filter(|start| offset.saturating_sub(*start) <= 160)
        .unwrap_or(offset);
    let end = contents
        .get(offset..)?
        .find('>')
        .map(|delta| offset + delta + 1)
        .filter(|end| end.saturating_sub(start) <= 200)
        .unwrap_or_else(|| (offset + 1).min(contents.len()));
    (start < end).then_some(start..end)
}

pub(in crate::lint::bpmn::api) fn find_unescaped_ampersand_span(
    contents: &str,
) -> Option<std::ops::Range<usize>> {
    let mut cursor = 0usize;
    while cursor < contents.len() {
        if contents.get(cursor..)?.starts_with("<!--") {
            cursor = contents
                .get(cursor + 4..)?
                .find("-->")
                .map_or(contents.len(), |offset| cursor + 4 + offset + 3);
            continue;
        }
        if contents.get(cursor..)?.starts_with("<![CDATA[") {
            cursor = contents
                .get(cursor + 9..)?
                .find("]]>")
                .map_or(contents.len(), |offset| cursor + 9 + offset + 3);
            continue;
        }
        if contents.as_bytes().get(cursor) == Some(&b'&')
            && !is_valid_xml_entity_at(contents, cursor)
        {
            return Some(cursor..cursor + 1);
        }
        cursor += contents
            .get(cursor..)?
            .chars()
            .next()
            .map_or(1, char::len_utf8);
    }
    None
}

pub(in crate::lint::bpmn::api) fn escaped_line_fix_for_ampersand(
    contents: &str,
    offset: usize,
) -> Option<String> {
    let (line_start, line_end) = line_bounds_for_offset(contents, offset)?;
    let line = contents.get(line_start..line_end)?;
    Some(escape_unescaped_ampersands(line.trim_start()))
}

pub(in crate::lint::bpmn::api) fn malformed_closing_tag_line_fix(
    contents: &str,
    token_offset: usize,
) -> Option<String> {
    let (line_start, line_end) = line_bounds_for_offset(contents, token_offset)?;
    let line = contents.get(line_start..line_end)?;
    let relative_offset = token_offset.checked_sub(line_start)?;
    let token_start = line.get(..=relative_offset)?.rfind('<')?;
    let token_end = line.get(token_start..)?.find('>')? + token_start + 1;
    let closing_tag = line.get(token_start..token_end)?;
    let closing_name = closing_tag_name(closing_tag)?;
    let closing_local_name = xml_local_name(closing_name);
    let opening_name = find_opening_name_for_local(line, token_start, closing_local_name)?;
    if opening_name == closing_name {
        return None;
    }

    let mut repaired = String::with_capacity(line.len() + opening_name.len());
    repaired.push_str(line.get(..token_start)?);
    repaired.push_str("</");
    repaired.push_str(&opening_name);
    repaired.push('>');
    repaired.push_str(line.get(token_end..)?);
    Some(repaired.trim_start().to_string())
}

fn closing_tag_name(tag: &str) -> Option<&str> {
    let name = tag.strip_prefix("</")?.strip_suffix('>')?.trim();
    if name.is_empty() || name.contains(char::is_whitespace) {
        return None;
    }
    Some(name)
}

fn find_opening_name_for_local(
    line: &str,
    before_offset: usize,
    local_name: &str,
) -> Option<String> {
    let mut cursor = 0usize;
    let mut matched = None;
    while cursor < before_offset {
        let Some(relative_start) = line.get(cursor..before_offset)?.find('<') else {
            break;
        };
        let start = relative_start + cursor;
        if line.get(start..).is_some_and(|text| {
            text.starts_with("</") || text.starts_with("<!") || text.starts_with("<?")
        }) {
            cursor = start + 1;
            continue;
        }
        let Some(end) = line
            .get(start..before_offset)?
            .find('>')
            .map(|offset| start + offset + 1)
        else {
            break;
        };
        if let Some(name) = opening_tag_name(line.get(start..end)?)
            && xml_local_name(name) == local_name
        {
            matched = Some(name.to_string());
        }
        cursor = end;
    }
    matched
}

fn opening_tag_name(tag: &str) -> Option<&str> {
    let body = tag.strip_prefix('<')?.trim_start();
    if body.starts_with(['/', '!', '?']) {
        return None;
    }
    let end = body
        .find(|character: char| character.is_whitespace() || character == '/' || character == '>')
        .unwrap_or(body.len());
    let name = body.get(..end)?;
    (!name.is_empty()).then_some(name)
}

fn xml_local_name(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_prefix, local)| local)
}

fn line_bounds_for_offset(contents: &str, offset: usize) -> Option<(usize, usize)> {
    if contents.is_empty() {
        return None;
    }
    let offset = offset.min(contents.len().saturating_sub(1));
    let line_start = contents
        .as_bytes()
        .get(..=offset)?
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |position| position + 1);
    let line_end = contents
        .as_bytes()
        .get(offset..)?
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(contents.len(), |position| offset + position);
    Some((line_start, line_end))
}

fn escape_unescaped_ampersands(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    let mut cursor = 0usize;
    while cursor < text.len() {
        if text.as_bytes().get(cursor) == Some(&b'&') && !is_valid_xml_entity_at(text, cursor) {
            escaped.push_str("&amp;");
            cursor += 1;
            continue;
        }
        let Some(character) = text.get(cursor..).and_then(|value| value.chars().next()) else {
            break;
        };
        escaped.push(character);
        cursor += character.len_utf8();
    }
    escaped
}

fn is_valid_xml_entity_at(text: &str, ampersand_offset: usize) -> bool {
    let Some(rest) = text.get(ampersand_offset + 1..) else {
        return false;
    };
    if let Some((entity, _tail)) = rest.split_once(';')
        && resolve_predefined_entity(entity).is_some()
    {
        return true;
    }
    if let Some(hex) = rest
        .strip_prefix("#x")
        .and_then(|value| value.split_once(';'))
    {
        return !hex.0.is_empty() && hex.0.bytes().all(|byte| byte.is_ascii_hexdigit());
    }
    if let Some(decimal) = rest
        .strip_prefix('#')
        .and_then(|value| value.split_once(';'))
    {
        return !decimal.0.is_empty() && decimal.0.bytes().all(|byte| byte.is_ascii_digit());
    }
    false
}

pub(in crate::lint::bpmn::api) fn find_missing_branch_condition_context(
    contents: &str,
    gateway_id: &str,
) -> Option<MissingBranchConditionContext> {
    let default_flow_id = find_gateway_default_flow_id(contents, gateway_id)?;
    let flows = find_gateway_flow_details(contents, gateway_id, &default_flow_id);
    let missing = flows
        .iter()
        .find(|flow| !flow.is_default && !flow.has_condition)?;
    let duplicate_conditioned_flow_ids = flows
        .iter()
        .filter(|flow| {
            flow.id != missing.id && flow.has_condition && flow.target_ref == missing.target_ref
        })
        .map(|flow| flow.id.clone())
        .collect::<Vec<_>>();
    let duplicate_default_flow_ids = flows
        .iter()
        .filter(|flow| {
            flow.id != missing.id && flow.is_default && flow.target_ref == missing.target_ref
        })
        .map(|flow| flow.id.clone())
        .collect::<Vec<_>>();
    Some(MissingBranchConditionContext {
        flow_id: (missing.id.clone()),
        target_ref: missing.target_ref.clone(),
        flow_span: missing.span.clone(),
        duplicate_conditioned_flow_ids,
        duplicate_default_flow_ids,
    })
}

fn find_gateway_flow_details(
    contents: &str,
    gateway_id: &str,
    default_flow_id: &str,
) -> Vec<GatewayFlowDetail> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut depth = 0usize;
    let mut active_flow: Option<ActiveGatewayFlow> = None;
    let mut flows = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "sequenceFlow") {
                    let source_ref = attribute_value(&reader, &event, "sourceRef");
                    let flow_id = attribute_value(&reader, &event, "id");
                    if source_ref.as_deref() == Some(gateway_id)
                        && let Some(flow_id) = flow_id
                        && let Some(event_end) = reader_position(&reader)
                        && let Some(span) = start_event_span(event_end, &event)
                    {
                        active_flow = Some(ActiveGatewayFlow {
                            depth,
                            id: flow_id.clone(),
                            target_ref: attribute_value(&reader, &event, "targetRef"),
                            span,
                            has_condition: false,
                            is_default: flow_id == default_flow_id,
                        });
                    }
                } else if active_flow.is_some()
                    && is_element(&event, "conditionExpression")
                    && let Some(flow) = active_flow.as_mut()
                {
                    flow.has_condition = true;
                }
            }
            Ok(Event::Empty(event)) => {
                if is_element(&event, "sequenceFlow") {
                    let source_ref = attribute_value(&reader, &event, "sourceRef");
                    let flow_id = attribute_value(&reader, &event, "id");
                    if source_ref.as_deref() == Some(gateway_id)
                        && let Some(flow_id) = flow_id
                        && let Some(event_end) = reader_position(&reader)
                        && let Some(span) = start_event_span(event_end, &event)
                    {
                        flows.push(GatewayFlowDetail {
                            is_default: flow_id == default_flow_id,
                            id: flow_id,
                            target_ref: attribute_value(&reader, &event, "targetRef"),
                            span,
                            has_condition: false,
                        });
                    }
                } else if active_flow.is_some()
                    && is_element(&event, "conditionExpression")
                    && let Some(flow) = active_flow.as_mut()
                {
                    flow.has_condition = true;
                }
            }
            Ok(Event::End(event)) => {
                if local_name(event.name().as_ref()) == "sequenceFlow"
                    && let Some(flow) = active_flow.take()
                {
                    let ActiveGatewayFlow {
                        depth: _flow_depth,
                        id,
                        target_ref,
                        span,
                        has_condition,
                        is_default,
                    } = flow;
                    flows.push(GatewayFlowDetail {
                        id,
                        target_ref,
                        span,
                        has_condition,
                        is_default,
                    });
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return flows,
            Ok(_) => {}
        }
    }
}

fn find_gateway_default_flow_id(contents: &str, gateway_id: &str) -> Option<String> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event))
                if is_element(&event, "exclusiveGateway")
                    || is_element(&event, "inclusiveGateway") =>
            {
                if attribute_value(&reader, &event, "id").as_deref() == Some(gateway_id) {
                    return attribute_value(&reader, &event, "default");
                }
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

pub(in crate::lint::bpmn::api) fn find_bounded_gateway_ids(contents: &str) -> Vec<String> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut ids = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event))
                if is_element(&event, "exclusiveGateway")
                    || is_element(&event, "inclusiveGateway") =>
            {
                if attribute_value(&reader, &event, "default").is_some()
                    && let Some(id) = attribute_value(&reader, &event, "id")
                {
                    ids.push(id);
                }
            }
            Ok(Event::Eof) | Err(_) => return ids,
            Ok(_) => {}
        }
    }
}

pub(in crate::lint::bpmn::api) fn find_gateway_default_span_and_id(
    contents: &str,
    gateway_id: &str,
) -> Option<(std::ops::Range<usize>, String)> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event))
                if is_element(&event, "exclusiveGateway")
                    || is_element(&event, "inclusiveGateway") =>
            {
                if attribute_value(&reader, &event, "id").as_deref() == Some(gateway_id) {
                    let default_flow_id = attribute_value(&reader, &event, "default")?;
                    let span = start_event_span(reader_position(&reader)?, &event)?;
                    return Some((span, default_flow_id));
                }
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

pub(in crate::lint::bpmn::api) fn find_routable_task_spans(
    contents: &str,
) -> Vec<(String, std::ops::Range<usize>)> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut spans = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event)) if is_routable_task_element(&event) => {
                if attribute_value(&reader, &event, "isForCompensation").as_deref() == Some("true")
                {
                    continue;
                }
                if let Some(task_id) = attribute_value(&reader, &event, "id")
                    && let Some(event_end) = reader_position(&reader)
                    && let Some(span) = start_event_span(event_end, &event)
                {
                    spans.push((task_id, span));
                }
            }
            Ok(Event::Eof) | Err(_) => return spans,
            Ok(_) => {}
        }
    }
}

pub(in crate::lint::bpmn::api) fn find_outgoing_flow_summaries(
    contents: &str,
    gateway_id: &str,
) -> Vec<OutgoingFlowSummary> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut depth = 0usize;
    let mut active_flow: Option<(usize, String, bool)> = None;
    let mut flows = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "sequenceFlow") {
                    let source_ref = attribute_value(&reader, &event, "sourceRef");
                    let flow_id = attribute_value(&reader, &event, "id");
                    if source_ref.as_deref() == Some(gateway_id) {
                        active_flow = Some((depth, flow_id.unwrap_or_default(), false));
                    }
                } else if active_flow.is_some()
                    && is_element(&event, "conditionExpression")
                    && let Some((_flow_depth, _flow_id, has_condition)) = active_flow.as_mut()
                {
                    *has_condition = true;
                }
            }
            Ok(Event::Empty(event)) => {
                if is_element(&event, "sequenceFlow") {
                    let source_ref = attribute_value(&reader, &event, "sourceRef");
                    let flow_id = attribute_value(&reader, &event, "id");
                    if source_ref.as_deref() == Some(gateway_id)
                        && let Some(flow_id) = flow_id
                    {
                        flows.push(OutgoingFlowSummary {
                            id: flow_id,
                            has_condition: false,
                        });
                    }
                } else if active_flow.is_some()
                    && is_element(&event, "conditionExpression")
                    && let Some((_flow_depth, _flow_id, has_condition)) = active_flow.as_mut()
                {
                    *has_condition = true;
                }
            }
            Ok(Event::End(event)) => {
                if local_name(event.name().as_ref()) == "sequenceFlow"
                    && let Some((flow_depth, flow_id, has_condition)) = active_flow.take()
                {
                    if !flow_id.is_empty() {
                        flows.push(OutgoingFlowSummary {
                            id: flow_id,
                            has_condition,
                        });
                    }
                    if flow_depth != depth {
                        active_flow = None;
                    }
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return flows,
            Ok(_) => {}
        }
    }
}

fn start_event_span(event_end: usize, event: &BytesStart<'_>) -> Option<std::ops::Range<usize>> {
    let raw: &[u8] = event.as_ref();
    let start = event_end.checked_sub(raw.len() + 2)?;
    Some(start..event_end)
}

pub(in crate::lint::bpmn::api) fn find_gateway_condition_expression_span(
    contents: &str,
    gateway_id: &str,
) -> Option<std::ops::Range<usize>> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut sequence_flow_depth = None;
    let mut depth = 0usize;
    let mut in_condition_expression = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "sequenceFlow")
                    && attribute_value(&reader, &event, "sourceRef").as_deref() == Some(gateway_id)
                {
                    sequence_flow_depth = Some(depth);
                } else if sequence_flow_depth.is_some() && is_element(&event, "conditionExpression")
                {
                    in_condition_expression = true;
                }
            }
            Ok(Event::Empty(event)) => {
                if is_element(&event, "sequenceFlow")
                    && attribute_value(&reader, &event, "sourceRef").as_deref() == Some(gateway_id)
                {
                    sequence_flow_depth = None;
                }
            }
            Ok(Event::Text(event)) if in_condition_expression => {
                return event_text_span(reader_position(&reader)?, event.as_ref());
            }
            Ok(Event::CData(event)) if in_condition_expression => {
                return event_text_span(reader_position(&reader)?, event.as_ref());
            }
            Ok(Event::End(event)) => {
                if local_name(event.name().as_ref()) == "conditionExpression" {
                    in_condition_expression = false;
                }
                if sequence_flow_depth == Some(depth) {
                    sequence_flow_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

pub(in crate::lint::bpmn::api) fn find_gateway_condition_expression_text(
    contents: &str,
    gateway_id: &str,
) -> Option<String> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut sequence_flow_depth = None;
    let mut depth = 0usize;
    let mut in_condition_expression = false;
    let mut condition_text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "sequenceFlow")
                    && attribute_value(&reader, &event, "sourceRef").as_deref() == Some(gateway_id)
                {
                    sequence_flow_depth = Some(depth);
                } else if sequence_flow_depth.is_some() && is_element(&event, "conditionExpression")
                {
                    in_condition_expression = true;
                    condition_text.clear();
                }
            }
            Ok(Event::Empty(event)) => {
                if is_element(&event, "sequenceFlow")
                    && attribute_value(&reader, &event, "sourceRef").as_deref() == Some(gateway_id)
                {
                    sequence_flow_depth = None;
                }
            }
            Ok(Event::Text(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok()?.as_ref());
            }
            Ok(Event::CData(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok()?.as_ref());
            }
            Ok(Event::GeneralRef(event)) if in_condition_expression => {
                let reference = event.decode().ok()?;
                if let Some(entity) = resolve_predefined_entity(reference.as_ref()) {
                    condition_text.push_str(entity);
                } else {
                    condition_text.push('&');
                    condition_text.push_str(reference.as_ref());
                    condition_text.push(';');
                }
            }
            Ok(Event::End(event)) => {
                if local_name(event.name().as_ref()) == "conditionExpression" {
                    if !condition_text.trim().is_empty() {
                        return Some(condition_text.trim().to_string());
                    }
                    in_condition_expression = false;
                }
                if sequence_flow_depth == Some(depth) {
                    sequence_flow_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

fn reader_position(reader: &Reader<&[u8]>) -> Option<usize> {
    usize::try_from(reader.buffer_position()).ok()
}

fn event_text_span(event_end: usize, raw_text: &[u8]) -> Option<std::ops::Range<usize>> {
    let event_start = event_end.checked_sub(raw_text.len())?;
    let leading = raw_text
        .iter()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count();
    let trailing = raw_text
        .iter()
        .rev()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count();
    Some((event_start + leading)..(event_end - trailing))
}

pub(in crate::lint::bpmn::api) fn unsupported_condition_expression_help(condition: &str) -> String {
    let decoded = decode_xml_text(condition);
    if let Some((lhs, operator, rhs)) = variable_to_variable_comparison(&decoded) {
        return format!(
            "Unsupported variable-to-variable comparison `{lhs} {operator} {rhs}`. Emit one boolean such as `hasMoreSections` from the upstream task and route on that boolean, or emit one numeric count such as `sectionsRemaining` and compare it to a numeric literal like `sectionsRemaining > 0`. Do not compare two variables directly in the gateway condition."
        );
    }
    "Use one boolean path such as `approved` or `not approved`, or one numeric comparison from a variable to a numeric literal such as `amount > 100`. For variable-to-variable decisions, emit an upstream boolean route variable and branch on that. Return a minimal unified diff only.".to_string()
}

fn variable_to_variable_comparison(condition: &str) -> Option<(&str, &str, &str)> {
    for operator in ["<=", ">=", "==", "!=", ">", "<"] {
        let Some(index) = condition.find(operator) else {
            continue;
        };
        let lhs = condition[..index].trim();
        let rhs = condition[index + operator.len()..].trim();
        if is_variable_operand_hint(lhs) && is_variable_operand_hint(rhs) {
            return Some((lhs, operator, rhs));
        }
    }
    None
}

fn is_variable_operand_hint(source: &str) -> bool {
    !matches!(source, "true" | "false" | "null")
        && source.parse::<f64>().is_err()
        && is_identifier_path_hint(source)
}

fn is_identifier_path_hint(path: &str) -> bool {
    !path.is_empty() && path.split('.').all(is_identifier_segment_hint)
}

fn is_identifier_segment_hint(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn decode_xml_text(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn is_element(event: &BytesStart<'_>, expected: &str) -> bool {
    local_name(event.name().as_ref()) == expected
}

fn is_routable_task_element(event: &BytesStart<'_>) -> bool {
    matches!(
        local_name(event.name().as_ref()),
        "task"
            | "serviceTask"
            | "scriptTask"
            | "userTask"
            | "manualTask"
            | "businessRuleTask"
            | "sendTask"
            | "receiveTask"
    )
}

fn attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Option<String> {
    for attribute in event.attributes().flatten() {
        if local_name(attribute.key.as_ref()) != attribute_name {
            continue;
        }
        let value = attribute.decode_and_unescape_value(reader.decoder()).ok()?;
        return Some(match value {
            Cow::Borrowed(value) => value.to_string(),
            Cow::Owned(value) => value,
        });
    }
    None
}

fn local_name(raw: &[u8]) -> &str {
    std::str::from_utf8(raw)
        .ok()
        .map_or("", |name| name.rsplit(':').next().unwrap_or(name))
}
