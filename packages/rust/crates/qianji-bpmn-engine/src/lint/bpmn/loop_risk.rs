use crate::bpmn_parse_api::BpmnSourceFile;
use crate::ir_node_api::{BpmnGatewayKind, BpmnNodeKind};
use crate::ir_package_api::BpmnPackage;
use crate::ir_process_spec::BpmnProcessSpec;
use crate::lint_api::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use crate::repeat_condition::{GatewayConditionSummary, parse_gateway_condition_summary};
use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use serde_json::{Value, json};
use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Range;

pub(super) fn loop_risk_issues(source: &BpmnSourceFile, package: &BpmnPackage) -> Vec<LintIssue> {
    let metadata_by_process = collect_process_metadata(source);
    package
        .processes
        .iter()
        .flat_map(|process| {
            let metadata = metadata_by_process
                .get(process.key.process_id.as_ref())
                .cloned()
                .unwrap_or_default();
            process_loop_risk_issues(source, process, &metadata)
        })
        .collect()
}

#[derive(Clone, Default)]
struct ProcessMetadata {
    task_inputs: HashMap<String, BTreeSet<String>>,
    task_outputs: HashMap<String, BTreeSet<String>>,
    task_input_spans: HashMap<String, Range<usize>>,
    task_output_spans: HashMap<String, Range<usize>>,
    gateway_default_flows: HashMap<String, String>,
    sequence_flows: HashMap<String, SequenceFlowMetadata>,
    node_spans: HashMap<String, Range<usize>>,
}

#[derive(Clone)]
struct SequenceFlowMetadata {
    target_ref: String,
    span: Range<usize>,
}

#[derive(Clone, serde::Serialize)]
struct DefaultReentryFlow {
    gateway_id: String,
    flow_id: String,
    target_id: String,
    suggested_exit_target_id: Option<String>,
}

#[derive(Default)]
struct ActiveTask {
    id: String,
    inputs: BTreeSet<String>,
    outputs: BTreeSet<String>,
    in_inputs: bool,
    in_outputs: bool,
}

fn process_loop_risk_issues(
    source: &BpmnSourceFile,
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
) -> Vec<LintIssue> {
    strongly_connected_components(process)
        .into_iter()
        .filter(|component| is_cyclic_component(process, component))
        .filter_map(|component| loop_risk_issue(source, process, metadata, &component))
        .collect()
}

fn loop_risk_issue(
    source: &BpmnSourceFile,
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
) -> Option<LintIssue> {
    let component_set = component.iter().copied().collect::<HashSet<_>>();
    let task_node_ids = task_node_ids(process, component);
    if task_node_ids.is_empty() {
        return None;
    }

    let gateway_ids = gateway_node_ids(process, component);
    let route_variables = route_variables(process, component);
    let updated_variables = updated_variables(metadata, &task_node_ids);
    let missing_progress_outputs = undeclared_variables(
        &updated_variables,
        route_variables.iter().map(String::as_str),
    );
    let user_outputs = user_task_outputs(process, metadata, component);
    let worker_inputs = worker_task_inputs(process, metadata, component);
    let worker_outputs = worker_task_outputs(process, metadata, component);
    let missing_feedback_inputs = if worker_outputs.iter().any(|output| is_prompt_output(output)) {
        undeclared_variables(&worker_inputs, user_outputs.iter().map(String::as_str))
    } else {
        BTreeSet::new()
    };
    let default_reentry_flows =
        default_reentry_flows(process, metadata, &component_set, &gateway_ids);
    let has_exit_path = component_has_exit_path(process, &component_set);
    let has_conditionless_gateway_cycle = !gateway_ids.is_empty() && route_variables.is_empty();

    if has_exit_path
        && !has_conditionless_gateway_cycle
        && default_reentry_flows.is_empty()
        && missing_progress_outputs.is_empty()
        && missing_feedback_inputs.is_empty()
    {
        return None;
    }

    let evidence = LoopRiskEvidence {
        task_node_ids,
        gateway_ids,
        route_variables,
        updated_variables,
        user_outputs,
        worker_inputs,
        missing_progress_outputs,
        missing_feedback_inputs,
        default_reentry_flows,
        has_exit_path,
        has_conditionless_gateway_cycle,
    };
    Some(unbounded_control_cycle_issue(
        source, process, metadata, component, &evidence,
    ))
}

struct LoopRiskEvidence {
    task_node_ids: Vec<String>,
    gateway_ids: Vec<String>,
    route_variables: BTreeSet<String>,
    updated_variables: BTreeSet<String>,
    user_outputs: BTreeSet<String>,
    worker_inputs: BTreeSet<String>,
    missing_progress_outputs: BTreeSet<String>,
    missing_feedback_inputs: BTreeSet<String>,
    default_reentry_flows: Vec<DefaultReentryFlow>,
    has_exit_path: bool,
    has_conditionless_gateway_cycle: bool,
}

