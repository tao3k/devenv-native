use super::StubHost;
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnGatewayKind,
    BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, InstanceLifecycle,
    NodeRuntimeStatus, ProcessKey, advance_instance, create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_terminate_end_cancels_parallel_host_work_and_completes() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_service_and_terminate_process()],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "terminate_parallel",
        BpmnInstanceInit::new("wf_terminate_parallel", json!({ "amount": 9 }), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(42))
        .await
        .must("terminate end should complete the full instance");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert!(instance.active_tokens.is_empty());
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.node_states[2].status, NodeRuntimeStatus::Cancelled);
    assert_eq!(instance.node_states[3].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.variables, json!({ "amount": 9 }));
    assert_eq!(instance.updated_at_ms, 42);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_terminate_end_inside_called_process_completes_parent_scope() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![
            call_activity_parent_with_terminating_child(),
            terminating_child_process(),
        ],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "terminate_parent",
        BpmnInstanceInit::new("wf_terminate_child", json!({ "amount": 11 }), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(88))
        .await
        .must("child terminate end should complete the parent route");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "terminate_parent");
    assert!(instance.call_stack.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.node_states[2].status, NodeRuntimeStatus::Completed);
}

fn parallel_service_and_terminate_process() -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new(
            "pkg_runtime",
            "terminate_parallel",
            "digest_terminate_parallel",
        ),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "fork", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(2, "slow_service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(3, "terminate_end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("slow_service")),
            BpmnEdgeSpec::new(1, 3, Some("terminate")),
        ],
        vec![BpmnEventSpec::new(3, BpmnEventKind::Terminate)],
    )
}

fn call_activity_parent_with_terminating_child() -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", "terminate_parent", "digest_terminate_parent"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "invoke_child", BpmnNodeKind::SubProcess)
                .with_called_process("terminating_child"),
            BpmnNodeSpec::new(2, "parent_end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

fn terminating_child_process() -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new(
            "pkg_runtime",
            "terminating_child",
            "digest_terminating_child",
        ),
        vec![
            BpmnNodeSpec::new(0, "child_start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "child_terminate_end", BpmnNodeKind::EndEvent),
        ],
        vec![BpmnEdgeSpec::new(0, 1, None::<&str>)],
        vec![BpmnEventSpec::new(1, BpmnEventKind::Terminate)],
    )
}
