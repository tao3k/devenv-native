use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint_api::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use crate::repeat_condition::{GatewayConditionSummary, parse_gateway_condition_summary};
use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use serde_json::json;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

pub(super) fn undeclared_gateway_condition_output_issues(
    source: &BpmnSourceFile,
) -> Vec<LintIssue> {
    collect_process_contracts(source)
        .into_iter()
        .flat_map(|process| process.undeclared_gateway_condition_output_issues(source))
        .collect()
}

#[derive(Default)]
struct ProcessContract {
    id: String,
    task_outputs: HashMap<String, HashSet<String>>,
    gateways: HashSet<String>,
    flows: Vec<SequenceFlowContract>,
}

impl ProcessContract {
    fn undeclared_gateway_condition_output_issues(self, source: &BpmnSourceFile) -> Vec<LintIssue> {
        let mut issues = Vec::new();
        for flow in &self.flows {
            if !self.gateways.contains(&flow.source_ref) {
                continue;
            }
            let Some(condition) = flow.condition.as_deref() else {
                continue;
            };
            let Some(variable_path) = gateway_condition_variable_path(condition) else {
                continue;
            };
            let producer_ids = self.direct_upstream_task_ids(&flow.source_ref);
            if producer_ids.is_empty() {
                continue;
            }
            let producer_outputs = producer_ids
                .iter()
                .flat_map(|producer_id| {
                    self.task_outputs
                        .get(producer_id)
                        .into_iter()
                        .flat_map(|outputs| outputs.iter().cloned())
                })
                .collect::<HashSet<_>>();
            if producer_outputs.is_empty()
                || declares_gateway_variable(&producer_outputs, &variable_path)
            {
                continue;
            }
            issues.push(undeclared_gateway_condition_output_issue(
                UndeclaredGatewayConditionIssue {
                    source,
                    process_id: &self.id,
                    gateway_id: &flow.source_ref,
                    target_id: &flow.target_ref,
                    condition,
                    variable_path: &variable_path,
                    producer_ids: &producer_ids,
                    producer_outputs: &producer_outputs,
                    condition_span: flow.condition_span.clone(),
                },
            ));
        }
        issues
    }

    fn direct_upstream_task_ids(&self, gateway_id: &str) -> Vec<String> {
        self.flows
            .iter()
            .filter(|flow| flow.target_ref == gateway_id)
            .filter(|flow| self.task_outputs.contains_key(&flow.source_ref))
            .map(|flow| flow.source_ref.clone())
            .collect()
    }
}

#[derive(Default)]
struct SequenceFlowContract {
    source_ref: String,
    target_ref: String,
    condition: Option<String>,
    condition_span: Option<Range<usize>>,
}

#[derive(Default)]
struct ActiveTask {
    id: String,
    outputs: HashSet<String>,
    in_output_association: bool,
    in_output_target_ref: bool,
}

#[derive(Default)]
struct ActiveFlow {
    source_ref: String,
    target_ref: String,
    condition: String,
    condition_span: Option<Range<usize>>,
    in_condition: bool,
}

fn collect_process_contracts(source: &BpmnSourceFile) -> Vec<ProcessContract> {
    ProcessContractCollector::new(source).collect()
}

struct ProcessContractCollector<'a> {
    source: &'a BpmnSourceFile,
    processes: Vec<ProcessContract>,
    active_process: Option<ProcessContract>,
    active_task: Option<ActiveTask>,
    active_flow: Option<ActiveFlow>,
}

impl<'a> ProcessContractCollector<'a> {
    fn new(source: &'a BpmnSourceFile) -> Self {
        Self {
            source,
            processes: Vec::new(),
            active_process: None,
            active_task: None,
            active_flow: None,
        }
    }

    fn collect(mut self) -> Vec<ProcessContract> {
        let mut reader = Reader::from_str(&self.source.contents);
        reader.config_mut().trim_text(false);
        loop {
            match reader.read_event() {
                Ok(Event::Start(event)) => self.handle_start(&reader, &event),
                Ok(Event::Empty(event)) => self.handle_empty(&reader, &event),
                Ok(Event::Text(event)) => self.handle_text(event.decode().ok().as_deref()),
                Ok(Event::CData(event)) => {
                    self.handle_condition_text(event.decode().ok().as_deref());
                }
                Ok(Event::GeneralRef(event)) => {
                    let reference = event.decode().ok();
                    let mut text = String::new();
                    append_entity_reference(&mut text, reference.as_deref());
                    self.handle_condition_text(Some(&text));
                }
                Ok(Event::End(event)) => self.handle_end(event.name().as_ref()),
                Ok(Event::Eof) | Err(_) => return self.processes,
                Ok(_) => {}
            }
        }
    }

