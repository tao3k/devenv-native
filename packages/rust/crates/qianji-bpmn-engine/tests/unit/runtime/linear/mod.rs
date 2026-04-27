use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnInstanceState, DmnDecisionRef, PendingHostWork, PendingHostWorkKind};

mod completion;
mod dmn;
mod error;
mod host;
mod message;

fn assert_single_pending_host_work(
    instance: &BpmnInstanceState,
    work_kind: PendingHostWorkKind,
    decision: Option<DmnDecisionRef>,
    script_format: Option<&str>,
    script_body: Option<&str>,
    event_reference: Option<&str>,
    event_name: Option<&str>,
) -> PendingHostWork {
    let pending = instance
        .pending_host_work
        .first()
        .cloned()
        .must("pending host work should be stored");
    assert_eq!(
        pending,
        PendingHostWork {
            token_id: instance.active_tokens[0].token_id,
            process_id: Some(instance.process.process_id.to_string()),
            node_index: 1,
            activity_id: Some("task".to_string()),
            kind: work_kind,
            decision,
            script_format: script_format.map(str::to_string),
            script_body: script_body.map(str::to_string),
            event_reference: event_reference.map(str::to_string),
            event_name: event_name.map(str::to_string),
            work_id: None,
        }
    );
    pending
}
