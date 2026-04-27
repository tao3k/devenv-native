use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint_api::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use serde_json::{Value, json};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

const SUPPORTED_INTERACTION_TYPES: &[&str] = &["input", "confirm", "choice", "choice_input"];

pub(super) fn qianji_extension_issues(source: &BpmnSourceFile) -> Vec<LintIssue> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);
    let mut state = ExtensionScanState::default();
    let mut issues = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => state.handle_start(source, &mut reader, &event, &mut issues),
            Ok(Event::Empty(event)) => state.handle_empty(source, &reader, &event, &mut issues),
            Ok(Event::Text(event)) => {
                if let Ok(text) = event.decode() {
                    state.capture_text(&text);
                }
            }
            Ok(Event::End(event)) => state.handle_end(&event),
            Ok(Event::Eof) | Err(_) => return state.finish(source, issues),
            Ok(_) => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextCaptureTarget {
    Inputs,
    Prompt,
    Outputs,
    Tools,
}

#[derive(Default)]
struct ExtensionScanState {
    active_config: ActiveConfig,
    active_node_ids: Vec<Option<String>>,
    active_node_names: Vec<Option<String>>,
    active_node_kinds: Vec<String>,
    active_node_spans: Vec<Option<Range<usize>>>,
    dynamic_choice_refs: Vec<DynamicChoiceRefContract>,
    producers_by_output: HashMap<String, Vec<OutputProducerContract>>,
    node_configs_by_id: HashMap<String, NodeConfigContract>,
    sequence_flows: Vec<SequenceFlowContract>,
    text_capture: Option<TextCaptureTarget>,
}

impl ExtensionScanState {
    fn handle_start(
        &mut self,
        source: &BpmnSourceFile,
        reader: &mut Reader<&[u8]>,
        event: &BytesStart<'_>,
        issues: &mut Vec<LintIssue>,
    ) {
        if is_bpmn_sequence_flow(event) {
            collect_sequence_flow_contract(reader, event, &mut self.sequence_flows);
        }
        if is_bpmn_node_with_qianji_config(event) {
            self.active_node_ids
                .push(attribute_value(reader, event, "id"));
            self.active_node_names
                .push(attribute_value(reader, event, "name"));
            self.active_node_kinds
                .push(local_name(event.name().as_ref()).to_string());
            self.active_node_spans.push(
                reader_position(reader).and_then(|event_end| start_event_span(event_end, event)),
            );
        } else if is_qianji_config(event) {
            self.active_config = ActiveConfig {
                node_id: self.active_node_ids.last().cloned().flatten(),
                node_name: self.active_node_names.last().cloned().flatten(),
                node_kind: self.active_node_kinds.last().cloned(),
                node_span: self.active_node_spans.last().cloned().flatten(),
                ..ActiveConfig::default()
            };
        } else if is_qianji_inputs(event) {
            self.text_capture = Some(TextCaptureTarget::Inputs);
        } else if is_qianji_prompt(event) {
            self.text_capture = Some(TextCaptureTarget::Prompt);
        } else if is_qianji_outputs(event) {
            self.text_capture = Some(TextCaptureTarget::Outputs);
            self.active_config.outputs_span =
                reader_position(reader).and_then(|event_end| start_event_span(event_end, event));
        } else if is_qianji_tools(event) {
            self.text_capture = Some(TextCaptureTarget::Tools);
        } else if is_qianji_output_schema(event) {
            collect_output_schema_contract(reader, event, &mut self.active_config);
        } else if is_qianji_interaction(event) {
            self.handle_interaction_start(source, reader, event, issues);
        }
    }

    fn handle_empty(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        issues: &mut Vec<LintIssue>,
    ) {
        if is_bpmn_sequence_flow(event) {
            collect_sequence_flow_contract(reader, event, &mut self.sequence_flows);
        } else if is_qianji_output_schema(event) {
            collect_output_schema_contract(reader, event, &mut self.active_config);
        } else if is_qianji_interaction(event) {
            collect_empty_interaction_issue(source, reader, event, issues);
        }
    }

    fn handle_interaction_start(
        &mut self,
        source: &BpmnSourceFile,
        reader: &mut Reader<&[u8]>,
        event: &BytesStart<'_>,
        issues: &mut Vec<LintIssue>,
    ) {
        let interaction_type = attribute_value(reader, event, "type");
        match interaction_type.as_deref() {
            Some(kind) if SUPPORTED_INTERACTION_TYPES.contains(&kind) => {
                let mut contract = read_interaction_contract(reader);
                contract.interaction_type = Some(kind.to_string());
                self.collect_interaction_contract(source, kind, &contract, issues);
            }
            Some(kind) => issues.push(unsupported_interaction_type_issue(source, kind)),
            None => issues.push(missing_interaction_type_issue(source)),
        }
    }

    fn collect_interaction_contract(
        &mut self,
        source: &BpmnSourceFile,
        interaction_type: &str,
        contract: &InteractionContract,
        issues: &mut Vec<LintIssue>,
    ) {
        self.active_config.interactions.push(contract.clone());
        if requires_choice_contract(interaction_type) && !contract.has_choice_contract {
            issues.push(missing_choice_contract_issue(source, interaction_type));
            return;
        }
        if let Some(issue) =
            undeclared_interaction_result_issue(source, contract, &self.active_config)
        {
            issues.push(issue);
            return;
        }
        if let Some(issue) =
            ambiguous_interaction_outputs_issue(source, contract, &self.active_config)
        {
            issues.push(issue);
            return;
        }
        issues.extend(ambiguous_question_choices_ref_issue(source, contract));
        collect_dynamic_choice_ref_contract(
            &self.active_config,
            contract,
            &mut self.dynamic_choice_refs,
        );
    }

    fn capture_text(&mut self, text: &str) {
        match self.text_capture {
            Some(TextCaptureTarget::Inputs) => self.capture_inputs_text(text),
            Some(TextCaptureTarget::Prompt) => self.capture_prompt_text(text),
            Some(TextCaptureTarget::Outputs) => self.capture_outputs_text(text),
            Some(TextCaptureTarget::Tools) => self.capture_tools_text(text),
            None => {}
        }
    }

    fn capture_inputs_text(&mut self, text: &str) {
        self.active_config.inputs_ordered = parse_output_names(text);
    }

    fn capture_prompt_text(&mut self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        if !self.active_config.prompt_text.is_empty() {
            self.active_config.prompt_text.push(' ');
        }
        self.active_config.prompt_text.push_str(text);
    }

    fn capture_outputs_text(&mut self, text: &str) {
        self.active_config.outputs_text = Some(text.trim().to_string());
        self.active_config.outputs_ordered = parse_output_names(text);
        self.active_config
            .outputs
            .extend(self.active_config.outputs_ordered.iter().cloned());
    }

    fn capture_tools_text(&mut self, text: &str) {
        self.active_config.tools_ordered = parse_output_names(text);
    }

    fn handle_end(&mut self, event: &quick_xml::events::BytesEnd<'_>) {
        if is_qianji_text_capture_end_event(event) {
            self.text_capture = None;
        } else if is_end_event_name(event, "qianji:config") {
            collect_output_producer_contracts(&self.active_config, &mut self.producers_by_output);
            collect_node_config_contract(&self.active_config, &mut self.node_configs_by_id);
            self.active_config = ActiveConfig::default();
            self.text_capture = None;
        } else if is_bpmn_node_end(event) {
            self.active_node_ids.pop();
            self.active_node_names.pop();
            self.active_node_kinds.pop();
            self.active_node_spans.pop();
        }
    }

    fn finish(self, source: &BpmnSourceFile, mut issues: Vec<LintIssue>) -> Vec<LintIssue> {
        issues.extend(static_interaction_producer_issues(
            source,
            &self.node_configs_by_id,
            &self.sequence_flows,
        ));
        issues.extend(dynamic_choice_output_schema_issues(
            source,
            &self.dynamic_choice_refs,
            &self.producers_by_output,
        ));
        issues.extend(dynamic_choice_input_binding_issues(
            source,
            &self.node_configs_by_id,
            &self.sequence_flows,
        ));
        issues.extend(redundant_user_answer_store_issues(
            source,
            &self.node_configs_by_id,
            &self.sequence_flows,
        ));
        issues
    }
}

fn is_qianji_text_capture_end_event(event: &quick_xml::events::BytesEnd<'_>) -> bool {
    is_end_event_name(event, "qianji:inputs")
        || is_end_event_name(event, "qianji:prompt")
        || is_end_event_name(event, "qianji:outputs")
        || is_end_event_name(event, "qianji:tools")
}

fn collect_empty_interaction_issue(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    issues: &mut Vec<LintIssue>,
) {
    let interaction_type = attribute_value(reader, event, "type");
    match interaction_type.as_deref() {
        Some(kind) if SUPPORTED_INTERACTION_TYPES.contains(&kind) => {
            if requires_choice_contract(kind) {
                issues.push(missing_choice_contract_issue(source, kind));
            }
        }
        Some(kind) => issues.push(unsupported_interaction_type_issue(source, kind)),
        None => issues.push(missing_interaction_type_issue(source)),
    }
}