fn unbounded_control_cycle_issue(
    source: &BpmnSourceFile,
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
    evidence: &LoopRiskEvidence,
) -> LintIssue {
    let process_id = process.key.process_id.as_ref();
    let cycle_node_ids = sorted_node_ids(process, component);
    let cycle_summary = cycle_node_ids.join(" -> ");
    let route_variable_list = sorted_set_values(&evidence.route_variables);
    let missing_progress_list = sorted_set_values(&evidence.missing_progress_outputs);
    let missing_feedback_list = sorted_set_values(&evidence.missing_feedback_inputs);
    let line_fixes = loop_progress_line_fixes(process, metadata, component, evidence);
    let xml_fixes = line_fix_xml_strings(&line_fixes);
    let help = loop_progress_help(process, metadata, component, evidence);
    let contract_message = loop_progress_contract_message();
    let guidance = if xml_fixes.is_empty() {
        vec!["No exact XML line fix inferred.".to_string()]
    } else {
        xml_fixes.clone()
    };
    let llm_fix_prompt = if xml_fixes.is_empty() {
        "No exact XML line fix inferred.".to_string()
    } else {
        xml_fixes.join("\n")
    };

    let mut issue = LintIssue::new(
        "bpmn.loop_risk.unbounded_control_cycle",
        "Workflow cycle is missing a complete loop-progress contract",
        format!(
            "Process '{process_id}' contains a cyclic path [{cycle_summary}] that can re-enter host/user work without a complete qianji progress contract."
        ),
        "Cycle progress state is incomplete.",
        guidance,
        llm_fix_prompt,
        json!({
            "process_id": process_id,
            "cycle_node_ids": cycle_node_ids,
            "task_node_ids": evidence.task_node_ids.clone(),
            "gateway_ids": evidence.gateway_ids.clone(),
            "route_variables": route_variable_list,
            "updated_variables_in_cycle": sorted_set_values(&evidence.updated_variables),
            "user_outputs_in_cycle": sorted_set_values(&evidence.user_outputs),
            "worker_inputs_in_cycle": sorted_set_values(&evidence.worker_inputs),
            "missing_progress_outputs": missing_progress_list,
            "missing_feedback_inputs": missing_feedback_list,
            "default_reentry_flows": evidence.default_reentry_flows,
            "has_exit_path": evidence.has_exit_path,
            "has_conditionless_gateway_cycle": evidence.has_conditionless_gateway_cycle,
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "qianji.bpmn.loop.progress.v1",
        "contract_message": contract_message,
        "strategy": "make_cycle_progress_explicit_or_remove_back_edge",
        "line_fixes": line_fixes,
        "actions": [{
            "op": "inspect_cycle",
            "nodes": sorted_node_ids(process, component),
        }, {
            "op": "ensure_unconditional_default_exit",
            "required": true,
            "default_reentry_flows": evidence.default_reentry_flows,
        }, {
            "op": "add_qianji_outputs_inside_cycle",
            "variables": sorted_set_values(&evidence.missing_progress_outputs),
        }, {
            "op": "add_qianji_inputs_to_question_service",
            "variables": sorted_set_values(&evidence.missing_feedback_inputs),
        }],
        "forbid": [
            "repeating a userTask question without feeding the user's prior answer into the next in-cycle serviceTask",
            "routing a cycle on variables that no task inside the cycle declares in qianji:outputs",
            "using a conditional default branch instead of an unconditional exit"
        ]
    }));

    if let Some(span) = primary_cycle_span(
        process,
        metadata,
        component,
        evidence,
        &evidence.gateway_ids,
        &cycle_node_ids,
    ) {
        issue = issue.with_source_diagnostic(LintSourceDiagnostic::new(
            &source.source_id,
            LintSourceSpan::new(span.start, span.end),
            "cycle needs explicit progress state",
            help,
        ));
    }

    issue
}

fn strongly_connected_components(process: &BpmnProcessSpec) -> Vec<Vec<usize>> {
    let mut tarjan = Tarjan::new(process);
    for node_index in 0..process.nodes.len() {
        if tarjan.indices[node_index].is_none() {
            tarjan.connect(node_index);
        }
    }
    tarjan.components
}

struct Tarjan<'a> {
    process: &'a BpmnProcessSpec,
    next_index: usize,
    stack: Vec<usize>,
    on_stack: Vec<bool>,
    indices: Vec<Option<usize>>,
    lowlinks: Vec<usize>,
    components: Vec<Vec<usize>>,
}

