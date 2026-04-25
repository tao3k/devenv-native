use crate::bpmn_cli::deps::{BpmnAdvanceOutcome, QianjiBpmnSession};
use crate::bpmn_cli::types::{
    BpmnCliOutput, BpmnEventPollCliCommand, BpmnExecutionRenderContext, BpmnResumeCliCommand,
    BpmnRunCliCommand, BpmnStartCliCommand, BpmnTaskCompleteCliCommand,
};

use super::body::render_bpmn_execution_output;

pub(crate) fn render_bpmn_start_output(
    command: &BpmnStartCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Start",
        command.process_id.as_str(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_run_output(
    command: &BpmnRunCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Run",
        command.process_id.as_str(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_resume_output(
    command: &BpmnResumeCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Resume",
        session.instance().process.process_id.as_ref(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_event_poll_output(
    command: &BpmnEventPollCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Event Poll",
        session.instance().process.process_id.as_ref(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_task_complete_output(
    command: &BpmnTaskCompleteCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Task Complete",
        session.instance().process.process_id.as_ref(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}
