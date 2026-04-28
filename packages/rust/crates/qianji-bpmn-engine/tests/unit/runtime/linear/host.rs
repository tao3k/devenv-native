use super::super::{StubHost, linear_blocking_process};
use super::{PendingHostWorkExpectation, assert_single_pending_host_work};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage,
    BpmnProcessSpec, DmnDecisionRef, InstanceLifecycle, PendingHostWorkKind, ProcessKey,
    advance_instance, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_service_task_blocks_on_host_boundary() {
    assert_host_blocking(BpmnNodeKind::ServiceTask, PendingHostWorkKind::Service).await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_send_task_blocks_on_host_boundary_with_message_metadata() {
    assert_host_blocking(BpmnNodeKind::SendTask, PendingHostWorkKind::Send).await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_script_task_blocks_on_host_boundary_with_script_metadata() {
    assert_host_blocking(BpmnNodeKind::ScriptTask, PendingHostWorkKind::Script).await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_user_task_blocks_on_host_boundary() {
    assert_host_blocking(BpmnNodeKind::UserTask, PendingHostWorkKind::User).await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_manual_task_blocks_on_host_boundary() {
    assert_host_blocking(BpmnNodeKind::ManualTask, PendingHostWorkKind::Manual).await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_blocks_on_host_boundary() {
    let decision = DmnDecisionRef::new("loan-decision");
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", "dmn_gate", "digest_dmn"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "decision", BpmnNodeKind::BusinessRuleTask)
                .with_decision(decision.clone()),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    );
    let package = Arc::new(BpmnPackage::new("pkg_runtime", vec![process]));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dmn_gate",
        BpmnInstanceInit::new("wf_dmn", json!({ "amount": 10 }), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(64))
        .await
        .must("business rule task should block at the host boundary");
    let pending = assert_single_pending_host_work(
        &instance,
        PendingHostWorkExpectation::new(PendingHostWorkKind::BusinessRule)
            .with_activity_id("decision")
            .with_decision(decision.clone()),
    );

    assert_eq!(
        outcome,
        BpmnAdvanceOutcome::BlockedOnHost(vec![pending.clone()])
    );
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.suspend_reason, None);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.sequence, 3);
    assert_eq!(instance.updated_at_ms, 64);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_repairs_stale_process_index_before_advancing() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![
            super::super::start_end_process_with_id("complete"),
            linear_blocking_process("block", BpmnNodeKind::ServiceTask),
        ],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "block",
        BpmnInstanceInit::new("wf_block", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    instance.process_index = 0;

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(55))
        .await
        .must("runtime should repair the cached process index");
    let pending = assert_single_pending_host_work(
        &instance,
        PendingHostWorkExpectation::new(PendingHostWorkKind::Service),
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::BlockedOnHost(vec![pending]));
    assert_eq!(instance.process_index, 1);
}

async fn assert_host_blocking(node_kind: BpmnNodeKind, work_kind: PendingHostWorkKind) {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![linear_blocking_process("block", node_kind)],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "block",
        BpmnInstanceInit::new("wf_block", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let outcome = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("bounded runtime should block at the host boundary");
    let (script_format, script_body, event_reference, event_name) =
        if work_kind == PendingHostWorkKind::Send {
            (
                None,
                None,
                Some("invoice_dispatched"),
                Some("InvoiceDispatched"),
            )
        } else if work_kind == PendingHostWorkKind::Script {
            (Some("feel"), Some("result = amount + tax"), None, None)
        } else {
            (None, None, None, None)
        };
    let pending = assert_single_pending_host_work(
        &instance,
        PendingHostWorkExpectation::new(work_kind.clone())
            .with_script(script_format, script_body)
            .with_event(event_reference, event_name),
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::BlockedOnHost(vec![pending]));
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(
        instance.node_states[0].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );
    assert_eq!(instance.sequence, 3);
    assert_eq!(instance.updated_at_ms, 55);

    let replay = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("blocked host work should be replayable");
    assert_eq!(
        replay,
        BpmnAdvanceOutcome::BlockedOnHost(instance.pending_host_work.clone())
    );
    assert_eq!(instance.sequence, 3);
}