#[derive(Default)]
struct ActiveConfig {
    node_id: Option<String>,
    node_name: Option<String>,
    node_kind: Option<String>,
    node_span: Option<Range<usize>>,
    inputs_ordered: Vec<String>,
    prompt_text: String,
    outputs: HashSet<String>,
    outputs_ordered: Vec<String>,
    outputs_text: Option<String>,
    outputs_span: Option<Range<usize>>,
    tools_ordered: Vec<String>,
    output_schema_kinds: HashMap<String, String>,
    interactions: Vec<InteractionContract>,
}

#[derive(Clone)]
struct OutputProducerContract {
    node_id: Option<String>,
    output_name: String,
    output_schema_kind: Option<String>,
    outputs_text: Option<String>,
    outputs_span: Option<Range<usize>>,
}

struct DynamicChoiceRefContract {
    node_id: Option<String>,
    choices_ref: String,
}

#[derive(Clone)]
struct NodeConfigContract {
    node_id: String,
    node_name: Option<String>,
    node_kind: String,
    node_span: Option<Range<usize>>,
    inputs_ordered: Vec<String>,
    prompt_text: String,
    outputs: HashSet<String>,
    outputs_ordered: Vec<String>,
    output_schema_kinds: HashMap<String, String>,
    tools_ordered: Vec<String>,
    interactions: Vec<InteractionContract>,
}

struct SequenceFlowContract {
    source_ref: String,
    target_ref: String,
}

fn is_qianji_interaction(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:interaction")
}

fn is_qianji_config(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:config")
}

fn is_qianji_inputs(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:inputs")
}

fn is_qianji_prompt(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:prompt")
}

fn is_qianji_outputs(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:outputs")
}

fn is_qianji_tools(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:tools")
}

fn is_qianji_output_schema(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:outputSchema")
}

fn is_qianji_choice(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:choice")
}

fn is_qianji_choices(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:choices")
}

fn is_qianji_question(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:question")
}

fn is_qianji_result(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:result")
}

fn is_event_name(event: &BytesStart<'_>, expected: &str) -> bool {
    let name = event.name();
    std::str::from_utf8(name.as_ref()).unwrap_or_default() == expected
}

fn is_end_event_name(event: &quick_xml::events::BytesEnd<'_>, expected: &str) -> bool {
    let name = event.name();
    std::str::from_utf8(name.as_ref()).unwrap_or_default() == expected
}

fn is_bpmn_node_with_qianji_config(event: &BytesStart<'_>) -> bool {
    matches!(
        local_name(event.name().as_ref()),
        "serviceTask"
            | "userTask"
            | "manualTask"
            | "sendTask"
            | "task"
            | "scriptTask"
            | "businessRuleTask"
    )
}

fn is_bpmn_sequence_flow(event: &BytesStart<'_>) -> bool {
    local_name(event.name().as_ref()) == "sequenceFlow"
}

fn is_bpmn_node_end(event: &quick_xml::events::BytesEnd<'_>) -> bool {
    matches!(
        local_name(event.name().as_ref()),
        "serviceTask"
            | "userTask"
            | "manualTask"
            | "sendTask"
            | "task"
            | "scriptTask"
            | "businessRuleTask"
    )
}

fn requires_choice_contract(interaction_type: &str) -> bool {
    interaction_type == "choice" || interaction_type == "choice_input"
}

#[derive(Clone, Default)]
struct InteractionContract {
    interaction_type: Option<String>,
    has_choice_contract: bool,
    question_ref: Option<String>,
    choices_ref: Option<String>,
    result_output: Option<String>,
    question_span: Option<std::ops::Range<usize>>,
    choices_span: Option<std::ops::Range<usize>>,
    result_span: Option<std::ops::Range<usize>>,
}

fn read_interaction_contract(reader: &mut Reader<&[u8]>) -> InteractionContract {
    let mut contract = InteractionContract::default();
    let mut depth = 0usize;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                collect_interaction_child_contract(reader, &event, &mut contract);
                depth += 1;
            }
            Ok(Event::Empty(event)) => {
                collect_interaction_child_contract(reader, &event, &mut contract);
            }
            Ok(Event::End(event)) => {
                if depth == 0 && is_end_event_name(&event, "qianji:interaction") {
                    return contract;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return contract,
            Ok(_) => {}
        }
    }
}

fn collect_interaction_child_contract(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    contract: &mut InteractionContract,
) {
    if is_qianji_choice(event) && has_non_empty_attribute(reader, event, "value") {
        contract.has_choice_contract = true;
    }
    if is_qianji_choices(event)
        && let Some(reference) = attribute_value(reader, event, "ref")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    {
        contract.has_choice_contract = true;
        let span = reader_position(reader).and_then(|event_end| start_event_span(event_end, event));
        contract.choices_ref.get_or_insert(reference);
        if let Some(span) = span {
            contract.choices_span.get_or_insert(span);
        }
    }
    if is_qianji_question(event)
        && let Some(reference) = attribute_value(reader, event, "ref")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    {
        let span = reader_position(reader).and_then(|event_end| start_event_span(event_end, event));
        contract.question_ref.get_or_insert(reference);
        if let Some(span) = span {
            contract.question_span.get_or_insert(span);
        }
    }
    if is_qianji_result(event)
        && let Some(output) = attribute_value(reader, event, "output")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    {
        let span = reader_position(reader).and_then(|event_end| start_event_span(event_end, event));
        contract.result_output.get_or_insert(output);
        if let Some(span) = span {
            contract.result_span.get_or_insert(span);
        }
    }
}

fn collect_output_schema_contract(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    active_config: &mut ActiveConfig,
) {
    let Some(name) = attribute_value(reader, event, "name")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let Some(kind) = attribute_value(reader, event, "kind")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    active_config.output_schema_kinds.insert(name, kind);
}

fn collect_dynamic_choice_ref_contract(
    active_config: &ActiveConfig,
    contract: &InteractionContract,
    dynamic_choice_refs: &mut Vec<DynamicChoiceRefContract>,
) {
    let Some(choices_ref) = contract.choices_ref.as_ref() else {
        return;
    };
    if contract.question_ref.as_ref() == Some(choices_ref) {
        return;
    }
    dynamic_choice_refs.push(DynamicChoiceRefContract {
        node_id: active_config.node_id.clone(),
        choices_ref: choices_ref.clone(),
    });
}

fn collect_output_producer_contracts(
    active_config: &ActiveConfig,
    producers_by_output: &mut HashMap<String, Vec<OutputProducerContract>>,
) {
    for output_name in &active_config.outputs {
        producers_by_output
            .entry(output_name.clone())
            .or_default()
            .push(OutputProducerContract {
                node_id: active_config.node_id.clone(),
                output_name: output_name.clone(),
                output_schema_kind: active_config.output_schema_kinds.get(output_name).cloned(),
                outputs_text: active_config.outputs_text.clone(),
                outputs_span: active_config.outputs_span.clone(),
            });
    }
}

fn collect_node_config_contract(
    active_config: &ActiveConfig,
    configs_by_id: &mut HashMap<String, NodeConfigContract>,
) {
    let Some(node_id) = active_config.node_id.clone() else {
        return;
    };
    let Some(node_kind) = active_config.node_kind.clone() else {
        return;
    };
    configs_by_id.insert(
        node_id.clone(),
        NodeConfigContract {
            node_id,
            node_name: active_config.node_name.clone(),
            node_kind,
            node_span: active_config.node_span.clone(),
            inputs_ordered: active_config.inputs_ordered.clone(),
            prompt_text: active_config.prompt_text.clone(),
            outputs: active_config.outputs.clone(),
            outputs_ordered: active_config.outputs_ordered.clone(),
            output_schema_kinds: active_config.output_schema_kinds.clone(),
            tools_ordered: active_config.tools_ordered.clone(),
            interactions: active_config.interactions.clone(),
        },
    );
}

fn collect_sequence_flow_contract(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    sequence_flows: &mut Vec<SequenceFlowContract>,
) {
    let Some(source_ref) = attribute_value(reader, event, "sourceRef")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let Some(target_ref) = attribute_value(reader, event, "targetRef")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    sequence_flows.push(SequenceFlowContract {
        source_ref,
        target_ref,
    });
}

fn static_interaction_producer_issues(
    source: &BpmnSourceFile,
    node_configs_by_id: &HashMap<String, NodeConfigContract>,
    sequence_flows: &[SequenceFlowContract],
) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let mut seen_pairs = HashSet::new();
    for flow in sequence_flows {
        let Some(service) = node_configs_by_id.get(&flow.source_ref) else {
            continue;
        };
        if service.node_kind != "serviceTask" {
            continue;
        }
        let Some(user_task) = node_configs_by_id.get(&flow.target_ref) else {
            continue;
        };
        if user_task.node_kind != "userTask" {
            continue;
        }
        if !service.inputs_ordered.is_empty() || !service.tools_ordered.is_empty() {
            continue;
        }
        if !seen_pairs.insert((service.node_id.clone(), user_task.node_id.clone())) {
            continue;
        }
        if let Some(issue) = dynamic_static_interaction_producer_issue(source, service, user_task) {
            issues.push(issue);
            continue;
        }
        if let Some(issue) = redundant_static_interaction_producer_issue(source, service, user_task)
        {
            issues.push(issue);
        }
    }
    issues
}

