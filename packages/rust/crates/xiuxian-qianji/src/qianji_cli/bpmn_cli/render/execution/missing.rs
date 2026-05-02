use crate::qianji_cli::bpmn_cli::render::support::bpmn_checkpoint_backend_selection_label;
use crate::qianji_cli::bpmn_cli::types::{
    BpmnCliOutput, BpmnEventPollCliCommand, BpmnResumeCliCommand, BpmnTaskCompleteCliCommand,
};

pub(crate) fn render_bpmn_resume_missing_output(command: &BpmnResumeCliCommand) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Resume\n\nSource: {}\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.bpmn_path.display(),
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

pub(crate) fn render_bpmn_event_poll_missing_output(
    command: &BpmnEventPollCliCommand,
) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Event Poll\n\nSource: {}\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.bpmn_path.display(),
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

pub(crate) fn render_bpmn_task_complete_missing_output(
    command: &BpmnTaskCompleteCliCommand,
) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Task Complete\n\nSource: {}\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.bpmn_path.display(),
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}
