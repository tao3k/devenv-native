mod labels;
mod waits;

pub(super) use labels::{
    bpmn_checkpoint_backend_label, bpmn_checkpoint_backend_selection_label, bpmn_event_kind_label,
    bpmn_human_task_assignment_label, bpmn_human_task_form_label, bpmn_lifecycle_label,
    bpmn_node_id_label, bpmn_node_kind_label, bpmn_outcome_label,
    bpmn_pending_host_work_kind_label, bpmn_suspend_reason_label, bpmn_timer_spec_label,
    bpmn_wait_kind_label, node_runtime_status_label,
};
pub(super) use waits::append_bpmn_wait_registrations;