fn dynamic_choice_input_binding_issues(
    source: &BpmnSourceFile,
    node_configs_by_id: &HashMap<String, NodeConfigContract>,
    sequence_flows: &[SequenceFlowContract],
) -> Vec<LintIssue> {
    let mut findings = Vec::new();
    let mut seen_pairs = HashSet::new();
    for flow in sequence_flows {
        let Some(service) = node_configs_by_id.get(&flow.source_ref) else {
            continue;
        };
        if service.node_kind != "serviceTask" || service.inputs_ordered.is_empty() {
            continue;
        }
        if !service.tools_ordered.is_empty() {
            continue;
        }
        let Some(user_task) = node_configs_by_id.get(&flow.target_ref) else {
            continue;
        };
        if user_task.node_kind != "userTask" {
            continue;
        }
        if !seen_pairs.insert((service.node_id.clone(), user_task.node_id.clone())) {
            continue;
        }
        let Some(interaction) = dynamic_choice_interaction(service, user_task) else {
            continue;
        };
        let Some(choices_ref) = interaction.choices_ref.as_deref() else {
            continue;
        };
        if service
            .output_schema_kinds
            .get(choices_ref)
            .map(String::as_str)
            != Some("choice_array")
        {
            continue;
        }
        let unbound_inputs = service
            .inputs_ordered
            .iter()
            .filter(|input| !prompt_mentions_input(&service.prompt_text, input))
            .cloned()
            .collect::<Vec<_>>();
        if unbound_inputs.is_empty() {
            continue;
        }
        findings.push(DynamicChoiceInputBindingFinding::new(
            service,
            user_task,
            interaction,
            unbound_inputs,
        ));
    }
    if findings.is_empty() {
        Vec::new()
    } else {
        vec![dynamic_choice_input_binding_issue(source, &findings)]
    }
}

fn redundant_user_answer_store_issues(
    source: &BpmnSourceFile,
    node_configs_by_id: &HashMap<String, NodeConfigContract>,
    sequence_flows: &[SequenceFlowContract],
) -> Vec<LintIssue> {
    let mut findings = Vec::new();
    let mut seen_services = HashSet::new();
    for incoming in sequence_flows {
        let Some(user_task) = node_configs_by_id.get(&incoming.source_ref) else {
            continue;
        };
        if user_task.node_kind != "userTask" {
            continue;
        }
        let Some(service) = node_configs_by_id.get(&incoming.target_ref) else {
            continue;
        };
        if !is_redundant_user_answer_store_task(service) {
            continue;
        }
        if !seen_services.insert(service.node_id.clone()) {
            continue;
        }
        let Some(input_name) = service.inputs_ordered.first() else {
            continue;
        };
        let Some(output_name) = service.outputs_ordered.first() else {
            continue;
        };
        if !user_task.outputs.contains(input_name) {
            continue;
        }
        let outgoing = sequence_flows
            .iter()
            .filter(|flow| flow.source_ref == service.node_id)
            .collect::<Vec<_>>();
        if outgoing.len() != 1 {
            continue;
        }
        let next_task_id = outgoing[0].target_ref.clone();
        let Some(next_task) = node_configs_by_id.get(&next_task_id) else {
            continue;
        };
        if next_task.node_kind != "userTask" && next_task.node_kind != "serviceTask" {
            continue;
        }
        let mut consumers = node_configs_by_id
            .values()
            .filter(|node| node.inputs_ordered.iter().any(|input| input == output_name))
            .map(|node| node.node_id.clone())
            .collect::<Vec<_>>();
        consumers.sort();
        if consumers.is_empty() {
            continue;
        }
        findings.push(UserAnswerStoreFinding {
            user_task_id: user_task.node_id.clone(),
            service_id: service.node_id.clone(),
            next_task_id,
            node_span: service.node_span.clone(),
            input_name: input_name.clone(),
            output_name: output_name.clone(),
            consumers,
            prompt_text: service.prompt_text.clone(),
        });
    }

    if findings.is_empty() {
        Vec::new()
    } else {
        vec![redundant_user_answer_store_issue(source, &findings)]
    }
}

fn is_redundant_user_answer_store_task(service: &NodeConfigContract) -> bool {
    if service.node_kind != "serviceTask" {
        return false;
    }
    if !service.tools_ordered.is_empty()
        || !service.output_schema_kinds.is_empty()
        || !service.interactions.is_empty()
    {
        return false;
    }
    if service.inputs_ordered.len() != 1 || service.outputs_ordered.len() != 1 {
        return false;
    }
    let id_or_name_marks_store = token_list_contains(&service.node_id, "store")
        || service
            .node_name
            .as_deref()
            .is_some_and(|name| token_list_contains(name, "store"));
    if !id_or_name_marks_store {
        return false;
    }
    let prompt = service.prompt_text.to_ascii_lowercase();
    token_list_contains(&prompt, "store") || prompt.contains("store it as")
}

fn token_list_contains(text: &str, expected: &str) -> bool {
    text.split(|character: char| !character.is_ascii_alphanumeric())
        .any(|token| token.eq_ignore_ascii_case(expected))
}

fn redundant_user_answer_store_issue(
    source: &BpmnSourceFile,
    findings: &[UserAnswerStoreFinding],
) -> LintIssue {
    let source_id = &source.source_id;
    let first = &findings[0];
    let target_pairs = findings
        .iter()
        .map(|finding| {
            format!(
                "{} -> {} -> {} (replace {} with {})",
                finding.user_task_id,
                finding.service_id,
                finding.next_task_id,
                finding.output_name,
                finding.input_name
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let issue_count_suffix = if findings.len() == 1 {
        String::new()
    } else {
        format!(
            " This file has {} matching store serviceTasks.",
            findings.len()
        )
    };
    let expected_xml = redundant_user_answer_store_expected_xml(findings);
    let mut issue = LintIssue::new(
        "bpmn.redundant_user_answer_store_service_task",
        "User answer store serviceTask should not invoke an LLM",
        format!(
            "Source '{source_id}' serviceTask '{}' consumes userTask answer '{}' and only stores it as '{}' before '{}'.{issue_count_suffix}",
            first.service_id, first.input_name, first.output_name, first.next_task_id,
        ),
        "A qianji userTask result is already persisted as a workflow variable. A no-tool, one-input, one-output store serviceTask immediately after the userTask adds an unnecessary LLM boundary and can dominate workflow latency without adding BPMN state semantics.",
        vec![
            format!(
                "Remove serviceTask '{}' and reconnect userTask '{}' directly to '{}'.",
                first.service_id, first.user_task_id, first.next_task_id
            ),
            format!(
                "Replace downstream qianji:inputs value '{}' with the original userTask result '{}'.",
                first.output_name, first.input_name
            ),
            format!("Apply the same repair pattern to: {target_pairs}."),
            "Keep an LLM serviceTask only when it derives route variables, summaries, decisions, or tool-backed outputs that are not already the userTask result.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by deleting redundant no-tool user-answer store serviceTasks, reconnecting their incoming sequenceFlow sources to their outgoing targets, and replacing downstream qianji:inputs aliases with the original userTask result variable."
        ),
        json!({
            "source_id": source_id,
            "first_store_task_id": first.service_id,
            "findings": findings.iter().map(UserAnswerStoreFinding::to_evidence).collect::<Vec<_>>(),
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.host_task.scope.v1",
        "contract_message": "qianji.bpmn.host_task.scope.v1 requires userTask answer persistence to use the qianji result variable directly; no-tool store serviceTasks must not call an LLM.",
        "strategy": "remove_redundant_user_answer_store_service_tasks",
        "expected_xml": expected_xml,
        "target": {
            "source_id": source_id,
            "findings": findings.iter().map(UserAnswerStoreFinding::to_evidence).collect::<Vec<_>>(),
        },
        "actions": findings.iter().map(UserAnswerStoreFinding::to_action).collect::<Vec<_>>(),
    }));

    if let Some(span) = first.node_span.as_ref() {
        let extra_count = findings.len().saturating_sub(1);
        let extra_suffix = if extra_count == 0 {
            String::new()
        } else {
            format!(" and {extra_count} more store task(s)")
        };
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            source_id,
            LintSourceSpan::new(span.start, span.end),
            "store serviceTask should not invoke an LLM",
            format!(
                "Remove this store task{extra_suffix}; use '{}' directly instead of '{}'.",
                first.input_name, first.output_name
            ),
        ));
    }
    issue
}

#[derive(Clone)]
struct UserAnswerStoreFinding {
    user_task_id: String,
    service_id: String,
    next_task_id: String,
    node_span: Option<Range<usize>>,
    input_name: String,
    output_name: String,
    consumers: Vec<String>,
    prompt_text: String,
}

impl UserAnswerStoreFinding {
    fn to_evidence(&self) -> Value {
        json!({
            "user_task_id": self.user_task_id,
            "store_task_id": self.service_id,
            "next_task_id": self.next_task_id,
            "input_name": self.input_name,
            "output_name": self.output_name,
            "downstream_input_consumers": self.consumers,
            "prompt_text": self.prompt_text,
        })
    }

    fn to_action(&self) -> Value {
        json!({
            "op": "remove_redundant_store_task",
            "target": self.service_id,
            "reconnect": {
                "source_ref": self.user_task_id,
                "target_ref": self.next_task_id,
            },
            "replace_downstream_inputs": {
                "from": self.output_name,
                "to": self.input_name,
                "consumers": self.consumers,
            },
        })
    }
}

fn redundant_user_answer_store_expected_xml(findings: &[UserAnswerStoreFinding]) -> String {
    let mut lines = Vec::new();
    if findings.len() > 1 {
        lines.push(
            "<!-- Apply this pattern to every redundant user-answer store serviceTask: -->"
                .to_string(),
        );
        for finding in findings {
            lines.push(format!(
                "<!-- - remove {}; reconnect {} -> {}; replace qianji:inputs `{}` with `{}` in: {} -->",
                finding.service_id,
                finding.user_task_id,
                finding.next_task_id,
                finding.output_name,
                finding.input_name,
                finding.consumers.join(", ")
            ));
        }
    }
    let first = &findings[0];
    lines.extend([
        format!(
            "<!-- Delete serviceTask '{}' and its adjacent sequenceFlow(s). -->",
            first.service_id
        ),
        format!(
            "<sequenceFlow id=\"Flow_{}_{}\" sourceRef=\"{}\" targetRef=\"{}\"/>",
            first.user_task_id, first.next_task_id, first.user_task_id, first.next_task_id
        ),
        format!(
            "<!-- In downstream qianji:inputs, replace `{}` with `{}`. -->",
            first.output_name, first.input_name
        ),
    ]);
    lines.join("\n")
}

fn dynamic_choice_interaction<'a>(
    service: &NodeConfigContract,
    user_task: &'a NodeConfigContract,
) -> Option<&'a InteractionContract> {
    user_task.interactions.iter().find(|interaction| {
        let Some(question_ref) = interaction.question_ref.as_deref() else {
            return false;
        };
        let Some(choices_ref) = interaction.choices_ref.as_deref() else {
            return false;
        };
        service.outputs.contains(question_ref) && service.outputs.contains(choices_ref)
    })
}

