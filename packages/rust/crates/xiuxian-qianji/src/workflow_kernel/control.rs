//! Control-plane event mapping for workflow-kernel traces.

use serde_json::json;
use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlEventRecord, ControlResult, RunId, StepId,
};

use super::{WorkflowStageStatus, WorkflowStageTrace, WorkflowTrace};

const WORKFLOW_KERNEL_SOURCE: &str = "xiuxian_qianji.workflow_kernel";
const WORKFLOW_STAGE_TOOL_NAME: &str = "workflow_kernel_stage";

/// Maps a workflow trace into generic Qianji control-plane events.
///
/// # Errors
///
/// Returns a control error when the workflow id or any stage id is blank.
pub fn workflow_trace_to_control_events(trace: &WorkflowTrace) -> ControlResult<Vec<ControlEvent>> {
    let run_id = RunId::new(trace.workflow_id.clone())?;
    let started_at_ms = trace
        .stages
        .first()
        .map_or(0, |stage| stage.started_unix_ms);
    let mut events = vec![
        ControlEvent::run(
            run_id.clone(),
            started_at_ms,
            ControlEventKind::RunCreated {
                intent: format!("workflow:{}", trace.workflow_id),
                budget: None,
                metadata: json!({
                    "source": WORKFLOW_KERNEL_SOURCE,
                    "stageCount": trace.stages.len(),
                }),
            },
        ),
        ControlEvent::run(run_id.clone(), started_at_ms, ControlEventKind::RunAdmitted),
        ControlEvent::run(
            run_id.clone(),
            started_at_ms,
            ControlEventKind::PlanRecorded {
                summary: format!("Workflow trace with {} stage(s)", trace.stages.len()),
            },
        ),
    ];

    for stage in &trace.stages {
        append_stage_events(&mut events, &run_id, stage)?;
    }

    let terminal_at_ms = trace
        .stages
        .last()
        .map_or(started_at_ms, stage_terminal_at_ms);
    if let Some(failed_stage) = trace
        .stages
        .iter()
        .find(|stage| stage.status == WorkflowStageStatus::Failed)
    {
        events.push(ControlEvent::run(
            run_id,
            terminal_at_ms,
            ControlEventKind::RunFailed {
                message: failed_stage
                    .error
                    .clone()
                    .unwrap_or_else(|| "workflow stage failed".to_owned()),
            },
        ));
    } else {
        events.push(ControlEvent::run(
            run_id,
            terminal_at_ms,
            ControlEventKind::RunCompleted,
        ));
    }

    Ok(events)
}

/// Maps a workflow trace into sequence-numbered control records for immediate replay.
///
/// # Errors
///
/// Returns a control error when event mapping fails.
pub fn workflow_trace_to_control_event_records(
    trace: &WorkflowTrace,
) -> ControlResult<Vec<ControlEventRecord>> {
    workflow_trace_to_control_events(trace).map(|events| {
        events
            .into_iter()
            .enumerate()
            .map(|(index, event)| ControlEventRecord {
                sequence: u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1),
                event,
            })
            .collect()
    })
}

fn append_stage_events(
    events: &mut Vec<ControlEvent>,
    run_id: &RunId,
    stage: &WorkflowStageTrace,
) -> ControlResult<()> {
    let step_id = StepId::new(stage.stage_id.clone())?;
    let terminal_at_ms = stage_terminal_at_ms(stage);
    events.push(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        stage.started_unix_ms,
        ControlEventKind::StepCreated {
            title: stage.stage_id.clone(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ));
    events.push(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        stage.started_unix_ms,
        ControlEventKind::StepStarted,
    ));
    events.push(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        terminal_at_ms,
        ControlEventKind::ToolCallRecorded {
            tool_name: WORKFLOW_STAGE_TOOL_NAME.to_owned(),
            metadata: stage_metadata(stage),
        },
    ));
    match stage.status {
        WorkflowStageStatus::Succeeded => events.push(ControlEvent::step(
            run_id.clone(),
            step_id,
            terminal_at_ms,
            ControlEventKind::StepSucceeded,
        )),
        WorkflowStageStatus::Failed => events.push(ControlEvent::step(
            run_id.clone(),
            step_id,
            terminal_at_ms,
            ControlEventKind::StepFailed {
                error_code: "workflow_stage_failed".to_owned(),
                message: stage
                    .error
                    .clone()
                    .unwrap_or_else(|| "workflow stage failed".to_owned()),
                retryable: false,
            },
        )),
    }
    Ok(())
}

fn stage_metadata(stage: &WorkflowStageTrace) -> serde_json::Value {
    json!({
        "source": WORKFLOW_KERNEL_SOURCE,
        "stageId": stage.stage_id,
        "status": stage.status,
        "durationNanos": stage.duration_nanos,
        "input": stage.input,
        "output": stage.output,
        "checkpoints": stage.checkpoints,
    })
}

fn stage_terminal_at_ms(stage: &WorkflowStageTrace) -> u64 {
    stage
        .started_unix_ms
        .saturating_add(stage.duration_nanos / 1_000_000)
}
