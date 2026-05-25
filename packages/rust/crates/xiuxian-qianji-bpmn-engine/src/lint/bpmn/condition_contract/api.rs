use super::{
    ActiveGatewayFlow, BpmnPackage, BpmnSourceFile, Event, GatewayConditionSummary, LintIssue,
    Reader, ambiguous_boolean_condition_issue, ambiguous_boolean_path_kind,
    append_entity_reference, attribute_value, collect_gateway_ids,
    collect_static_interaction_choice_outputs, grouped_unsupported_gateway_condition_issues,
    is_boolean_interaction_choice_value, is_element, local_name,
    non_boolean_interaction_choice_condition_issue, parse_gateway_condition_summary,
    source_ambiguous_boolean_condition_issue, source_unsupported_gateway_condition,
};

pub(in crate::lint::bpmn) fn ambiguous_boolean_gateway_condition_issues(
    source: &BpmnSourceFile,
    package: &BpmnPackage,
) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let interaction_outputs = collect_static_interaction_choice_outputs(&source.contents);
    for process in &package.processes {
        for edge in &process.edges {
            let Some(condition) = edge.condition_expression.as_deref() else {
                continue;
            };
            let Some(GatewayConditionSummary::BooleanPath { path, .. }) =
                parse_gateway_condition_summary(condition)
            else {
                continue;
            };
            let gateway_id = process
                .nodes
                .get(edge.from as usize)
                .map_or_else(|| edge.from.to_string(), |node| node.bpmn_id.to_string());
            if let Some(interaction_output) = interaction_outputs
                .iter()
                .find(|output| output.output == path)
                && !interaction_output
                    .choice_values
                    .iter()
                    .all(|value| is_boolean_interaction_choice_value(value))
            {
                issues.push(non_boolean_interaction_choice_condition_issue(
                    source,
                    process.key.process_id.as_ref(),
                    &gateway_id,
                    condition,
                    &path,
                    interaction_output,
                ));
                continue;
            }
            let Some(kind) = ambiguous_boolean_path_kind(&path) else {
                continue;
            };
            issues.push(ambiguous_boolean_condition_issue(
                source,
                process.key.process_id.as_ref(),
                &gateway_id,
                condition,
                &path,
                kind,
            ));
        }
    }
    issues
}

pub(in crate::lint::bpmn) fn ambiguous_boolean_gateway_condition_source_issues(
    source: &BpmnSourceFile,
) -> Vec<LintIssue> {
    let gateway_ids = collect_gateway_ids(&source.contents);
    if gateway_ids.is_empty() {
        return Vec::new();
    }

    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(false);
    let mut depth = 0usize;
    let mut current_process_id: Option<String> = None;
    let mut active_flow: Option<ActiveGatewayFlow> = None;
    let mut in_condition_expression = false;
    let mut condition_text = String::new();
    let mut issues = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "process") {
                    current_process_id = attribute_value(&reader, &event, "id");
                } else if is_element(&event, "sequenceFlow") {
                    if let Some(source_ref) = attribute_value(&reader, &event, "sourceRef")
                        && gateway_ids.contains(&source_ref)
                    {
                        active_flow = Some(ActiveGatewayFlow {
                            gateway_id: source_ref,
                            process_id: current_process_id
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            depth,
                        });
                    }
                } else if active_flow.is_some() && is_element(&event, "conditionExpression") {
                    in_condition_expression = true;
                    condition_text.clear();
                }
            }
            Ok(Event::Text(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok().as_deref().unwrap_or_default());
            }
            Ok(Event::CData(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok().as_deref().unwrap_or_default());
            }
            Ok(Event::GeneralRef(event)) if in_condition_expression => {
                let reference = event.decode().ok();
                append_entity_reference(&mut condition_text, reference.as_deref());
            }
            Ok(Event::End(event)) => {
                let event_name = event.name();
                let name = local_name(event_name.as_ref());
                if name == "conditionExpression" {
                    let condition = condition_text.trim();
                    if let Some(issue) = source_ambiguous_boolean_condition_issue(
                        source,
                        active_flow.as_ref(),
                        Some(condition),
                    ) {
                        issues.push(issue);
                    }
                    condition_text.clear();
                    in_condition_expression = false;
                }
                if active_flow.as_ref().is_some_and(|flow| flow.depth == depth) {
                    active_flow = None;
                }
                if name == "process" {
                    current_process_id = None;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return issues,
            Ok(_) => {}
        }
    }
}

pub(in crate::lint::bpmn) fn unsupported_gateway_condition_source_issues(
    source: &BpmnSourceFile,
) -> Vec<LintIssue> {
    let gateway_ids = collect_gateway_ids(&source.contents);
    if gateway_ids.is_empty() {
        return Vec::new();
    }

    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(false);
    let mut depth = 0usize;
    let mut current_process_id: Option<String> = None;
    let mut active_flow: Option<ActiveGatewayFlow> = None;
    let mut in_condition_expression = false;
    let mut condition_text = String::new();
    let mut conditions = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "process") {
                    current_process_id = attribute_value(&reader, &event, "id");
                } else if is_element(&event, "sequenceFlow") {
                    if let Some(source_ref) = attribute_value(&reader, &event, "sourceRef")
                        && gateway_ids.contains(&source_ref)
                    {
                        active_flow = Some(ActiveGatewayFlow {
                            gateway_id: source_ref,
                            process_id: current_process_id
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            depth,
                        });
                    }
                } else if active_flow.is_some() && is_element(&event, "conditionExpression") {
                    in_condition_expression = true;
                    condition_text.clear();
                }
            }
            Ok(Event::Text(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok().as_deref().unwrap_or_default());
            }
            Ok(Event::CData(event)) if in_condition_expression => {
                condition_text.push_str(event.decode().ok().as_deref().unwrap_or_default());
            }
            Ok(Event::GeneralRef(event)) if in_condition_expression => {
                let reference = event.decode().ok();
                append_entity_reference(&mut condition_text, reference.as_deref());
            }
            Ok(Event::End(event)) => {
                let event_name = event.name();
                let name = local_name(event_name.as_ref());
                if name == "conditionExpression" {
                    let condition = condition_text.trim();
                    if let Some(condition) =
                        source_unsupported_gateway_condition(active_flow.as_ref(), Some(condition))
                    {
                        conditions.push(condition);
                    }
                    condition_text.clear();
                    in_condition_expression = false;
                }
                if active_flow.as_ref().is_some_and(|flow| flow.depth == depth) {
                    active_flow = None;
                }
                if name == "process" {
                    current_process_id = None;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => {
                return grouped_unsupported_gateway_condition_issues(source, conditions);
            }
            Ok(_) => {}
        }
    }
}
