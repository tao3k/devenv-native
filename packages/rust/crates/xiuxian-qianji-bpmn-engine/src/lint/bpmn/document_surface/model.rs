#[derive(Default)]
pub(super) struct CollaborationCounts {
    pub(super) participant: usize,
    pub(super) participant_interface_ref: usize,
    pub(super) participant_end_point_ref: usize,
    pub(super) participant_multiplicity: usize,
    pub(super) message_flow: usize,
    pub(super) conversation_node: usize,
    pub(super) conversation_link: usize,
    pub(super) conversation_association: usize,
    pub(super) participant_association: usize,
    pub(super) message_flow_association: usize,
    pub(super) correlation_key: usize,
    pub(super) choreography_activity: usize,
    pub(super) association: usize,
    pub(super) group: usize,
    pub(super) text_annotation: usize,
}

#[derive(Debug, Default)]
pub(super) struct ProcessCallableCounts {
    pub(super) support: usize,
    pub(super) property: usize,
    pub(super) correlation_subscription: usize,
    pub(super) correlation_binding: usize,
    pub(super) process_io_binding: usize,
    pub(super) global_task_io_specification: usize,
    pub(super) global_task_io_binding: usize,
}

#[derive(Debug, Default)]
pub(super) struct ResourceRoleCounts {
    pub(super) process_role: usize,
    pub(super) global_task_role: usize,
    pub(super) parameter_binding: usize,
    pub(super) assignment_expression: usize,
}

#[derive(Debug, Default)]
pub(super) struct FlowElementMetadataCounts {
    pub(super) element: usize,
    pub(super) auditing: usize,
    pub(super) monitoring: usize,
    pub(super) category_value_ref: usize,
}
