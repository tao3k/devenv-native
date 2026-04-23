use crate::bpmn_cli::deps::QianjiBpmnWorkflowCancelReport;
use crate::bpmn_cli::types::{BpmnCancelCliCommand, BpmnCliOutput};

use super::support::{
    bpmn_checkpoint_backend_label, bpmn_checkpoint_backend_selection_label, bpmn_lifecycle_label,
};

pub(crate) fn render_bpmn_cancel_output(
    command: &BpmnCancelCliCommand,
    report: &QianjiBpmnWorkflowCancelReport,
) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Cancel\n\nInstance: {}\nProcess: {}\nPackage: {}\nLifecycle at cancel: {}\nCheckpoint backend: {}\nCheckpoint status: deleted\nCheckpoint sequence: {}\nState sequence: {}\nUpdated at (unix ms): {}\nActive tokens: {}\nPending host work: {}\nWait registrations: {}\nCall stack depth: {}\n",
            command.instance_id,
            report.instance.process.process_id,
            report.instance.process.package_id,
            bpmn_lifecycle_label(&report.instance.lifecycle),
            bpmn_checkpoint_backend_label(&report.checkpoint_store),
            report.checkpoint_sequence,
            report.instance.sequence,
            report.instance.updated_at_ms,
            report.instance.active_tokens.len(),
            report.instance.pending_host_work.len(),
            report.instance.waits.len(),
            report.instance.call_stack.len(),
        ),
        exit_code: 0,
    }
}

pub(crate) fn render_bpmn_cancel_missing_output(command: &BpmnCancelCliCommand) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Cancel\n\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}