impl<'a> Tarjan<'a> {
    fn new(process: &'a BpmnProcessSpec) -> Self {
        let node_count = process.nodes.len();
        Self {
            process,
            next_index: 0,
            stack: Vec::new(),
            on_stack: vec![false; node_count],
            indices: vec![None; node_count],
            lowlinks: vec![0; node_count],
            components: Vec::new(),
        }
    }

    fn connect(&mut self, node_index: usize) {
        self.indices[node_index] = Some(self.next_index);
        self.lowlinks[node_index] = self.next_index;
        self.next_index += 1;
        self.stack.push(node_index);
        self.on_stack[node_index] = true;

        if let Some(edge_indices) = outgoing_edge_indices(self.process, node_index) {
            for edge_index in edge_indices {
                let target_index = self.process.edges[*edge_index as usize].to as usize;
                if self.indices[target_index].is_none() {
                    self.connect(target_index);
                    self.lowlinks[node_index] =
                        self.lowlinks[node_index].min(self.lowlinks[target_index]);
                } else if self.on_stack[target_index] {
                    let target_order = self.indices[target_index].unwrap_or_default();
                    self.lowlinks[node_index] = self.lowlinks[node_index].min(target_order);
                }
            }
        }

        if self.lowlinks[node_index] == self.indices[node_index].unwrap_or_default() {
            let mut component = Vec::new();
            while let Some(member_index) = self.stack.pop() {
                self.on_stack[member_index] = false;
                component.push(member_index);
                if member_index == node_index {
                    break;
                }
            }
            component.sort_unstable();
            self.components.push(component);
        }
    }
}

fn is_cyclic_component(process: &BpmnProcessSpec, component: &[usize]) -> bool {
    if component.len() > 1 {
        return true;
    }
    let Some(node_index) = component.first().copied() else {
        return false;
    };
    outgoing_edge_indices(process, node_index).is_some_and(|edge_indices| {
        edge_indices
            .iter()
            .any(|edge_index| process.edges[*edge_index as usize].to as usize == node_index)
    })
}

fn component_has_exit_path(process: &BpmnProcessSpec, component_set: &HashSet<usize>) -> bool {
    component_set.iter().any(|node_index| {
        outgoing_edge_indices(process, *node_index).is_some_and(|edge_indices| {
            edge_indices.iter().any(|edge_index| {
                let target_index = process.edges[*edge_index as usize].to as usize;
                !component_set.contains(&target_index)
            })
        })
    })
}

fn default_reentry_flows(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component_set: &HashSet<usize>,
    gateway_ids: &[String],
) -> Vec<DefaultReentryFlow> {
    let node_indices = node_id_to_index(process);
    gateway_ids
        .iter()
        .filter_map(|gateway_id| {
            let flow_id = metadata.gateway_default_flows.get(gateway_id)?;
            let flow = metadata.sequence_flows.get(flow_id)?;
            let target_index = node_indices.get(flow.target_ref.as_str())?;
            if !component_set.contains(target_index) {
                return None;
            }
            Some(DefaultReentryFlow {
                gateway_id: gateway_id.clone(),
                flow_id: flow_id.clone(),
                target_id: flow.target_ref.clone(),
                suggested_exit_target_id: suggested_default_exit_target(
                    process,
                    component_set,
                    gateway_id,
                ),
            })
        })
        .collect()
}

fn node_id_to_index(process: &BpmnProcessSpec) -> HashMap<&str, usize> {
    process
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.bpmn_id.as_ref(), index))
        .collect()
}

fn suggested_default_exit_target(
    process: &BpmnProcessSpec,
    component_set: &HashSet<usize>,
    gateway_id: &str,
) -> Option<String> {
    let gateway_index = process
        .nodes
        .iter()
        .position(|node| node.bpmn_id.as_ref() == gateway_id)
        .unwrap_or_default();

    if let Some(target) = source_component_entry_candidate(process, component_set, gateway_index) {
        return Some(target);
    }

    let incoming_counts = incoming_edge_counts(process);
    let mut candidates = process
        .nodes
        .iter()
        .enumerate()
        .filter(|(index, node)| {
            !component_set.contains(index)
                && node.kind != BpmnNodeKind::StartEvent
                && incoming_counts.get(*index).copied().unwrap_or_default() == 0
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(index, node)| {
        (
            *index <= gateway_index,
            !is_host_task(&node.kind),
            !matches!(node.kind, BpmnNodeKind::EndEvent),
            *index,
        )
    });
    candidates.first().map(|(_, node)| node.bpmn_id.to_string())
}

fn source_component_entry_candidate(
    process: &BpmnProcessSpec,
    current_component_set: &HashSet<usize>,
    gateway_index: usize,
) -> Option<String> {
    let mut candidates = strongly_connected_components(process)
        .into_iter()
        .filter(|component| {
            !component
                .iter()
                .any(|index| current_component_set.contains(index))
        })
        .filter(|component| {
            !component
                .iter()
                .any(|index| process.nodes[*index].kind == BpmnNodeKind::StartEvent)
        })
        .filter(|component| component_has_no_external_incoming(process, component))
        .filter_map(|component| source_component_entry(process, &component, gateway_index))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| {
        (
            candidate.index <= gateway_index,
            !candidate.is_host_task,
            !candidate.is_end_event,
            candidate.index,
        )
    });
    candidates
        .first()
        .map(|candidate| candidate.node_id.clone())
}