fn prompt_mentions_input(prompt: &str, input: &str) -> bool {
    if input.is_empty() {
        return true;
    }
    let mut search_start = 0usize;
    while let Some(relative_start) = prompt.get(search_start..).and_then(|text| text.find(input)) {
        let start = search_start + relative_start;
        let end = start + input.len();
        let before_ok = start == 0
            || prompt
                .get(..start)
                .and_then(|text| text.chars().next_back())
                .is_none_or(|character| !is_identifier_character(character));
        let after_ok = end >= prompt.len()
            || prompt
                .get(end..)
                .and_then(|text| text.chars().next())
                .is_none_or(|character| !is_identifier_character(character));
        if before_ok && after_ok {
            return true;
        }
        search_start = end;
    }
    false
}

fn is_identifier_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn dynamic_static_interaction_producer_issue(
    source: &BpmnSourceFile,
    service: &NodeConfigContract,
    user_task: &NodeConfigContract,
) -> Option<LintIssue> {
    let interaction = user_task.interactions.iter().find(|interaction| {
        interaction.question_ref.is_some() && interaction.choices_ref.is_some()
    })?;
    let question_ref = interaction.question_ref.as_ref()?;
    let choices_ref = interaction.choices_ref.as_ref()?;
    if !service.outputs.contains(question_ref) || !service.outputs.contains(choices_ref) {
        return None;
    }
    Some(static_interaction_producer_issue(
        source,
        service,
        user_task,
        interaction,
        question_ref,
        choices_ref,
    ))
}

fn dynamic_choice_input_binding_issue(
    source: &BpmnSourceFile,
    findings: &[DynamicChoiceInputBindingFinding],
) -> LintIssue {
    let source_id = &source.source_id;
    let first = &findings[0];
    let unbound_inputs_text = first.unbound_inputs.join(",");
    let target_pairs = dynamic_choice_binding_target_pairs(findings);
    let expected_xml = dynamic_choice_input_binding_expected_xml(findings);
    let issue_count_suffix = if findings.len() == 1 {
        String::new()
    } else {
        format!(
            " This file has {} matching unbound producers.",
            findings.len()
        )
    };
    let mut issue = LintIssue::new(
        "bpmn.dynamic_qianji_interaction_producer_unbound_inputs",
        "Dynamic qianji choices producer does not bind declared inputs",
        format!(
            "Source '{source_id}' serviceTask '{}' declares qianji:inputs '{unbound_inputs_text}' but its qianji:prompt does not reference those input names before producing '{}' for userTask '{}'.{issue_count_suffix}",
            first.service_id, first.choices_ref, first.user_task_id,
        ),
        "A dynamic pi-ask choices producer is allowed only when the question or choices actually depend on runtime inputs. If the prompt does not bind the input variable names, LLM repair and execution can drift into unrelated domains while still returning structurally valid choices.",
        vec![
            format!(
                "If choices are fixed, remove serviceTask '{}' and declare qianji:question plus qianji:choice entries directly on userTask '{}'.",
                first.service_id, first.user_task_id
            ),
            format!(
                "If choices are truly dynamic, keep the producer but reference every declared input name in its qianji:prompt: {unbound_inputs_text}."
            ),
            format!("Apply the same repair pattern to: {target_pairs}."),
            format!(
                "Keep qianji:outputSchema kind=\"choice_array\" for dynamic `{}` producers.",
                first.choices_ref
            ),
        ],
        format!(
            "Repair BPMN source '{source_id}' by converting fixed UI choices to static userTask interaction XML, or by making serviceTask '{}' explicitly bind qianji:inputs '{unbound_inputs_text}' in its qianji:prompt if the choices are runtime-dependent.",
            first.service_id,
        ),
        json!({
            "source_id": source_id,
            "first_producer_task_id": first.service_id,
            "first_consumer_task_id": first.user_task_id,
            "findings": findings.iter().map(DynamicChoiceInputBindingFinding::to_evidence).collect::<Vec<_>>(),
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.user_task.interaction.v1",
        "contract_message": "qianji.bpmn.user_task.interaction.v1 requires dynamic choices producers to reference every declared qianji:inputs name in qianji:prompt; fixed choices must be static userTask interaction XML.",
        "strategy": "bind_dynamic_choices_inputs_or_inline_static_interaction",
        "expected_xml": expected_xml,
        "target": {
            "source_id": source_id,
            "findings": findings.iter().map(DynamicChoiceInputBindingFinding::to_evidence).collect::<Vec<_>>(),
        },
        "actions": [
            {
                "op": "inline_static_interaction_when_choices_are_fixed",
                "target": format!("{}.qianji:interaction", first.user_task_id),
                "xml": expected_xml.clone(),
            },
            {
                "op": "bind_runtime_inputs_when_choices_are_dynamic",
                "target": format!("{}.qianji:prompt", first.service_id),
                "required_input_names": first.unbound_inputs.clone(),
            }
        ],
    }));

    if let Some(span) = first.node_span.as_ref() {
        let extra_count = findings.len().saturating_sub(1);
        let extra_suffix = if extra_count == 0 {
            String::new()
        } else {
            format!(" and {extra_count} more matching producer(s)")
        };
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            source_id,
            LintSourceSpan::new(span.start, span.end),
            "dynamic choices producer must bind qianji:inputs by name",
            format!(
                "Reference '{unbound_inputs_text}' in this producer prompt{extra_suffix}, or inline fixed choices on userTask '{}'.",
                first.user_task_id
            ),
        ));
    }
    issue
}

#[derive(Clone)]
struct DynamicChoiceInputBindingFinding {
    service_id: String,
    user_task_id: String,
    node_span: Option<Range<usize>>,
    unbound_inputs: Vec<String>,
    question_ref: String,
    choices_ref: String,
    result_output: String,
    interaction_type: String,
    prompt_text: String,
    expected_xml: String,
}

