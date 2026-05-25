use super::{
    BTreeSet, BpmnProcessSpec, BpmnSourceFile, HashSet, LintIssue, LintSourceDiagnostic,
    LintSourceSpan, LoopRiskEvidence, ProcessMetadata, component_has_exit_path,
    default_reentry_flows, gateway_node_ids, is_cyclic_component, is_prompt_output, json,
    line_fix_xml_strings, loop_progress_contract_message, loop_progress_help,
    loop_progress_line_fixes, primary_cycle_span, route_variables, sorted_node_ids,
    sorted_set_values, strongly_connected_components, task_node_ids, undeclared_variables,
    updated_variables, user_task_outputs, worker_task_inputs, worker_task_outputs,
};

pub(super) fn process_loop_risk_issues(
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

pub(super) fn loop_risk_issue(
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

pub(super) fn unbounded_control_cycle_issue(
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

    let mut issue = LintIssue::from_parts(
        "bpmn.loop_risk.unbounded_control_cycle",
        "Workflow cycle is missing a complete loop-progress contract",
        format!(
            "Process '{process_id}' contains a cyclic path [{cycle_summary}] that can re-enter host/user work without a complete native BPMN progress contract."
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
        "contract": "bpmn.native.loop.progress.v1",
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
            "op": "add_native_outputs_inside_cycle",
            "variables": sorted_set_values(&evidence.missing_progress_outputs),
        }, {
            "op": "add_native_inputs_to_question_service",
            "variables": sorted_set_values(&evidence.missing_feedback_inputs),
        }],
        "forbid": [
            "repeating a userTask question without feeding the user's prior answer into the next in-cycle serviceTask",
            "routing a cycle on variables that no task inside the cycle declares through native BPMN output metadata",
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
