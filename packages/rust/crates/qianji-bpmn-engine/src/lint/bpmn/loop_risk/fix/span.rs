use super::{BpmnProcessSpec, LoopRiskEvidence, ProcessMetadata, Range, progress_owner_task_id};

pub(in crate::lint::bpmn::loop_risk) fn primary_cycle_span(
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
