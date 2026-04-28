use std::fmt::Write as _;

use crate::bpmn_cli::deps::BpmnHumanTaskLifecycleEvent;

use super::labels::{
    bpmn_human_task_lifecycle_event_kind_label, bpmn_pending_host_work_kind_label,
};

pub(in crate::bpmn_cli::render) fn append_bpmn_human_task_lifecycle_event_summary(
    rendered: &mut String,
    events: &[BpmnHumanTaskLifecycleEvent],
) {
    let _ = writeln!(rendered, "Human task lifecycle events: {}", events.len());
    let Some(event) = events.last() else {
        return;
    };

    let _ = write!(
        rendered,
        "Last human task event: {} | sequence={} | token#{} | process={} | activity={} | kind={} | occurred_at_ms={}",
        bpmn_human_task_lifecycle_event_kind_label(&event.kind),
        event.sequence,
        event.token_id,
        event.process_id,
        event.activity_id,
        bpmn_pending_host_work_kind_label(&event.work_kind),
        event.occurred_at_ms,
    );
    if let Some(claimant) = event.claimant.as_deref() {
        let _ = write!(rendered, " | claimant={claimant}");
    }
    if let Some(work_id) = event.work_id.as_deref() {
        let _ = write!(rendered, " | work_id={work_id}");
    }
    let _ = writeln!(rendered);
}
