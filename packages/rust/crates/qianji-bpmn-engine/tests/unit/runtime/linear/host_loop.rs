use crate::runtime::{StubHost, runtime_optional_output_io};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnGatewayKind,
    BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, InstanceLifecycle,
    NodeRuntimeStatus, PendingHostWorkKind, PendingHostWorkResult, ProcessKey, UserTaskOutcome,
    WaitKind, advance_instance, apply_event_poll_outcome, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_human_interaction_loop_advances_from_engine_work_to_human_wait_and_completion() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![human_interaction_loop_process()],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "human_interaction_loop",
        BpmnInstanceInit::new("wf_human_interaction_loop", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("runtime should stop at the user task host boundary");
    let BpmnAdvanceOutcome::BlockedOnHost(pending) = blocked else {
        panic!("expected blocked human work, got {blocked:?}");
    };

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].kind, PendingHostWorkKind::User);
    assert_eq!(
        pending[0].process_id.as_deref(),
        Some("human_interaction_loop")
    );
    assert_eq!(pending[0].activity_id.as_deref(), Some("collect_answer"));
    assert_eq!(pending[0].node_index, 2);
    assert_eq!(instance.pending_host_work, pending);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.node_states[0].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.node_states[2].status, NodeRuntimeStatus::Executing);
    assert_eq!(instance.active_tokens[0].node_index, 2);

    let token_id = pending[0].token_id;
    let resumed = crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "answer": "approve" }),
        }),
        100,
    )
    .must("typed user completion should resume the workflow");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens[0].node_index, 3);
    assert_eq!(instance.node_states[2].status, NodeRuntimeStatus::Completed);
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "answer": "approve" })
    );

    let waiting = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("runtime should advance from the human task to the next wait");
    assert_eq!(waiting, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 3);
    assert_eq!(instance.waits[0].kind, WaitKind::ExternalEvent);
    assert_eq!(instance.waits[0].event_kind, Some(BpmnEventKind::Message));
    assert_eq!(
        instance.waits[0].event_reference.as_deref(),
        Some("operator_acknowledged")
    );

    let event_resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        qianji_bpmn_engine::EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "acknowledged": true }),
        },
        120,
    )
    .must("ready external event should resume the wait");
    assert_eq!(event_resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.active_tokens[0].node_index, 4);
    assert_eq!(instance.node_states[3].status, NodeRuntimeStatus::Completed);

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("runtime should complete after the resumed wait");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(instance.node_states[4].status, NodeRuntimeStatus::Completed);
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "answer": "approve", "acknowledged": true })
    );
}

fn human_interaction_loop_process() -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new(
            "pkg_runtime",
            "human_interaction_loop",
            "digest_human_interaction_loop",
        ),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "route_to_human", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Exclusive),
            BpmnNodeSpec::new(2, "collect_answer", BpmnNodeKind::UserTask)
                .with_task_io(runtime_optional_output_io()),
            BpmnNodeSpec::new(3, "wait_ack", BpmnNodeKind::IntermediateCatchEvent),
            BpmnNodeSpec::new(4, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
            BpmnEdgeSpec::new(2, 3, None::<&str>),
            BpmnEdgeSpec::new(3, 4, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(3, BpmnEventKind::Message)
                .with_reference_id("operator_acknowledged")
                .with_name("OperatorAcknowledged"),
        ],
    )
}