struct SourceComponentEntry {
    node_id: String,
    index: usize,
    is_host_task: bool,
    is_end_event: bool,
}

fn component_has_no_external_incoming(process: &BpmnProcessSpec, component: &[usize]) -> bool {
    let component_set = component.iter().copied().collect::<HashSet<_>>();
    !process.edges.iter().any(|edge| {
        let Ok(source) = usize::try_from(edge.from) else {
            return false;
        };
        let Ok(target) = usize::try_from(edge.to) else {
            return false;
        };
        component_set.contains(&target) && !component_set.contains(&source)
    })
}

fn source_component_entry(
    process: &BpmnProcessSpec,
    component: &[usize],
    gateway_index: usize,
) -> Option<SourceComponentEntry> {
    component
        .iter()
        .map(|index| {
            let node = &process.nodes[*index];
            SourceComponentEntry {
                node_id: node.bpmn_id.to_string(),
                index: *index,
                is_host_task: is_host_task(&node.kind),
                is_end_event: node.kind == BpmnNodeKind::EndEvent,
            }
        })
        .min_by_key(|candidate| {
            (
                candidate.index <= gateway_index,
                !candidate.is_host_task,
                !candidate.is_end_event,
                candidate.index,
            )
        })
}

fn incoming_edge_counts(process: &BpmnProcessSpec) -> Vec<usize> {
    let mut counts = vec![0; process.nodes.len()];
    for edge in &process.edges {
        if let Ok(index) = usize::try_from(edge.to)
            && let Some(count) = counts.get_mut(index)
        {
            *count += 1;
        }
    }
    counts
}

