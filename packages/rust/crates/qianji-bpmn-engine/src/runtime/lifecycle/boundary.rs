use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnProcessSpec, NodeRuntimeStatus, Result,
};
use crate::runtime::lifecycle::state;

pub(crate) fn cancel_attached_boundary_siblings(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    owner_node_index: BpmnNodeIndex,
    selected_boundary_indices: &[BpmnNodeIndex],
) -> Result<()> {
    for boundary in process.boundary_events_for_attached_node(owner_node_index) {
        if selected_boundary_indices.contains(&boundary.index) {
            continue;
        }
        let _ = process.event_for_node(boundary.index).ok_or_else(|| {
            BpmnEngineError::MissingRequiredNodeElement {
                process_id: (process.key.process_id.to_string()).into(),
                node_id: (boundary.bpmn_id.to_string()).into(),
                element: "event_definition",
            }
        })?;
        state::set_node_status(instance, boundary.index, NodeRuntimeStatus::Cancelled);
    }
    Ok(())
}