impl DynamicChoiceInputBindingFinding {
    fn new(
        service: &NodeConfigContract,
        user_task: &NodeConfigContract,
        interaction: &InteractionContract,
        unbound_inputs: Vec<String>,
    ) -> Self {
        let question_ref = interaction
            .question_ref
            .clone()
            .unwrap_or_else(|| "currentQuestion".to_string());
        let choices_ref = interaction
            .choices_ref
            .clone()
            .unwrap_or_else(|| "currentChoices".to_string());
        let result_output = interaction
            .result_output
            .clone()
            .unwrap_or_else(|| "answer".to_string());
        let interaction_type = interaction
            .interaction_type
            .clone()
            .unwrap_or_else(|| "choice_input".to_string());
        let expected_xml = static_interaction_xml(
            service,
            user_task,
            &interaction_type,
            &result_output,
            &question_ref,
            &choices_ref,
        );
        Self {
            service_id: service.node_id.clone(),
            user_task_id: user_task.node_id.clone(),
            node_span: service.node_span.clone(),
            unbound_inputs,
            question_ref,
            choices_ref,
            result_output,
            interaction_type,
            prompt_text: service.prompt_text.clone(),
            expected_xml,
        }
    }

    fn to_evidence(&self) -> Value {
        json!({
            "producer_task_id": self.service_id,
            "consumer_task_id": self.user_task_id,
            "unbound_inputs": self.unbound_inputs,
            "question_ref": self.question_ref,
            "choices_ref": self.choices_ref,
            "result_output": self.result_output,
            "interaction_type": self.interaction_type,
            "prompt_text": self.prompt_text,
        })
    }
}

fn dynamic_choice_binding_target_pairs(findings: &[DynamicChoiceInputBindingFinding]) -> String {
    findings
        .iter()
        .map(|finding| {
            format!(
                "{} -> {} ({})",
                finding.service_id,
                finding.user_task_id,
                finding.unbound_inputs.join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn dynamic_choice_input_binding_expected_xml(
    findings: &[DynamicChoiceInputBindingFinding],
) -> String {
    let first = &findings[0];
    let mut lines = Vec::new();
    if findings.len() > 1 {
        lines.push(
            "<!-- Apply this pattern to every unbound dynamic choices producer: -->".to_string(),
        );
        for finding in findings {
            lines.push(format!(
                "<!-- - {} -> {} (inputs: {}) -->",
                finding.service_id,
                finding.user_task_id,
                finding.unbound_inputs.join(",")
            ));
        }
    }
    lines.push(first.expected_xml.clone());
    lines.join("\n")
}

fn redundant_static_interaction_producer_issue(
    source: &BpmnSourceFile,
    service: &NodeConfigContract,
    user_task: &NodeConfigContract,
) -> Option<LintIssue> {
    if !user_task
        .interactions
        .iter()
        .any(is_inline_choice_interaction)
    {
        return None;
    }
    let redundant_inputs = user_task
        .inputs_ordered
        .iter()
        .filter(|input| service.outputs.contains(*input))
        .cloned()
        .collect::<Vec<_>>();
    if redundant_inputs.is_empty() {
        return None;
    }
    Some(redundant_static_interaction_producer_lint_issue(
        source,
        service,
        user_task,
        &redundant_inputs,
    ))
}

fn is_inline_choice_interaction(interaction: &InteractionContract) -> bool {
    interaction.has_choice_contract
        && interaction.question_ref.is_none()
        && interaction.choices_ref.is_none()
}

fn static_interaction_producer_issue(
    source: &BpmnSourceFile,
    service: &NodeConfigContract,
    user_task: &NodeConfigContract,
    interaction: &InteractionContract,
    question_ref: &str,
    choices_ref: &str,
) -> LintIssue {
    let source_id = &source.source_id;
    let result_output = interaction.result_output.as_deref().unwrap_or("answer");
    let interaction_type = interaction
        .interaction_type
        .as_deref()
        .unwrap_or("choice_input");
    let static_interaction_xml = static_interaction_xml(
        service,
        user_task,
        interaction_type,
        result_output,
        question_ref,
        choices_ref,
    );
    let mut issue = LintIssue::new(
        "bpmn.static_qianji_interaction_producer",
        "Static qianji choices should be declared on the userTask",
        format!(
            "Source '{source_id}' serviceTask '{}' has no qianji inputs or tools, but outputs '{question_ref}' and '{choices_ref}' only to feed userTask '{}'.",
            service.node_id, user_task.node_id,
        ),
        "A fixed pi-ask question is userTask metadata, not LLM work. Routing static question text or fixed choices through a serviceTask makes old artifacts slower and can turn UI schema into model output.",
        vec![
            format!(
                "Inline the question and choices as direct children of userTask '{}' qianji:interaction.",
                user_task.node_id
            ),
            format!(
                "Remove '{question_ref}' and '{choices_ref}' from userTask '{}' qianji:inputs after replacing the dynamic refs.",
                user_task.node_id
            ),
            format!(
                "Remove serviceTask '{}' and reconnect incoming flow(s) to userTask '{}' when that serviceTask exists only for fixed pi-ask metadata.",
                service.node_id, user_task.node_id
            ),
            "Keep a dynamic producer only when the question or choices depend on declared runtime qianji:inputs.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by replacing userTask '{}' dynamic `<qianji:question ref=\"{question_ref}\"/>` and `<qianji:choices ref=\"{choices_ref}\"/>` with static qianji:question and qianji:choice XML. Use structured_repair.expected_xml as the shape, filling the fixed question and choices from the producer prompt. Remove the no-input/no-tool producer serviceTask '{}' and reconnect sequence flows if it only prepared fixed UI text.",
            user_task.node_id, service.node_id,
        ),
        json!({
            "source_id": source_id,
            "producer_task_id": service.node_id,
            "consumer_task_id": user_task.node_id,
            "question_ref": question_ref,
            "choices_ref": choices_ref,
            "expected_interaction_xml": static_interaction_xml,
        }),
    )
    .with_structured_repair(static_interaction_producer_repair(
        source_id,
        service,
        user_task,
        question_ref,
        choices_ref,
        &static_interaction_xml,
    ));

    if let Some(span) = service.node_span.as_ref() {
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            source_id,
            LintSourceSpan::new(span.start, span.end),
            "static UI producer should not invoke an LLM",
            format!(
                "Remove this producer, clear stale qianji:inputs, and keep fixed qianji choices on userTask '{}'.",
                user_task.node_id,
            ),
        ));
    }
    issue
}

fn redundant_static_interaction_producer_lint_issue(
    source: &BpmnSourceFile,
    service: &NodeConfigContract,
    user_task: &NodeConfigContract,
    redundant_inputs: &[String],
) -> LintIssue {
    let source_id = &source.source_id;
    let redundant_inputs_text = redundant_inputs.join(",");
    let expected_xml = redundant_static_interaction_xml(service, user_task, redundant_inputs);
    let mut issue = LintIssue::new(
        "bpmn.redundant_static_qianji_interaction_producer",
        "Static qianji interaction still has a UI producer serviceTask",
        format!(
            "Source '{source_id}' serviceTask '{}' has no qianji inputs or tools, and its outputs '{redundant_inputs_text}' only remain as inputs on static userTask '{}'.",
            service.node_id, user_task.node_id,
        ),
        "Once fixed questions and choices are declared directly on the userTask, the old producer serviceTask no longer owns executable work and only adds an unnecessary LLM boundary.",
        vec![
            format!(
                "Remove serviceTask '{}' when it only prepared fixed user interaction metadata.",
                service.node_id
            ),
            format!(
                "Reconnect incoming flow(s) for '{}' directly to userTask '{}'.",
                service.node_id, user_task.node_id
            ),
            format!(
                "Remove '{redundant_inputs_text}' from userTask '{}' qianji:inputs.",
                user_task.node_id
            ),
        ],
        format!(
            "Repair BPMN source '{source_id}' by deleting redundant static interaction producer '{}' and removing stale qianji:inputs '{redundant_inputs_text}' from userTask '{}'. Use structured_repair.expected_xml as the target shape.",
            service.node_id, user_task.node_id,
        ),
        json!({
            "source_id": source_id,
            "producer_task_id": service.node_id,
            "consumer_task_id": user_task.node_id,
            "redundant_inputs": redundant_inputs,
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.user_task.interaction.v1",
        "contract_message": "qianji.bpmn.user_task.interaction.v1 requires fixed interaction metadata to live on the userTask without a no-input/no-tool producer serviceTask.",
        "strategy": "remove_redundant_static_interaction_producer",
        "expected_xml": expected_xml,
        "target": {
            "source_id": source_id,
            "producer_task_id": service.node_id,
            "consumer_task_id": user_task.node_id,
            "redundant_inputs": redundant_inputs,
        },
        "actions": [
            {
                "op": "remove_service_task_if_ui_only",
                "target": service.node_id,
                "also": format!("reconnect incoming sequenceFlow(s) to {}", user_task.node_id)
            },
            {
                "op": "remove_from_inputs",
                "target": format!("{}.qianji:inputs", user_task.node_id),
                "forbidden_forms": redundant_inputs
            }
        ],
    }));

    if let Some(span) = service.node_span.as_ref() {
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            source_id,
            LintSourceSpan::new(span.start, span.end),
            "redundant static UI producer should be removed",
            format!(
                "Remove this no-input/no-tool producer and keep fixed qianji choices on userTask '{}'.",
                user_task.node_id
            ),
        ));
    }
    issue
}