    fn handle_start(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        let name = local_name(event.name().as_ref());
        match name.as_str() {
            "process" => {
                self.active_process = Some(ProcessContract {
                    id: attribute_value(reader, event, "id")
                        .unwrap_or_else(|| "unknown".to_string()),
                    ..ProcessContract::default()
                });
            }
            tag if is_task_tag(tag) && self.active_process.is_some() => {
                self.active_task = Some(ActiveTask {
                    id: attribute_value(reader, event, "id")
                        .unwrap_or_else(|| "unknown".to_string()),
                    ..ActiveTask::default()
                });
            }
            "exclusiveGateway" => self.record_gateway(reader, event),
            "sequenceFlow" if self.active_process.is_some() => {
                self.active_flow = Some(ActiveFlow {
                    source_ref: attribute_value(reader, event, "sourceRef").unwrap_or_default(),
                    target_ref: attribute_value(reader, event, "targetRef").unwrap_or_default(),
                    ..ActiveFlow::default()
                });
            }
            "dataOutput" => self.record_task_output(reader, event),
            "dataOutputAssociation" => {
                if let Some(task) = self.active_task.as_mut() {
                    task.in_output_association = true;
                }
            }
            "targetRef" => {
                if let Some(task) = self.active_task.as_mut()
                    && task.in_output_association
                {
                    task.in_output_target_ref = true;
                }
            }
            "conditionExpression" => self.start_condition(reader, event),
            _ => {}
        }
    }

    fn handle_empty(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        match local_name(event.name().as_ref()).as_str() {
            "exclusiveGateway" => self.record_gateway(reader, event),
            "dataOutput" => self.record_task_output(reader, event),
            "sequenceFlow" => {
                if let Some(process) = self.active_process.as_mut() {
                    process.flows.push(SequenceFlowContract {
                        source_ref: attribute_value(reader, event, "sourceRef").unwrap_or_default(),
                        target_ref: attribute_value(reader, event, "targetRef").unwrap_or_default(),
                        ..SequenceFlowContract::default()
                    });
                }
            }
            _ => {}
        }
    }

    fn handle_text(&mut self, text: Option<&str>) {
        if let Some(task) = self.active_task.as_mut()
            && task.in_output_target_ref
            && let Some(text) = text
        {
            task.outputs.extend(parse_output_names(text));
        }
        self.handle_condition_text(text);
    }

    fn handle_condition_text(&mut self, text: Option<&str>) {
        if let Some(flow) = self.active_flow.as_mut()
            && flow.in_condition
            && let Some(text) = text
        {
            flow.condition.push_str(text);
        }
    }

    fn handle_end(&mut self, raw_name: &[u8]) {
        let name = local_name(raw_name);
        match name.as_str() {
            "targetRef" => {
                if let Some(task) = self.active_task.as_mut() {
                    task.in_output_target_ref = false;
                }
            }
            "dataOutputAssociation" => {
                if let Some(task) = self.active_task.as_mut() {
                    task.in_output_association = false;
                }
            }
            "conditionExpression" => {
                if let Some(flow) = self.active_flow.as_mut() {
                    flow.in_condition = false;
                }
            }
            "sequenceFlow" => self.finish_flow(),
            tag if is_task_tag(tag) => self.finish_task(),
            "process" => {
                if let Some(process) = self.active_process.take() {
                    self.processes.push(process);
                }
            }
            _ => {}
        }
    }

    fn record_gateway(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if let (Some(process), Some(id)) = (
            self.active_process.as_mut(),
            attribute_value(reader, event, "id"),
        ) {
            process.gateways.insert(id);
        }
    }

    fn record_task_output(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if let Some(task) = self.active_task.as_mut()
            && let Some(name) = attribute_value(reader, event, "name")
        {
            task.outputs.insert(name);
        }
    }

    fn start_condition(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if let Some(flow) = self.active_flow.as_mut() {
            flow.in_condition = true;
            flow.condition.clear();
            flow.condition_span =
                reader_position(reader).and_then(|event_end| start_event_span(event_end, event));
        }
    }

    fn finish_flow(&mut self) {
        if let (Some(process), Some(flow)) = (self.active_process.as_mut(), self.active_flow.take())
        {
            process.flows.push(SequenceFlowContract {
                source_ref: flow.source_ref,
                target_ref: flow.target_ref,
                condition: (!flow.condition.trim().is_empty())
                    .then(|| flow.condition.trim().to_string()),
                condition_span: flow.condition_span,
            });
        }
    }

    fn finish_task(&mut self) {
        if let (Some(process), Some(task)) = (self.active_process.as_mut(), self.active_task.take())
        {
            process.task_outputs.insert(task.id, task.outputs);
        }
    }
}

fn is_task_tag(tag: &str) -> bool {
    matches!(
        tag,
        "serviceTask" | "userTask" | "manualTask" | "businessRuleTask" | "scriptTask"
    )
}

