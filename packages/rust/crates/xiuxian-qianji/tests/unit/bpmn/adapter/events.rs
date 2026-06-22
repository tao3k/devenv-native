use serde_json::json;
use xiuxian_qianji_bpmn_engine::{BpmnAdvanceOutcome, EventPollOutcome, InstanceLifecycle};

use super::support::{ok_of, waiting_instance};
use crate::{QianjiBpmnHostBridge, resolve_waiting_external_event};

#[tokio::test(flavor = "current_thread")]
async fn resolve_waiting_external_event_preserves_waiting_when_poll_is_unsupported() {
    let (package, mut instance) = waiting_instance();
    let host = QianjiBpmnHostBridge::default();

    let outcome = ok_of(
        resolve_waiting_external_event(package.as_ref(), &mut instance, &host).await,
        "unsupported event polling should preserve the waiting state",
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 1);
    assert_eq!(instance.variables, json!({ "amount": 7 }));
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_waiting_external_event_applies_ready_outcome_through_bridge() {
    let (package, mut instance) = waiting_instance();
    let host = QianjiBpmnHostBridge::builder()
        .on_event_poll(|_request| async move {
            Ok(EventPollOutcome {
                ready: true,
                winning_wait_node_index: None,
                data: json!({ "approved": true }),
            })
        })
        .clock(|| 144)
        .build();

    let resumed = ok_of(
        resolve_waiting_external_event(package.as_ref(), &mut instance, &host).await,
        "ready event polling should resume the waiting instance",
    );

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(
        instance.variables,
        json!({
            "amount": 7,
            "approved": true,
        })
    );
}