fn static_interaction_xml(
    service: &NodeConfigContract,
    user_task: &NodeConfigContract,
    interaction_type: &str,
    result_output: &str,
    question_ref: &str,
    choices_ref: &str,
) -> String {
    [
        format!(
            "<!-- Remove serviceTask '{}' and reconnect its incoming sequenceFlow(s) directly to userTask '{}'. -->",
            service.node_id, user_task.node_id
        ),
        format!("<userTask id=\"{}\">", user_task.node_id),
        "  <extensionElements>".to_string(),
        "    <qianji:config>".to_string(),
        format!(
            "      <!-- Remove stale qianji:inputs value(s): {question_ref},{choices_ref}. -->"
        ),
        "      <qianji:inputs></qianji:inputs>".to_string(),
        format!("      <qianji:interaction type=\"{interaction_type}\">"),
        "        <qianji:question>Write the user-facing question text here.</qianji:question>"
            .to_string(),
        "        <qianji:choice value=\"option_value\" label=\"Option label\">Optional concise help text.</qianji:choice>"
            .to_string(),
        "        <qianji:choice value=\"other_value\" label=\"Other option\">Optional concise help text.</qianji:choice>"
            .to_string(),
        format!("        <qianji:result output=\"{result_output}\"/>"),
        "      </qianji:interaction>".to_string(),
        "    </qianji:config>".to_string(),
        "  </extensionElements>".to_string(),
        "</userTask>".to_string(),
    ]
    .join("\n")
}

fn redundant_static_interaction_xml(
    service: &NodeConfigContract,
    user_task: &NodeConfigContract,
    redundant_inputs: &[String],
) -> String {
    let redundant_inputs_text = redundant_inputs.join(",");
    [
        format!("<!-- Remove serviceTask '{}' and reconnect its incoming sequenceFlow(s) directly to userTask '{}'. -->", service.node_id, user_task.node_id),
        format!("<userTask id=\"{}\">", user_task.node_id),
        "  <extensionElements>".to_string(),
        "    <qianji:config>".to_string(),
        format!("      <!-- Remove stale qianji:inputs value(s): {redundant_inputs_text}. -->"),
        "      <qianji:inputs></qianji:inputs>".to_string(),
        "      <!-- Keep the existing static qianji:interaction with qianji:question, qianji:choice, and qianji:result. -->".to_string(),
        "    </qianji:config>".to_string(),
        "  </extensionElements>".to_string(),
        "</userTask>".to_string(),
    ]
    .join("\n")
}

fn static_interaction_producer_repair(
    source_id: &str,
    service: &NodeConfigContract,
    user_task: &NodeConfigContract,
    question_ref: &str,
    choices_ref: &str,
    static_interaction_xml: &str,
) -> Value {
    json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.user_task.interaction.v1",
        "contract_message": "qianji.bpmn.user_task.interaction.v1 requires fixed questions and choices to be declared as static userTask interaction XML.",
        "strategy": "inline_static_interaction_on_user_task",
        "target": {
            "source_id": source_id,
            "producer_task_id": service.node_id,
            "consumer_task_id": user_task.node_id,
            "question_ref": question_ref,
            "choices_ref": choices_ref,
        },
        "expected_xml": static_interaction_xml,
        "actions": [
            {
                "op": "replace_dynamic_interaction_refs",
                "target": format!("{}.qianji:interaction", user_task.node_id),
                "xml": static_interaction_xml,
                "when": "question and choices are known at compile time"
            },
            {
                "op": "remove_from_inputs",
                "target": format!("{}.qianji:inputs", user_task.node_id),
                "forbidden_forms": [
                    question_ref,
                    choices_ref
                ]
            },
            {
                "op": "remove_service_task_if_ui_only",
                "target": service.node_id,
                "also": format!("reconnect incoming sequenceFlow(s) to {}", user_task.node_id)
            },
            {
                "op": "keep_dynamic_producer_only_when_runtime_dependent",
                "target": service.node_id,
                "requires": "non-empty qianji:inputs and qianji:outputSchema kind=\"choice_array\" for dynamic choices"
            }
        ],
    })
}

fn dynamic_choice_output_schema_issues(
    source: &BpmnSourceFile,
    dynamic_choice_refs: &[DynamicChoiceRefContract],
    producers_by_output: &HashMap<String, Vec<OutputProducerContract>>,
) -> Vec<LintIssue> {
    let mut seen_refs = HashSet::new();
    let mut issues = Vec::new();
    for choice_ref in dynamic_choice_refs {
        if !seen_refs.insert(choice_ref.choices_ref.clone()) {
            continue;
        }
        let producers = producers_by_output
            .get(&choice_ref.choices_ref)
            .cloned()
            .unwrap_or_default();
        let missing_schema_producers = producers
            .iter()
            .filter(|producer| producer.output_schema_kind.as_deref() != Some("choice_array"))
            .cloned()
            .collect::<Vec<_>>();
        if missing_schema_producers.is_empty() {
            continue;
        }
        issues.push(missing_dynamic_choice_output_schema_issue(
            source,
            choice_ref,
            &missing_schema_producers,
        ));
    }
    issues
}

fn missing_dynamic_choice_output_schema_issue(
    source: &BpmnSourceFile,
    choice_ref: &DynamicChoiceRefContract,
    producers: &[OutputProducerContract],
) -> LintIssue {
    let source_id = &source.source_id;
    let producer = select_dynamic_choice_schema_producer(producers);
    let producer_ids = producer_task_ids(producers);
    let schema_xml = dynamic_choice_output_schema_xml(&choice_ref.choices_ref);
    let mut issue = LintIssue::new(
        "bpmn.missing_qianji_dynamic_choices_output_schema",
        "Dynamic qianji choices producer is missing choice_array schema",
        dynamic_choice_schema_description(source_id, choice_ref, &producer_ids),
        "Dynamic qianji choices must be structured choice objects. Without an explicit producer output schema, LLM-generated BPMN can regress to string arrays or prompt-prose options that hosts cannot render safely.",
        dynamic_choice_schema_guidance(choice_ref, &schema_xml),
        format!(
            "Repair BPMN source '{source_id}' by inserting `{schema_xml}` immediately after the producer qianji:outputs line for `{}`. Do not add runtime compatibility; make the BPMN contract explicit.",
            choice_ref.choices_ref
        ),
        dynamic_choice_schema_evidence(source_id, choice_ref, &producer_ids, &schema_xml),
    )
    .with_structured_repair(dynamic_choice_schema_repair(
        source_id,
        choice_ref,
        producers,
        &producer_ids,
        &schema_xml,
    ));
    if let Some(span) = producer.outputs_span.as_ref() {
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            source_id,
            LintSourceSpan::new(span.start, span.end),
            "producer must declare structured choices schema",
            format!("Add `{schema_xml}` after this qianji:outputs line."),
        ));
    }
    issue
}

fn select_dynamic_choice_schema_producer(
    producers: &[OutputProducerContract],
) -> &OutputProducerContract {
    producers
        .iter()
        .find(|producer| producer.outputs_span.is_some())
        .unwrap_or(&producers[0])
}

fn producer_task_ids(producers: &[OutputProducerContract]) -> Vec<String> {
    producers
        .iter()
        .filter_map(|producer| producer.node_id.as_deref())
        .map(ToString::to_string)
        .collect()
}

fn dynamic_choice_schema_description(
    source_id: &str,
    choice_ref: &DynamicChoiceRefContract,
    producer_ids: &[String],
) -> String {
    let producer_list = if producer_ids.is_empty() {
        "unknown producer task".to_string()
    } else {
        producer_ids.join(", ")
    };
    format!(
        "Source '{source_id}' uses qianji:choices ref '{}' from userTask '{}', but producer task(s) {producer_list} do not declare qianji:outputSchema kind=\"choice_array\" for that output.",
        choice_ref.choices_ref,
        choice_ref
            .node_id
            .as_deref()
            .unwrap_or("(unknown userTask)"),
    )
}

fn dynamic_choice_schema_guidance(
    choice_ref: &DynamicChoiceRefContract,
    schema_xml: &str,
) -> Vec<String> {
    vec![
        format!(
            "Add `{schema_xml}` inside the qianji:config of the task that outputs `{}`.",
            choice_ref.choices_ref
        ),
        format!(
            "Keep `<qianji:choices ref=\"{}\"/>` in the consuming userTask.",
            choice_ref.choices_ref
        ),
        "Return choices as JSON objects with required non-empty `value` and optional `label` and `description` fields.".to_string(),
    ]
}

