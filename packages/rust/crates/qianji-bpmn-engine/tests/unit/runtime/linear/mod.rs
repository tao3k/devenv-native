use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnInstanceState, DmnDecisionRef, PendingHostWork, PendingHostWorkKind};

mod completion;
mod dmn;
mod host;
mod message;

fn assert_single_pending_host_work(
    instance: &BpmnInstanceState,
    work_kind: PendingHostWorkKind,
    decision: Option<DmnDecisionRef>,
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
            node_index: 1,
            kind: work_kind,
            decision,
            event_reference: event_reference.map(str::to_string),
            event_name: event_name.map(str::to_string),
            work_id: None,
        }
    );
    pending
}
