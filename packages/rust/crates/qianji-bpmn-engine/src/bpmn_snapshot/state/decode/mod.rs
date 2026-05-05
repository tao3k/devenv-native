//! bpmn snapshot state event decoding branch wiring for focused BPMN/DMN owner leaves.

mod capture;
mod tags;

pub(in crate::bpmn_snapshot::state) use capture::{
    bounds_from_event, data_association_expression_from_event, data_input_output_from_event,
    data_state_from_event, font_from_event, io_binding_from_event, label_from_event,
    resource_role_from_event, root_from_event, waypoint_from_event,
};
pub(in crate::bpmn_snapshot::state) use tags::{
    is_artifact_container, is_choreography_activity_tag, is_collaboration_container,
    is_conversation_node_tag, is_data_association_tag, is_flow_element_metadata_owner_tag,
    is_global_task_tag, is_resource_role_tag,
};