fn dynamic_choice_schema_evidence(
    source_id: &str,
    choice_ref: &DynamicChoiceRefContract,
    producer_ids: &[String],
    schema_xml: &str,
) -> Value {
    json!({
        "source_id": source_id,
        "choices_ref": choice_ref.choices_ref,
        "consumer_task_id": choice_ref.node_id.as_deref(),
        "producer_task_ids": producer_ids,
        "required_xml": schema_xml,
        "expected_value_shape": {
            "type": "array",
            "items": {
                "type": "object",
                "required": ["value"],
                "properties": {
                    "value": "non-empty string returned as the userTask reply",
                    "label": "optional display text",
                    "description": "optional help text"
                }
            }
        }
    })
}

fn dynamic_choice_schema_repair(
    source_id: &str,
    choice_ref: &DynamicChoiceRefContract,
    producers: &[OutputProducerContract],
    producer_ids: &[String],
    schema_xml: &str,
) -> Value {
    let line_fixes = producers
        .iter()
        .map(|producer| {
            json!({
                "target": format!("{}.qianji:outputs", producer.node_id.as_deref().unwrap_or("(unknown producer)")),
                "offset": producer.outputs_span.as_ref().map(|span| span.start),
                "xml": [
                    dynamic_choice_output_line(producer),
                    schema_xml,
                ],
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.user_task.interaction.v1",
        "contract_message": "qianji.bpmn.user_task.interaction.v1 requires dynamic choices producers to declare qianji:outputSchema kind=\"choice_array\".",
        "strategy": "declare_dynamic_choices_output_schema",
        "target": {
            "source_id": source_id,
            "choices_ref": choice_ref.choices_ref,
            "producer_task_ids": producer_ids,
        },
        "line_fixes": line_fixes,
        "actions": [
            {
                "op": "insert_after",
                "target": "producer qianji:outputs",
                "xml": schema_xml,
            }
        ],
    })
}

fn dynamic_choice_output_line(producer: &OutputProducerContract) -> String {
    format!(
        "<qianji:outputs>{}</qianji:outputs>",
        producer
            .outputs_text
            .as_deref()
            .unwrap_or(producer.output_name.as_str())
    )
}

fn dynamic_choice_output_schema_xml(choices_ref: &str) -> String {
    format!(
        "<qianji:outputSchema name=\"{choices_ref}\" kind=\"choice_array\" value=\"required\" label=\"optional\" description=\"optional\"/>"
    )
}

fn parse_output_names(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn reader_position(reader: &Reader<&[u8]>) -> Option<usize> {
    usize::try_from(reader.buffer_position()).ok()
}

fn start_event_span(event_end: usize, event: &BytesStart<'_>) -> Option<std::ops::Range<usize>> {
    let raw: &[u8] = event.as_ref();
    let start = event_end.checked_sub(raw.len() + 2)?;
    Some(start..event_end)
}

fn has_non_empty_attribute(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> bool {
    attribute_value(reader, event, attribute_name).is_some_and(|value| !value.trim().is_empty())
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

fn unsupported_interaction_type_issue(
    source: &BpmnSourceFile,
    interaction_type: &str,
) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::new(
        "bpmn.unsupported_qianji_interaction_type",
        "Qianji user interaction type is unsupported",
        format!(
            "Source '{source_id}' uses qianji interaction type '{interaction_type}', which is outside the active qianji extension contract."
        ),
        "Qianji-owned user-task interaction rendering currently supports only a bounded native UI subset: input, confirm, choice, and choice_input.",
        vec![
            "Replace unsupported interaction types such as `free_form` with `input` for plain text input.".to_string(),
            "Use `choice_input` when the prompt needs option selection plus optional free-form feedback.".to_string(),
            "Keep the answer mapping in declared `qianji:outputs` so downstream gateways only consume declared variables.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by changing qianji interaction type '{interaction_type}' to one supported value: input, confirm, choice, or choice_input. Preserve the user prompt, choices, freeText fields, and declared qianji outputs."
        ),
        json!({
            "source_id": source_id,
            "interaction_type": interaction_type,
            "supported_interaction_types": SUPPORTED_INTERACTION_TYPES,
        }),
    )
    .with_structured_repair(interaction_repair_plan(
        source_id,
        "replace_unsupported_interaction_type",
        json!({
            "op": "set_attribute",
            "element": "qianji:interaction",
            "attribute": "type",
            "allowed_values": SUPPORTED_INTERACTION_TYPES,
            "selection_hint": {
                "input": "plain free-form answer",
                "confirm": "yes/no approval",
                "choice": "bounded option selection",
                "choice_input": "option selection plus optional free-form feedback"
            },
            "replace": interaction_type
        }),
    ))
}

fn missing_interaction_type_issue(source: &BpmnSourceFile) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::new(
        "bpmn.missing_qianji_interaction_type",
        "Qianji user interaction type is missing",
        format!("Source '{source_id}' has qianji:interaction without a `type` attribute."),
        "Qianji-owned user-task interaction rendering needs an explicit bounded native UI type.",
        vec![
            "Add `type=\"input\"` for plain free-form text input.".to_string(),
            "Add `type=\"confirm\"`, `type=\"choice\"`, or `type=\"choice_input\"` for bounded approval and selection checkpoints.".to_string(),
            "Keep the selected type aligned with the declared `qianji:outputs` mapping.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by adding one supported qianji interaction type: input, confirm, choice, or choice_input."
        ),
        json!({
            "source_id": source_id,
            "supported_interaction_types": SUPPORTED_INTERACTION_TYPES,
        }),
    )
    .with_structured_repair(interaction_repair_plan(
        source_id,
        "add_missing_interaction_type",
        json!({
            "op": "set_attribute",
            "element": "qianji:interaction",
            "attribute": "type",
            "allowed_values": SUPPORTED_INTERACTION_TYPES,
            "default_when_unsure": "choice_input"
        }),
    ))
}

fn missing_choice_contract_issue(source: &BpmnSourceFile, interaction_type: &str) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::new(
        "bpmn.missing_qianji_interaction_choices",
        "Qianji choice interaction is missing choices",
        format!(
            "Source '{source_id}' has qianji interaction type '{interaction_type}' without static qianji:choice entries or a dynamic qianji:choices ref."
        ),
        "Choice interactions need a structured option contract so hosts can render choices without parsing prompt prose.",
        vec![
            "For static options, add one or more `qianji:choice` elements with non-empty `value` attributes.".to_string(),
            "For dynamic options, add `<qianji:choices ref=\"currentChoices\"/>` and have an upstream task output `currentChoices` as structured choice objects.".to_string(),
            "Keep generated question text in `currentQuestion`; do not embed numbered choices inside the question string.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by adding either static `qianji:choice value=\"...\"` entries or a dynamic `<qianji:choices ref=\"currentChoices\"/>` child to the qianji:interaction."
        ),
        json!({
            "source_id": source_id,
            "interaction_type": interaction_type,
            "valid_choice_contracts": ["qianji:choice[value]", "qianji:choices[ref]"],
        }),
    )
    .with_structured_repair(interaction_repair_plan(
        source_id,
        "add_missing_choice_contract",
        json!({
            "op": "add_child",
            "parent": "qianji:interaction",
            "allowed_children": [
                {
                    "element": "qianji:choice",
                    "required_attributes": ["value"],
                    "use_when": "choices are static at compile time"
                },
                {
                    "element": "qianji:choices",
                    "required_attributes": ["ref"],
                    "example": "<qianji:choices ref=\"currentChoices\"/>",
                    "use_when": "choices are generated by an upstream serviceTask"
                }
            ]
        }),
    ))
}

fn undeclared_interaction_result_issue(
    source: &BpmnSourceFile,
    contract: &InteractionContract,
    active_config: &ActiveConfig,
) -> Option<LintIssue> {
    let result_output = contract.result_output.as_ref()?;
    if active_config.outputs.contains(result_output) {
        return None;
    }
    let declared_outputs = active_config.outputs.iter().cloned().collect::<Vec<_>>();
    let declared_list = if declared_outputs.is_empty() {
        "none".to_string()
    } else {
        declared_outputs.join(", ")
    };
    let source_id = &source.source_id;
    let mut issue = LintIssue::new(
        "bpmn.undeclared_qianji_interaction_result",
        "Qianji interaction result output is not declared",
        format!(
            "Source '{source_id}' maps qianji:result to '{result_output}', but that name is missing from the same qianji:outputs declaration."
        ),
        "Qianji only persists declared task outputs. If qianji:result writes to an undeclared name, downstream tasks and gateways may not receive the user's answer.",
        vec![
            format!(
                "Add '{result_output}' to the same `qianji:outputs` list when downstream nodes need that answer."
            ),
            format!(
                "Or change `qianji:result output` to one declared output. Currently declared outputs: {declared_list}."
            ),
            "Keep qianji:result, qianji:outputs, downstream qianji:inputs, and gateway conditions aligned on the same variable name.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by aligning `<qianji:result output=\"{result_output}\"/>` with the same task's `<qianji:outputs>`. Either add `{result_output}` to qianji:outputs, or change qianji:result to an already declared output. Preserve downstream input and gateway variable names."
        ),
        json!({
            "source_id": source_id,
            "result_output": result_output,
            "declared_outputs": declared_outputs.clone(),
        }),
    )
    .with_structured_repair(interaction_repair_plan(
        source_id,
        "declare_interaction_result_output",
        json!({
            "op": "align_qianji_result_with_outputs",
            "result_output": result_output,
            "declared_outputs": declared_outputs.clone(),
            "preferred_fix": format!("add `{result_output}` to qianji:outputs when downstream nodes consume it"),
            "forbid": "qianji:result output names missing from qianji:outputs"
        }),
    ));
    if let Some(span) = contract.result_span.clone() {
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            source_id,
            LintSourceSpan::new(span.start, span.end),
            "declare this qianji:result output",
            format!(
                "`{result_output}` must appear in the same qianji:outputs list, or qianji:result must target a declared output."
            ),
        ));
    }
    Some(issue)
}