fn route_variables(process: &BpmnProcessSpec, component: &[usize]) -> BTreeSet<String> {
    component
        .iter()
        .filter(|node_index| is_gateway(process.nodes[**node_index].gateway_kind.as_ref()))
        .flat_map(|node_index| {
            let Some(edge_indices) = outgoing_edge_indices(process, *node_index) else {
                return Vec::new();
            };
            edge_indices
                .iter()
                .filter_map(|edge_index| {
                    process.edges[*edge_index as usize]
                        .condition_expression
                        .as_deref()
                        .and_then(gateway_condition_variable_path)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn gateway_condition_variable_path(condition: &str) -> Option<String> {
    match parse_gateway_condition_summary(condition)? {
        GatewayConditionSummary::BooleanPath { path, .. } => Some(path),
        GatewayConditionSummary::NumericComparison { lhs, .. } => Some(lhs),
    }
}

fn task_node_ids(process: &BpmnProcessSpec, component: &[usize]) -> Vec<String> {
    component
        .iter()
        .filter(|node_index| is_host_task(&process.nodes[**node_index].kind))
        .map(|node_index| process.nodes[*node_index].bpmn_id.to_string())
        .collect()
}

fn gateway_node_ids(process: &BpmnProcessSpec, component: &[usize]) -> Vec<String> {
    component
        .iter()
        .filter(|node_index| is_gateway(process.nodes[**node_index].gateway_kind.as_ref()))
        .map(|node_index| process.nodes[*node_index].bpmn_id.to_string())
        .collect()
}

fn updated_variables(metadata: &ProcessMetadata, task_node_ids: &[String]) -> BTreeSet<String> {
    task_node_ids
        .iter()
        .flat_map(|node_id| metadata.task_outputs.get(node_id).into_iter().flatten())
        .cloned()
        .collect()
}

fn user_task_outputs(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
) -> BTreeSet<String> {
    component
        .iter()
        .filter(|node_index| process.nodes[**node_index].kind == BpmnNodeKind::UserTask)
        .flat_map(|node_index| {
            let node_id = process.nodes[*node_index].bpmn_id.as_ref();
            metadata.task_outputs.get(node_id).into_iter().flatten()
        })
        .cloned()
        .collect()
}

fn worker_task_inputs(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
) -> BTreeSet<String> {
    component
        .iter()
        .filter(|node_index| is_state_worker_task(&process.nodes[**node_index].kind))
        .flat_map(|node_index| {
            let node_id = process.nodes[*node_index].bpmn_id.as_ref();
            metadata.task_inputs.get(node_id).into_iter().flatten()
        })
        .cloned()
        .collect()
}

fn worker_task_outputs(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
) -> BTreeSet<String> {
    component
        .iter()
        .filter(|node_index| is_state_worker_task(&process.nodes[**node_index].kind))
        .flat_map(|node_index| {
            let node_id = process.nodes[*node_index].bpmn_id.as_ref();
            metadata.task_outputs.get(node_id).into_iter().flatten()
        })
        .cloned()
        .collect()
}

fn undeclared_variables<'a>(
    declared: &BTreeSet<String>,
    variables: impl Iterator<Item = &'a str>,
) -> BTreeSet<String> {
    variables
        .filter(|variable| !declares_variable(declared, variable))
        .map(ToString::to_string)
        .collect()
}

fn declares_variable(declared: &BTreeSet<String>, variable_path: &str) -> bool {
    let root = variable_path.split('.').next().unwrap_or(variable_path);
    declared.contains(variable_path) || declared.contains(root)
}

fn loop_progress_line_fixes(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
    evidence: &LoopRiskEvidence,
) -> Vec<Value> {
    let Some(task_id) = progress_owner_task_id(process, metadata, component) else {
        return Vec::new();
    };

    let mut fixes = Vec::new();
    for flow in &evidence.default_reentry_flows {
        if let Some(target_id) = flow.suggested_exit_target_id.as_deref() {
            let target = format!("{}.default_exit_flow", flow.gateway_id);
            let xml = format!(
                "<sequenceFlow id=\"{}\" sourceRef=\"{}\" targetRef=\"{}\"/>",
                flow.flow_id, flow.gateway_id, target_id
            );
            fixes.push(line_fix(
                metadata
                    .sequence_flows
                    .get(&flow.flow_id)
                    .map(|sequence_flow| &sequence_flow.span),
                &target,
                &xml,
            ));
        }
    }

    if !evidence.missing_feedback_inputs.is_empty() {
        let mut inputs = metadata
            .task_inputs
            .get(&task_id)
            .cloned()
            .unwrap_or_default();
        inputs.extend(evidence.missing_feedback_inputs.iter().cloned());
        let target = format!("{task_id}.qianji:inputs");
        let xml = format!(
            "<qianji:inputs>{}</qianji:inputs>",
            sorted_set_values(&inputs).join(",")
        );
        fixes.push(line_fix(
            metadata
                .task_input_spans
                .get(&task_id)
                .or_else(|| metadata.node_spans.get(&task_id)),
            &target,
            &xml,
        ));
    }

    if !evidence.missing_progress_outputs.is_empty() {
        let mut outputs = metadata
            .task_outputs
            .get(&task_id)
            .cloned()
            .unwrap_or_default();
        outputs.extend(evidence.missing_progress_outputs.iter().cloned());
        let target = format!("{task_id}.qianji:outputs");
        let xml = format!(
            "<qianji:outputs>{}</qianji:outputs>",
            sorted_set_values(&outputs).join(",")
        );
        fixes.push(line_fix(
            metadata
                .task_output_spans
                .get(&task_id)
                .or_else(|| metadata.node_spans.get(&task_id)),
            &target,
            &xml,
        ));
    }

    fixes
}

fn progress_owner_task_id(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
) -> Option<String> {
    component
        .iter()
        .filter(|node_index| is_state_worker_task(&process.nodes[**node_index].kind))
        .find(|node_index| {
            let node_id = process.nodes[**node_index].bpmn_id.as_ref();
            metadata
                .task_outputs
                .get(node_id)
                .is_some_and(|outputs| outputs.iter().any(|output| is_prompt_output(output)))
        })
        .or_else(|| {
            component
                .iter()
                .find(|node_index| is_state_worker_task(&process.nodes[**node_index].kind))
        })
        .map(|node_index| process.nodes[*node_index].bpmn_id.to_string())
}

fn line_fix(span: Option<&Range<usize>>, target: &str, xml: &str) -> Value {
    let mut fix = json!({
        "target": target,
        "xml": xml,
    });
    if let Some(span) = span {
        fix["offset"] = json!(span.start);
    } else {
        fix["line"] = json!("primary");
    }
    fix
}

fn line_fix_xml_strings(line_fixes: &[Value]) -> Vec<String> {
    line_fixes
        .iter()
        .filter_map(|line_fix| line_fix.get("xml").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect()
}

fn loop_progress_help(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
    evidence: &LoopRiskEvidence,
) -> String {
    if let Some(flow) = evidence.default_reentry_flows.first() {
        let target = flow.suggested_exit_target_id.as_deref().map_or_else(
            || "the normal next node outside the cycle".to_string(),
            |target| format!("`{target}`"),
        );
        return format!(
            "Default flow `{}` from `{}` currently re-enters `{}`. Retarget that default flow to {target}; keep repeat paths conditional.",
            flow.flow_id, flow.gateway_id, flow.target_id,
        );
    }

    let task_id = progress_owner_task_id(process, metadata, component);
    let mut requirements = Vec::new();
    if !evidence.missing_progress_outputs.is_empty() {
        requirements.push(format!(
            "update {} inside the cycle",
            sorted_set_values(&evidence.missing_progress_outputs).join(", ")
        ));
    }
    if !evidence.missing_feedback_inputs.is_empty() {
        let feedback = sorted_set_values(&evidence.missing_feedback_inputs).join(", ");
        if let Some(task_id) = task_id.as_deref() {
            requirements.push(format!(
                "feed {feedback} into {task_id} before the next prompt"
            ));
        } else {
            requirements.push(format!(
                "feed {feedback} into the in-cycle service task before the next prompt"
            ));
        }
    }

    let requirement = if requirements.is_empty() {
        "make loop progress explicit".to_string()
    } else {
        requirements.join(" and ")
    };

    if let Some(default_flow) = default_exit_flow_id(metadata, evidence) {
        format!(
            "The loop must {requirement} and keep {default_flow} as the unconditional default exit."
        )
    } else {
        format!("The loop must {requirement} and include one unconditional default exit.")
    }
}

fn loop_progress_contract_message() -> &'static str {
    "qianji.bpmn.loop.progress.v1 requires in-cycle tasks to consume user feedback and emit the gateway route state."
}

fn default_exit_flow_id(metadata: &ProcessMetadata, evidence: &LoopRiskEvidence) -> Option<String> {
    evidence
        .gateway_ids
        .iter()
        .find_map(|gateway_id| metadata.gateway_default_flows.get(gateway_id).cloned())
}

fn primary_cycle_span(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
    evidence: &LoopRiskEvidence,
    gateway_ids: &[String],
    cycle_node_ids: &[String],
) -> Option<Range<usize>> {
    if let Some(task_id) = progress_owner_task_id(process, metadata, component) {
        if !evidence.missing_feedback_inputs.is_empty()
            && let Some(span) = metadata
                .task_input_spans
                .get(&task_id)
                .or_else(|| metadata.node_spans.get(&task_id))
        {
            return Some(span.clone());
        }
        if !evidence.missing_progress_outputs.is_empty()
            && let Some(span) = metadata
                .task_output_spans
                .get(&task_id)
                .or_else(|| metadata.node_spans.get(&task_id))
        {
            return Some(span.clone());
        }
    }

    gateway_ids
        .iter()
        .chain(cycle_node_ids.iter())
        .find_map(|node_id| metadata.node_spans.get(node_id).cloned())
}

fn sorted_node_ids(process: &BpmnProcessSpec, component: &[usize]) -> Vec<String> {
    component
        .iter()
        .map(|node_index| process.nodes[*node_index].bpmn_id.to_string())
        .collect()
}

fn sorted_set_values(values: &BTreeSet<String>) -> Vec<String> {
    values.iter().cloned().collect()
}

fn is_host_task(kind: &BpmnNodeKind) -> bool {
    matches!(
        kind,
        BpmnNodeKind::ServiceTask
            | BpmnNodeKind::ScriptTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
            | BpmnNodeKind::SendTask
            | BpmnNodeKind::ReceiveTask
    )
}

fn is_state_worker_task(kind: &BpmnNodeKind) -> bool {
    matches!(
        kind,
        BpmnNodeKind::ServiceTask | BpmnNodeKind::ScriptTask | BpmnNodeKind::BusinessRuleTask
    )
}

fn is_gateway(kind: Option<&BpmnGatewayKind>) -> bool {
    matches!(
        kind,
        Some(
            BpmnGatewayKind::Exclusive
                | BpmnGatewayKind::Inclusive
                | BpmnGatewayKind::Parallel
                | BpmnGatewayKind::EventBased
        )
    )
}

fn is_prompt_output(output: &str) -> bool {
    let normalized = output.to_ascii_lowercase();
    [
        "question",
        "questions",
        "choice",
        "choices",
        "prompt",
        "clarif",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn collect_process_metadata(source: &BpmnSourceFile) -> HashMap<String, ProcessMetadata> {
    MetadataCollector::new(source).collect()
}

struct MetadataCollector<'a> {
    source: &'a BpmnSourceFile,
    processes: HashMap<String, ProcessMetadata>,
    active_process_id: Option<String>,
    active_metadata: ProcessMetadata,
    active_task: Option<ActiveTask>,
}

impl<'a> MetadataCollector<'a> {
    fn new(source: &'a BpmnSourceFile) -> Self {
        Self {
            source,
            processes: HashMap::new(),
            active_process_id: None,
            active_metadata: ProcessMetadata::default(),
            active_task: None,
        }
    }

    fn collect(mut self) -> HashMap<String, ProcessMetadata> {
        let mut reader = Reader::from_str(&self.source.contents);
        reader.config_mut().trim_text(false);
        loop {
            match reader.read_event() {
                Ok(Event::Start(event)) => self.handle_start(&reader, &event),
                Ok(Event::Empty(event)) => self.handle_empty(&reader, &event),
                Ok(Event::Text(event)) => {
                    if let Some(task) = self.active_task.as_mut()
                        && let Ok(text) = event.decode()
                    {
                        append_task_variables(task, &text);
                    }
                }
                Ok(Event::GeneralRef(event)) => self.handle_general_ref(&event),
                Ok(Event::End(event)) => self.handle_end(event.name().as_ref()),
                Ok(Event::Eof) | Err(_) => return self.processes,
                Ok(_) => {}
            }
        }
    }

    fn handle_start(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        let name = local_name(event.name().as_ref());
        match name.as_str() {
            "process" => self.start_process(reader, event),
            tag if is_task_tag(tag) => self.start_task(reader, event),
            "sequenceFlow" => self.record_sequence_flow(reader, event),
            tag if is_span_only_node_tag(tag) => self.record_span(reader, event),
            "inputs" if is_qianji_name(event, "inputs") => {
                self.record_active_task_io_span(reader, event, true);
                if let Some(task) = self.active_task.as_mut() {
                    task.in_inputs = true;
                }
            }
            "outputs" if is_qianji_name(event, "outputs") => {
                self.record_active_task_io_span(reader, event, false);
                if let Some(task) = self.active_task.as_mut() {
                    task.in_outputs = true;
                }
            }
            _ => {}
        }
    }

    fn handle_empty(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        let name = local_name(event.name().as_ref());
        match name.as_str() {
            "inputs" if is_qianji_name(event, "inputs") => {
                self.record_active_task_io_span(reader, event, true);
            }
            "outputs" if is_qianji_name(event, "outputs") => {
                self.record_active_task_io_span(reader, event, false);
            }
            "sequenceFlow" => self.record_sequence_flow(reader, event),
            tag if is_task_tag(tag) => self.record_empty_task(reader, event),
            tag if is_span_only_node_tag(tag) => self.record_span(reader, event),
            _ => {}
        }
    }

    fn handle_general_ref(&mut self, event: &quick_xml::events::BytesRef<'_>) {
        if let Some(task) = self.active_task.as_mut() {
            let reference = event.decode().ok();
            let mut text = String::new();
            append_entity_reference(&mut text, reference.as_deref());
            append_task_variables(task, &text);
        }
    }

    fn handle_end(&mut self, raw_name: &[u8]) {
        let name = local_name(raw_name);
        match name.as_str() {
            "inputs" => {
                if let Some(task) = self.active_task.as_mut() {
                    task.in_inputs = false;
                }
            }
            "outputs" => {
                if let Some(task) = self.active_task.as_mut() {
                    task.in_outputs = false;
                }
            }
            tag if is_task_tag(tag) => self.finish_task(),
            "process" => self.finish_process(),
            _ => {}
        }
    }

    fn start_process(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        self.active_process_id = attribute_value(reader, event, "id");
        self.active_metadata = ProcessMetadata::default();
    }

    fn start_task(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if self.active_process_id.is_none() {
            return;
        }
        let Some(id) = attribute_value(reader, event, "id") else {
            return;
        };
        self.record_span_for_id(reader, event, &id);
        self.active_task = Some(ActiveTask {
            id,
            ..ActiveTask::default()
        });
    }

    fn record_empty_task(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if self.active_process_id.is_none() {
            return;
        }
        let Some(id) = attribute_value(reader, event, "id") else {
            return;
        };
        self.record_span_for_id(reader, event, &id);
        self.active_metadata
            .task_inputs
            .entry(id.clone())
            .or_default();
        self.active_metadata.task_outputs.entry(id).or_default();
    }

    fn record_span(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if self.active_process_id.is_none() {
            return;
        }
        if let Some(id) = attribute_value(reader, event, "id") {
            self.record_span_for_id(reader, event, &id);
        }
    }

    fn record_sequence_flow(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>) {
        if self.active_process_id.is_none() {
            return;
        }
        let Some(id) = attribute_value(reader, event, "id") else {
            return;
        };
        if attribute_value(reader, event, "sourceRef").is_none() {
            return;
        }
        let Some(target_ref) = attribute_value(reader, event, "targetRef") else {
            return;
        };
        let Some(event_end) = reader_position(reader) else {
            return;
        };
        let Some(span) = start_or_empty_event_span(&self.source.contents, event_end, event) else {
            return;
        };
        self.active_metadata
            .sequence_flows
            .insert(id, SequenceFlowMetadata { target_ref, span });
    }

    fn record_span_for_id(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>, id: &str) {
        record_node_span(
            &mut self.active_metadata,
            &self.source.contents,
            reader,
            event,
            id,
        );
        if let Some(default_flow) = attribute_value(reader, event, "default") {
            self.active_metadata
                .gateway_default_flows
                .insert(id.to_string(), default_flow);
        }
    }

    fn record_active_task_io_span(
        &mut self,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_input: bool,
    ) {
        let Some(task_id) = self.active_task.as_ref().map(|task| task.id.clone()) else {
            return;
        };
        let Some(event_end) = reader_position(reader) else {
            return;
        };
        let Some(span) = start_or_empty_event_span(&self.source.contents, event_end, event) else {
            return;
        };
        let spans = if is_input {
            &mut self.active_metadata.task_input_spans
        } else {
            &mut self.active_metadata.task_output_spans
        };
        spans.insert(task_id, span);
    }

    fn finish_task(&mut self) {
        if let Some(task) = self.active_task.take() {
            self.active_metadata
                .task_inputs
                .insert(task.id.clone(), task.inputs);
            self.active_metadata
                .task_outputs
                .insert(task.id, task.outputs);
        }
    }

    fn finish_process(&mut self) {
        if let Some(process_id) = self.active_process_id.take() {
            self.processes
                .insert(process_id, std::mem::take(&mut self.active_metadata));
        }
    }
}

fn append_task_variables(task: &mut ActiveTask, text: &str) {
    if task.in_inputs {
        task.inputs.extend(parse_variable_names(text));
    }
    if task.in_outputs {
        task.outputs.extend(parse_variable_names(text));
    }
}

fn outgoing_edge_indices(process: &BpmnProcessSpec, node_index: usize) -> Option<&[u32]> {
    let node_index = u32::try_from(node_index).ok()?;
    Some(process.outgoing_edge_indices(node_index))
}

fn is_task_tag(tag: &str) -> bool {
    matches!(
        tag,
        "serviceTask"
            | "userTask"
            | "manualTask"
            | "businessRuleTask"
            | "scriptTask"
            | "sendTask"
            | "receiveTask"
    )
}

fn is_span_only_node_tag(tag: &str) -> bool {
    matches!(
        tag,
        "exclusiveGateway"
            | "inclusiveGateway"
            | "parallelGateway"
            | "eventBasedGateway"
            | "startEvent"
            | "endEvent"
    )
}

fn record_node_span(
    metadata: &mut ProcessMetadata,
    contents: &str,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    node_id: &str,
) {
    if let Some(event_end) = reader_position(reader)
        && let Some(span) = start_or_empty_event_span(contents, event_end, event)
    {
        metadata.node_spans.insert(node_id.to_string(), span);
    }
}

fn start_or_empty_event_span(
    contents: &str,
    event_end: usize,
    event: &BytesStart<'_>,
) -> Option<Range<usize>> {
    let raw: &[u8] = event.as_ref();
    [2, 3].into_iter().find_map(|extra| {
        let start = event_end.checked_sub(raw.len() + extra)?;
        contents
            .as_bytes()
            .get(start)
            .is_some_and(|byte| *byte == b'<')
            .then_some(start..event_end)
    })
}

fn parse_variable_names(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn reader_position(reader: &Reader<&[u8]>) -> Option<usize> {
    usize::try_from(reader.buffer_position()).ok()
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

fn is_qianji_name(event: &BytesStart<'_>, expected_local_name: &str) -> bool {
    event_name_parts(event.name().as_ref()) == Some(("qianji", expected_local_name))
}

fn local_name(name: &[u8]) -> String {
    let raw = std::str::from_utf8(name).unwrap_or_default();
    raw.rsplit_once(':')
        .map_or(raw, |(_, local)| local)
        .to_string()
}

fn event_name_parts(name: &[u8]) -> Option<(&str, &str)> {
    let raw = std::str::from_utf8(name).ok()?;
    raw.rsplit_once(':')
}

fn append_entity_reference(target: &mut String, reference: Option<&str>) {
    if let Some(reference) = reference
        && let Some(resolved) = resolve_predefined_entity(reference)
    {
        target.push_str(resolved);
    }
}