fn gateway_condition_variable_path(condition: &str) -> Option<String> {
    match parse_gateway_condition_summary(condition)? {
        GatewayConditionSummary::BooleanPath { path, .. } => Some(path),
        GatewayConditionSummary::NumericComparison { lhs, .. } => Some(lhs),
    }
}

fn declares_gateway_variable(outputs: &HashSet<String>, variable_path: &str) -> bool {
    let root = variable_path.split('.').next().unwrap_or(variable_path);
    outputs.contains(variable_path) || outputs.contains(root)
}

struct UndeclaredGatewayConditionIssue<'a> {
    source: &'a BpmnSourceFile,
    process_id: &'a str,
    gateway_id: &'a str,
    target_id: &'a str,
    condition: &'a str,
    variable_path: &'a str,
    producer_ids: &'a [String],
    producer_outputs: &'a HashSet<String>,
    condition_span: Option<Range<usize>>,
}

fn undeclared_gateway_condition_output_issue(
    context: UndeclaredGatewayConditionIssue<'_>,
) -> LintIssue {
    let producer_list = context.producer_ids.join(", ");
    let mut output_list = context.producer_outputs.iter().cloned().collect::<Vec<_>>();
    output_list.sort();
    let output_summary = output_list.join(", ");
    let mut issue = LintIssue::new(
        "bpmn.undeclared_gateway_condition_output",
        "Gateway condition variable is not declared by upstream task outputs",
        format!(
            "Process '{}' gateway '{}' routes to '{}' with condition `{}`, but direct upstream task(s) [{producer_list}] do not declare native BPMN output '{}'.",
            context.process_id,
            context.gateway_id,
            context.target_id,
            context.condition,
            context.variable_path
        ),
        "Gateway conditions resolve against runtime variables. A task immediately before a gateway must declare any route variable it is expected to produce through native BPMN output metadata, and its prompt should say to return that JSON field.",
        vec![
            format!("Add '{}' as a native BPMN data output or output association target on upstream task(s) [{producer_list}].", context.variable_path),
            format!("Update the same upstream task prompt to return JSON with boolean or numeric field '{}', matching the gateway condition type.", context.variable_path),
            "Keep the gateway condition unchanged after the producer declares and emits the variable.".to_string(),
        ],
        format!(
            "Repair process '{}' by aligning gateway '{}' condition `{}` with upstream native BPMN outputs. Add `{}` to task output metadata for task(s) [{producer_list}] and update their prompt to return JSON field `{}`. Preserve the branch target '{}' and keep the condition inside the bounded gateway subset.",
            context.process_id,
            context.gateway_id,
            context.condition,
            context.variable_path,
            context.variable_path,
            context.target_id
        ),
        json!({
            "process_id": context.process_id,
            "gateway_id": context.gateway_id,
            "target_id": context.target_id,
            "condition": context.condition,
            "variable_path": context.variable_path,
            "producer_task_ids": context.producer_ids,
            "producer_outputs": output_list,
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "bpmn.native.gateway.data_contract.v1",
        "strategy": "declare_gateway_condition_variable_on_upstream_task",
        "actions": [{
            "op": "add_native_bpmn_output",
            "tasks": context.producer_ids,
            "output": context.variable_path,
        }, {
            "op": "update_task_prompt",
            "tasks": context.producer_ids,
            "requires": format!("return JSON field `{}`", context.variable_path)
        }, {
            "op": "keep_gateway_condition",
            "gateway": context.gateway_id,
            "condition": context.condition
        }],
        "forbid": [
            "routing on variables missing from direct upstream task outputs",
            "renaming the gateway condition without updating the producer prompt and outputs"
        ]
    }));
    if let Some(span) = context.condition_span {
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            &context.source.source_id,
            LintSourceSpan::new(span.start, span.end),
            format!(
                "condition uses undeclared upstream output `{}`",
                context.variable_path
            ),
            format!(
                "Add `{}` to native BPMN task outputs and prompt JSON on upstream task(s) [{producer_list}]. Current outputs: {output_summary}.",
                context.variable_path
            ),
        ));
    }
    issue
}

fn parse_output_names(text: &str) -> Vec<String> {
    text.split([',', '\n', '\t', ' '])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn reader_position(reader: &Reader<&[u8]>) -> Option<usize> {
    usize::try_from(reader.buffer_position()).ok()
}

fn start_event_span(event_end: usize, event: &BytesStart<'_>) -> Option<Range<usize>> {
    let raw: &[u8] = event.as_ref();
    let start = event_end.checked_sub(raw.len() + 2)?;
    Some(start..event_end)
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

fn local_name(name: &[u8]) -> String {
    let raw = std::str::from_utf8(name).unwrap_or_default();
    raw.rsplit_once(':')
        .map_or(raw, |(_, local)| local)
        .to_string()
}

fn append_entity_reference(target: &mut String, reference: Option<&str>) {
    if let Some(reference) = reference
        && let Some(resolved) = resolve_predefined_entity(reference)
    {
        target.push_str(resolved);
    }
}
