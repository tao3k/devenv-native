use super::super::{StubHost, start_end_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnPackage, InstanceLifecycle, advance_instance,
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
