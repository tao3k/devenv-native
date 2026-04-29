use super::{
    BpmnProcessSpec, LoopRiskEvidence, ProcessMetadata, Value, line_fix, native_input_fragment,
    native_output_fragment, progress_owner_task_id,
};

pub(in crate::lint::bpmn::loop_risk) use super::guidance::{
    loop_progress_contract_message, loop_progress_help,
};
pub(in crate::lint::bpmn::loop_risk) use super::span::primary_cycle_span;

pub(in crate::lint::bpmn::loop_risk) fn loop_progress_line_fixes(
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
        if let Some(target_id) = flow.suggested_exit_target.as_deref() {
            let target = format!("{}.default_exit_flow", flow.gateway_node);
            let xml = format!(
                "<sequenceFlow id=\"{}\" sourceRef=\"{}\" targetRef=\"{}\"/>",
                flow.sequence_flow, flow.gateway_node, target_id
            );
            fixes.push(line_fix(
                metadata
                    .sequence_flows
                    .get(&flow.sequence_flow)
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
        let target = format!("{task_id}.native_inputs");
        let xml = native_input_fragment(&task_id, &inputs);
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
        let target = format!("{task_id}.native_outputs");
        let xml = native_output_fragment(&task_id, &outputs);
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

pub(in crate::lint::bpmn::loop_risk) fn line_fix_xml_strings(line_fixes: &[Value]) -> Vec<String> {
    line_fixes
        .iter()
        .filter_map(|line_fix| line_fix.get("xml").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect()
}
