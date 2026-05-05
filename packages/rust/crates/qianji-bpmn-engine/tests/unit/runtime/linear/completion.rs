use crate::runtime::{StubHost, start_end_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnExecutionTraceEventKind, BpmnInstanceInit, BpmnPackage,
    InstanceLifecycle, NodeRuntimeStatus, advance_instance, create_instance,
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
    assert_eq!(instance.node_states[0].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.sequence, 3);
    assert_eq!(instance.updated_at_ms, 42);
    assert_eq!(
        instance
            .trace
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5]
    );
    assert!(
        instance
            .trace
            .iter()
            .all(|event| event.process.process_id.as_ref() == "complete")
    );
    assert_eq!(
        instance
            .trace
            .iter()
            .map(|event| {
                (
                    event.kind.clone(),
                    event.node_index,
                    event.edge_index,
                    event.status.clone(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (
                BpmnExecutionTraceEventKind::NodeStatus,
                Some(0),
                None,
                Some(NodeRuntimeStatus::Queued),
            ),
            (
                BpmnExecutionTraceEventKind::NodeStatus,
                Some(0),
                None,
                Some(NodeRuntimeStatus::Completed),
            ),
            (
                BpmnExecutionTraceEventKind::FlowTake,
                Some(1),
                Some(0),
                None,
            ),
            (
                BpmnExecutionTraceEventKind::NodeStatus,
                Some(1),
                None,
                Some(NodeRuntimeStatus::Queued),
            ),
            (
                BpmnExecutionTraceEventKind::NodeStatus,
                Some(1),
                None,
                Some(NodeRuntimeStatus::Completed),
            ),
        ]
    );
}
