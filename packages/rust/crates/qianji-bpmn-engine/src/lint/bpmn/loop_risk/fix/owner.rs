use super::{BpmnProcessSpec, ProcessMetadata, is_prompt_output, is_state_worker_task};

pub(in crate::lint::bpmn::loop_risk) fn progress_owner_task_id(
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
