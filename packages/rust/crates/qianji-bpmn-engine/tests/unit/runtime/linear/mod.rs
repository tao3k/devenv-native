use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnInstanceState, DmnDecisionRef, PendingHostWork, PendingHostWorkKind};

mod completion;
mod dmn;
mod host;

fn assert_single_pending_host_work(
    instance: &BpmnInstanceState,
    work_kind: PendingHostWorkKind,
    decision: Option<DmnDecisionRef>,
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
            work_id: None,
        }
    );
    pending
}
