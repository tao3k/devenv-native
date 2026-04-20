use super::{
    StubHost, dmn_fixture_definition, linear_blocking_process, start_end_process,
    start_end_process_with_id,
};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnNodeKind, BpmnPackage, DmnDecisionRef,
    InstanceLifecycle, PendingHostWork, PendingHostWorkKind, ProcessKey, advance_instance,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_start_end_path_completes_deterministically() {
    let package = Arc::new(BpmnPackage::new("pkg_runtime", vec![start_end_process()]));
    let mut instance = create_instance(
        Arc::clone(&package),
        "complete",
        BpmnInstanceInit::new("wf_complete", json!({}), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(42))
        .await
        .must("bounded runtime should complete");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.node_states[0].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.sequence, 3);
    assert_eq!(instance.updated_at_ms, 42);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_service_task_blocks_on_host_boundary() {
    assert_host_blocking(BpmnNodeKind::ServiceTask, PendingHostWorkKind::Service).await;
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
    let process = qianji_bpmn_engine::BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", "dmn_gate", "digest_dmn"),
        vec![
            qianji_bpmn_engine::BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            qianji_bpmn_engine::BpmnNodeSpec::new(1, "decision", BpmnNodeKind::BusinessRuleTask)
                .with_decision(decision.clone()),
            qianji_bpmn_engine::BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            qianji_bpmn_engine::BpmnEdgeSpec::new(0, 1, None::<&str>),
            qianji_bpmn_engine::BpmnEdgeSpec::new(1, 2, None::<&str>),
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
        PendingHostWorkKind::BusinessRule,
        Some(decision.clone()),
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
async fn runtime_business_rule_task_evaluates_registered_dmn_decision_locally() {
    let decision =
        DmnDecisionRef::new("loan-decision").with_source_id("simple-unique-eligibility.dmn");
    let process = qianji_bpmn_engine::BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", "dmn_local", "digest_dmn_local"),
        vec![
            qianji_bpmn_engine::BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            qianji_bpmn_engine::BpmnNodeSpec::new(1, "decision", BpmnNodeKind::BusinessRuleTask)
                .with_decision(decision),
            qianji_bpmn_engine::BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            qianji_bpmn_engine::BpmnEdgeSpec::new(0, 1, None::<&str>),
            qianji_bpmn_engine::BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    );
    let package = Arc::new(
        BpmnPackage::new("pkg_runtime", vec![process]).with_dmn_decisions(vec![
            dmn_fixture_definition("simple-unique-eligibility.dmn"),
        ]),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "dmn_local",
        BpmnInstanceInit::new("wf_dmn_local", json!({ "tier": "gold" }), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(70))
        .await
        .must("business rule task should execute locally when the decision is registered");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(
        instance.variables,
        json!({ "tier": "gold", "approval": "approve" })
    );
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.sequence, 4);
    assert_eq!(instance.updated_at_ms, 70);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_repairs_stale_process_index_before_advancing() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![
            start_end_process_with_id("complete"),
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
    let pending = assert_single_pending_host_work(&instance, PendingHostWorkKind::Service, None);

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
    let pending = assert_single_pending_host_work(&instance, work_kind.clone(), None);

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

fn assert_single_pending_host_work(
    instance: &qianji_bpmn_engine::BpmnInstanceState,
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
