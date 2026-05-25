use super::{
    BpmnProcessSpec, LoopRiskEvidence, ProcessMetadata, progress_owner_task_id, sorted_set_values,
};

pub(in crate::lint::bpmn::loop_risk) fn loop_progress_help(
    process: &BpmnProcessSpec,
    metadata: &ProcessMetadata,
    component: &[usize],
    evidence: &LoopRiskEvidence,
) -> String {
    if let Some(flow) = evidence.default_reentry_flows.first() {
        let target = flow.suggested_exit_target.as_deref().map_or_else(
            || "the normal next node outside the cycle".to_string(),
            |target| format!("`{target}`"),
        );
        return format!(
            "Default flow `{}` from `{}` currently re-enters `{}`. Retarget that default flow to {target}; keep repeat paths conditional.",
            flow.sequence_flow, flow.gateway_node, flow.target_node,
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

pub(in crate::lint::bpmn::loop_risk) fn loop_progress_contract_message() -> &'static str {
    "native BPMN loop progress requires in-cycle tasks to consume user feedback and emit the gateway route state through standard IO metadata."
}

pub(in crate::lint::bpmn::loop_risk) fn default_exit_flow_id(
    metadata: &ProcessMetadata,
    evidence: &LoopRiskEvidence,
) -> Option<String> {
    evidence
        .gateway_ids
        .iter()
        .find_map(|gateway_id| metadata.gateway_default_flows.get(gateway_id).cloned())
}
