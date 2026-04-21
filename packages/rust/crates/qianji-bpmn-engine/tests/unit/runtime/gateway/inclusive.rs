use super::super::{
    StubHost, inclusive_branch_process, inclusive_host_block_process,
    inclusive_numeric_branch_process,
};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnPackage, PendingHostWorkKind, PendingHostWorkResult,
    ServiceTaskOutcome, advance_instance, apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_inclusive_gateway_fans_out_matching_branches_and_joins() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![inclusive_branch_process("inclusive_branch_parallel")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "inclusive_branch_parallel",
        BpmnInstanceInit::new(
            "wf_inclusive_parallel",
            json!({ "approved": true, "vip": true }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(71))
        .await
        .must("structured inclusive gateway should complete");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
    assert!(instance.active_tokens.is_empty());
    assert!(instance.joins.is_empty());
    assert_eq!(
        instance.node_states[5].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[6].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_inclusive_gateway_uses_default_when_no_conditions_match() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![inclusive_branch_process("inclusive_branch_default")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "inclusive_branch_default",
        BpmnInstanceInit::new(
            "wf_inclusive_default",
            json!({ "approved": false, "vip": false }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(72))
        .await
        .must("default branch should complete");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.node_states[4].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_inclusive_gateway_waits_for_blocked_selected_branch_before_join() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![inclusive_host_block_process("inclusive_host_block")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "inclusive_host_block",
        BpmnInstanceInit::new(
            "wf_inclusive_host_block",
            json!({ "approved": true, "vip": true }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(88);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("inclusive split should keep the join waiting while service work blocks");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].node_index, 2);
    assert_eq!(pending[0].kind, PendingHostWorkKind::Service);
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Waiting
    );
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(instance.joins.len(), 1);
    assert_eq!(instance.joins[0].node_index, 4);
    assert_eq!(instance.joins[0].activation_id, Some(pending[0].token_id));
    assert_eq!(instance.joins[0].arrived, 1);
    assert_eq!(instance.joins[0].expected, 2);

    let resume = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        pending[0].token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome { data: json!({}) }),
        99,
    )
    .must("host completion should be applied");
    assert_eq!(resume, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 4);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("join should activate after the blocked branch resumes");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
    assert!(instance.active_tokens.is_empty());
    assert!(instance.joins.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_inclusive_gateway_numeric_conditions_select_matching_branches() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![inclusive_numeric_branch_process("inclusive_branch_numeric")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "inclusive_branch_numeric",
        BpmnInstanceInit::new(
            "wf_inclusive_numeric",
            json!({ "amount": 50, "risk": 8 }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(72))
        .await
        .must("inclusive branching should select the matching numeric branch");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[3].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[4].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
}
