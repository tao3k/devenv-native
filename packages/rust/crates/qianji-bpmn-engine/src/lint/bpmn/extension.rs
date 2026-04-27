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
            Ok(Event::Text(event)) if state.in_outputs => {
                if let Ok(text) = event.decode() {
                    state.capture_outputs_text(&text);
                }
            }
            Ok(Event::End(event)) => state.handle_end(&event),
            Ok(Event::Eof) | Err(_) => return state.finish(source, issues),
            Ok(_) => {}
        }
    }
}

#[derive(Default)]
struct ExtensionScanState {
    active_config: ActiveConfig,
    active_node_ids: Vec<Option<String>>,
    active_node_kinds: Vec<String>,
    dynamic_choice_refs: Vec<DynamicChoiceRefContract>,
    producers_by_output: HashMap<String, Vec<OutputProducerContract>>,
    in_outputs: bool,
}

impl ExtensionScanState {
    fn handle_start(
        &mut self,
        source: &BpmnSourceFile,
        reader: &mut Reader<&[u8]>,
        event: &BytesStart<'_>,
        issues: &mut Vec<LintIssue>,
    ) {
        if is_bpmn_node_with_qianji_config(event) {
            self.active_node_ids
                .push(attribute_value(reader, event, "id"));
            self.active_node_kinds
                .push(local_name(event.name().as_ref()).to_string());
        } else if is_qianji_config(event) {
            self.active_config = ActiveConfig {
                node_id: self.active_node_ids.last().cloned().flatten(),
                node_kind: self.active_node_kinds.last().cloned(),
                ..ActiveConfig::default()
            };
        } else if is_qianji_outputs(event) {
            self.in_outputs = true;
            self.active_config.outputs_span =
                reader_position(reader).and_then(|event_end| start_event_span(event_end, event));
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
        if is_qianji_output_schema(event) {
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
                let contract = read_interaction_contract(reader);
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

    fn capture_outputs_text(&mut self, text: &str) {
        self.active_config.outputs_text = Some(text.trim().to_string());
        self.active_config.outputs_ordered = parse_output_names(text);
        self.active_config
            .outputs
            .extend(self.active_config.outputs_ordered.iter().cloned());
    }

    fn handle_end(&mut self, event: &quick_xml::events::BytesEnd<'_>) {
        if is_end_event_name(event, "qianji:outputs") {
            self.in_outputs = false;
        } else if is_end_event_name(event, "qianji:config") {
            collect_output_producer_contracts(&self.active_config, &mut self.producers_by_output);
            self.active_config = ActiveConfig::default();
            self.in_outputs = false;
        } else if is_bpmn_node_end(event) {
            self.active_node_ids.pop();
            self.active_node_kinds.pop();
        }
    }

    fn finish(self, source: &BpmnSourceFile, mut issues: Vec<LintIssue>) -> Vec<LintIssue> {
        issues.extend(dynamic_choice_output_schema_issues(
            source,
            &self.dynamic_choice_refs,
            &self.producers_by_output,
        ));
        issues
    }
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
    node_kind: Option<String>,
    outputs: HashSet<String>,
    outputs_ordered: Vec<String>,
    outputs_text: Option<String>,
    outputs_span: Option<Range<usize>>,
    output_schema_kinds: HashMap<String, String>,
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

fn is_qianji_interaction(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:interaction")
}

fn is_qianji_config(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:config")
}

fn is_qianji_outputs(event: &BytesStart<'_>) -> bool {
    is_event_name(event, "qianji:outputs")
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

#[derive(Default)]
struct InteractionContract {
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