fn ambiguous_interaction_outputs_issue(
    source: &BpmnSourceFile,
    contract: &InteractionContract,
    active_config: &ActiveConfig,
) -> Option<LintIssue> {
    if active_config.node_kind.as_deref() != Some("userTask") {
        return None;
    }
    let result_output = contract.result_output.as_ref()?;
    if active_config.outputs_ordered.len() == 1
        && active_config.outputs_ordered.first() == Some(result_output)
    {
        return None;
    }

    let source_id = &source.source_id;
    let node_id = active_config
        .node_id
        .as_deref()
        .unwrap_or("(unknown userTask)");
    let declared_outputs = active_config.outputs_ordered.clone();
    let declared_list = if declared_outputs.is_empty() {
        "none".to_string()
    } else {
        declared_outputs.join(", ")
    };
    let output_xml = qianji_outputs_xml(result_output);
    let derived_outputs = declared_outputs
        .iter()
        .filter(|output| *output != result_output)
        .cloned()
        .collect::<Vec<_>>();
    let mut issue = LintIssue::new(
        "bpmn.ambiguous_qianji_interaction_outputs",
        "Qianji userTask interaction must declare exactly one result output",
        format!(
            "Source '{source_id}' userTask '{node_id}' maps qianji:result to '{result_output}', but qianji:outputs declares [{declared_list}]."
        ),
        "A single human reply cannot safely populate multiple qianji:outputs. Derived variables must be produced by a following serviceTask that consumes the user answer.",
        vec![
            format!("Set this userTask qianji:outputs to exactly `{result_output}`."),
            format!(
                "If downstream nodes need derived variables [{}], add a following serviceTask that consumes `{result_output}` and emits those variables.",
                if derived_outputs.is_empty() {
                    "none".to_string()
                } else {
                    derived_outputs.join(", ")
                }
            ),
            "Keep qianji:result, qianji:outputs, downstream qianji:inputs, and gateway conditions aligned on the same variable name.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' userTask '{node_id}' by replacing its qianji:outputs line with `{output_xml}`. Move any removed derived outputs into a downstream serviceTask that consumes `{result_output}`. Do not map one human reply to multiple userTask outputs."
        ),
        json!({
            "source_id": source_id,
            "user_task_id": node_id,
            "result_output": result_output,
            "declared_outputs": declared_outputs,
            "derived_outputs_to_move": derived_outputs,
            "required_xml": output_xml,
        }),
    )
    .with_structured_repair(interaction_outputs_repair_plan(
        source_id,
        node_id,
        result_output,
        &output_xml,
        active_config,
        &derived_outputs,
    ));

    if let Some(span) = active_config.outputs_span.clone() {
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            source_id,
            LintSourceSpan::new(span.start, span.end),
            "userTask must declare only the interaction result output",
            format!(
                "Replace this line with `{output_xml}`; derive other variables in a later serviceTask."
            ),
        ));
    }

    Some(issue)
}

fn qianji_outputs_xml(output_name: &str) -> String {
    format!("<qianji:outputs>{output_name}</qianji:outputs>")
}

fn interaction_outputs_repair_plan(
    source_id: &str,
    node_id: &str,
    result_output: &str,
    output_xml: &str,
    active_config: &ActiveConfig,
    derived_outputs: &[String],
) -> Value {
    let mut line_fixes = Vec::new();
    if let Some(span) = active_config.outputs_span.as_ref() {
        line_fixes.push(json!({
            "target": format!("{node_id}.qianji:outputs"),
            "offset": span.start,
            "xml": output_xml,
        }));
    }
    json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.user_task.interaction.v1",
        "contract_message": "qianji.bpmn.user_task.interaction.v1 requires each userTask interaction to declare exactly the qianji:result output in qianji:outputs.",
        "strategy": "align_user_task_interaction_outputs",
        "target": {
            "source_id": source_id,
            "user_task_id": node_id,
            "result_output": result_output,
            "derived_outputs_to_move": derived_outputs,
        },
        "line_fixes": line_fixes,
        "actions": [
            {
                "op": "replace_line",
                "target": "userTask qianji:outputs",
                "xml": output_xml,
            },
            {
                "op": "move_derived_outputs",
                "target": "following serviceTask",
                "requires": format!("serviceTask consumes `{result_output}` before emitting derived variables"),
                "forbid": "mapping one human reply to multiple userTask outputs",
            }
        ],
    })
}

fn ambiguous_question_choices_ref_issue(
    source: &BpmnSourceFile,
    contract: &InteractionContract,
) -> Option<LintIssue> {
    let question_ref = contract.question_ref.as_deref()?;
    let choices_ref = contract.choices_ref.as_deref()?;
    if question_ref != choices_ref {
        return None;
    }
    let source_id = &source.source_id;
    let mut issue = LintIssue::new(
            "bpmn.ambiguous_qianji_interaction_choices_ref",
            "Qianji interaction reuses one variable as both question text and choices",
            format!(
                "Source '{source_id}' uses qianji:question ref and qianji:choices ref with the same variable '{question_ref}'."
            ),
            "A dynamic choices ref must resolve to a structured option array, while a question ref resolves to display text. Reusing one variable for both makes host rendering ambiguous.",
            vec![
                "Split generated prompts into separate variables such as `currentQuestion` and `currentChoices`.".to_string(),
                "Have the upstream serviceTask output the choices variable as an array of `{value,label,description}` objects.".to_string(),
                "For approval gates over a text artifact, keep `qianji:question ref` on the artifact text and use static `qianji:choice` elements instead of `qianji:choices ref`.".to_string(),
            ],
            format!(
                "Repair BPMN source '{source_id}' by changing the qianji interaction so `{question_ref}` is not used as both question text and dynamic choices. Either introduce a separate choices variable such as `currentChoices`, or replace `qianji:choices ref` with static `qianji:choice` entries."
            ),
            json!({
                "source_id": source_id,
                "question_ref": question_ref,
                "choices_ref": choices_ref,
                "valid_dynamic_shape": {
                    "question_ref": "string display text",
                    "choices_ref": "array of objects with value, label, and optional description"
                },
            }),
        );
    if let Some(span) = contract
        .choices_span
        .as_ref()
        .or(contract.question_span.as_ref())
    {
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            source_id,
            LintSourceSpan::new(span.start, span.end),
            format!("split choices ref `{question_ref}` from the question text ref"),
            format!(
                "Keep `{question_ref}` as display text, then either add static qianji:choice entries or introduce a separate choices array variable such as `currentChoices`."
            ),
        ));
    }
    Some(issue.with_structured_repair(interaction_repair_plan(
            source_id,
            "split_question_text_from_dynamic_choices",
            json!({
                "op": "choose_one",
                "invalid_ref": question_ref,
                "options": [
                    {
                        "op": "introduce_dynamic_choices_variable",
                        "question_ref": question_ref,
                        "choices_ref_example": "currentChoices",
                        "producer_requirement": "upstream serviceTask outputs choices as array of {value,label,description} objects",
                        "example": "<qianji:question ref=\"currentQuestion\"/><qianji:choices ref=\"currentChoices\"/>"
                    },
                    {
                        "op": "replace_dynamic_choices_with_static_choices",
                        "use_when": "the question text is a design section, spec path, or approval artifact rather than a choice array",
                        "example": "<qianji:question ref=\"designSection\"/><qianji:choice value=\"approved\" label=\"Approve\"/><qianji:choice value=\"revise\" label=\"Revise\"/>"
                    }
                ],
                "forbidden_forms": [
                    format!("<qianji:question ref=\"{question_ref}\"/><qianji:choices ref=\"{question_ref}\"/>"),
                    "using prose text, file paths, or design sections as dynamic choices arrays"
                ]
            }),
        )))
}

fn interaction_repair_plan(
    source_id: &str,
    strategy: &'static str,
    action: serde_json::Value,
) -> serde_json::Value {
    let actions = serde_json::Value::Array(vec![action]);
    json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.user_task.interaction.v1",
        "strategy": strategy,
        "target": {
            "source_id": source_id,
        },
        "construct_cards": ["user-task.interaction"],
        "actions": actions,
    })
}
