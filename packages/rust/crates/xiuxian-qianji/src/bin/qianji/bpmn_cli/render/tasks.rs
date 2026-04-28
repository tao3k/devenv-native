use std::fmt::Write as _;

use crate::bpmn_cli::deps::{
    PendingHostWorkKind, QianjiBpmnWorkflowTaskClaimReport, QianjiBpmnWorkflowTaskReleaseReport,
    QianjiBpmnWorkflowWorklistItem, QianjiBpmnWorkflowWorklistReport,
};
use crate::bpmn_cli::types::{
    BpmnCliOutput, BpmnTaskClaimCliCommand, BpmnTaskReleaseCliCommand, BpmnTaskWorklistCliCommand,
};

use super::support::{
    append_bpmn_human_task_lifecycle_event_summary, bpmn_checkpoint_backend_label,
    bpmn_checkpoint_backend_selection_label, bpmn_human_task_assignment_label,
    bpmn_human_task_form_label, bpmn_lane_membership_label, bpmn_pending_host_work_kind_label,
};

pub(crate) fn render_bpmn_task_claim_output(
    command: &BpmnTaskClaimCliCommand,
    report: &QianjiBpmnWorkflowTaskClaimReport,
) -> BpmnCliOutput {
    let claim = report.claimed_work.claim.as_ref();
    let mut rendered = format!(
        "# BPMN Task Claim\n\nInstance: {}\nProcess: {}\nActivity: {}\nToken: {}\nKind: {}\nCheckpoint backend: {}\nCheckpoint status: loaded\nCheckpoint sequence: {}\nState sequence: {}\nChanged: {}\nClaimant: {}\nClaim status: {}\n",
        command.instance_id,
        command.process_id,
        command.activity_id,
        command.token_id,
        bpmn_pending_host_work_kind_label(&report.claimed_work.kind),
        bpmn_checkpoint_backend_label(&report.checkpoint_store),
        report.checkpoint_sequence,
        report.instance.sequence,
        yes_no(report.changed),
        command.claimant,
        if claim.is_some() {
            "claimed"
        } else {
            "unclaimed"
        },
    );
    if let Some(claim) = claim {
        let _ = writeln!(rendered, "Claimed at (unix ms): {}", claim.claimed_at_ms);
    }
    append_bpmn_human_task_lifecycle_event_summary(
        &mut rendered,
        &report.instance.human_task_events,
    );
    append_task_coordination_boundary(&mut rendered);

    BpmnCliOutput {
        rendered,
        exit_code: 0,
    }
}

pub(crate) fn render_bpmn_task_claim_missing_output(
    command: &BpmnTaskClaimCliCommand,
) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Task Claim\n\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

pub(crate) fn render_bpmn_task_release_output(
    command: &BpmnTaskReleaseCliCommand,
    report: &QianjiBpmnWorkflowTaskReleaseReport,
) -> BpmnCliOutput {
    let mut rendered = format!(
        "# BPMN Task Release\n\nInstance: {}\nProcess: {}\nActivity: {}\nToken: {}\nKind: {}\nCheckpoint backend: {}\nCheckpoint status: loaded\nCheckpoint sequence: {}\nState sequence: {}\nChanged: {}\nClaimant: {}\nClaim status: {}\n",
        command.instance_id,
        command.process_id,
        command.activity_id,
        command.token_id,
        bpmn_pending_host_work_kind_label(&report.released_work.kind),
        bpmn_checkpoint_backend_label(&report.checkpoint_store),
        report.checkpoint_sequence,
        report.instance.sequence,
        yes_no(report.changed),
        command.claimant,
        if report.released_work.claim.is_some() {
            "claimed"
        } else {
            "unclaimed"
        },
    );
    append_bpmn_human_task_lifecycle_event_summary(
        &mut rendered,
        &report.instance.human_task_events,
    );
    append_task_coordination_boundary(&mut rendered);

    BpmnCliOutput {
        rendered,
        exit_code: 0,
    }
}

pub(crate) fn render_bpmn_task_release_missing_output(
    command: &BpmnTaskReleaseCliCommand,
) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Task Release\n\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

pub(crate) fn render_bpmn_task_worklist_output(
    command: &BpmnTaskWorklistCliCommand,
    report: &QianjiBpmnWorkflowWorklistReport,
) -> BpmnCliOutput {
    let mut rendered = format!(
        "# BPMN Task Worklist\n\nCheckpoint backend: {}\nClaimant filter: {}\nAssignment resource filter: {}\nLane filter: {}\nItem count: {}\n",
        bpmn_checkpoint_backend_label(&report.checkpoint_store),
        command.claimant.as_deref().unwrap_or("none"),
        command.assignment_resource.as_deref().unwrap_or("none"),
        command.lane.as_deref().unwrap_or("none"),
        report.work_items.len(),
    );
    append_task_coordination_boundary(&mut rendered);

    if !report.work_items.is_empty() {
        let _ = writeln!(rendered, "\n## Human Work");
        for item in &report.work_items {
            append_worklist_item(&mut rendered, item);
        }
    }

    BpmnCliOutput {
        rendered,
        exit_code: 0,
    }
}

fn append_worklist_item(rendered: &mut String, item: &QianjiBpmnWorkflowWorklistItem) {
    let _ = write!(
        rendered,
        "- {} | token#{} | process={} | activity={} | kind={} | checkpoint_sequence={} | state_sequence={} | updated_at_ms={}",
        item.instance_id,
        item.token_id,
        item.process_id,
        item.activity_id,
        human_task_kind_label(&item.kind),
        item.checkpoint_sequence,
        item.state_sequence,
        item.updated_at_ms,
    );
    if let Some(claim) = item.claim.as_ref() {
        let _ = write!(
            rendered,
            " | claim={} | claimed_at_ms={}",
            claim.claimant, claim.claimed_at_ms
        );
    } else {
        let _ = write!(rendered, " | claim=unclaimed");
    }
    if let Some(form) = item.form.as_ref() {
        let _ = write!(rendered, " | form={}", bpmn_human_task_form_label(form));
    }
    if let Some(assignment) = item.assignment.as_ref() {
        let label = bpmn_human_task_assignment_label(assignment);
        if !label.is_empty() {
            let _ = write!(rendered, " | assignment={label}");
        }
    }
    if let Some(lane) = item.lane.as_ref() {
        let _ = write!(rendered, " | lane={}", bpmn_lane_membership_label(lane));
    }
    let _ = writeln!(rendered);
}

fn append_task_coordination_boundary(rendered: &mut String) {
    let _ = writeln!(
        rendered,
        "Authorization: not evaluated; BPMN assignment and lane metadata are routing-only."
    );
}

fn human_task_kind_label(kind: &PendingHostWorkKind) -> &'static str {
    bpmn_pending_host_work_kind_label(kind)
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
