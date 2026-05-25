use crate::runtime::StubHost;
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnInstanceInit, BpmnNodeKind,
    BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, InstanceLifecycle, ProcessKey, advance_instance,
    create_instance,
};

fn top_level_error_end_process() -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new(
            "pkg_runtime",
            "root_error_process",
            "digest_root_error_process",
        ),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "fatal_end", BpmnNodeKind::EndEvent),
        ],
        vec![BpmnEdgeSpec::new(0, 1, None::<&str>)],
        vec![BpmnEventSpec::new(1, BpmnEventKind::Error).with_reference_id("fatal_review_error")],
    )
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_top_level_error_end_fails_instance_deterministically() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![top_level_error_end_process()],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "root_error_process",
        BpmnInstanceInit::new("wf_root_error", json!({ "approved": false }), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(42))
        .await
        .must("top-level BPMN error end should fail deterministically");

    assert_eq!(
        outcome,
        BpmnAdvanceOutcome::Failed(
            "process 'root_error_process' terminated with BPMN error end 'fatal_end' (errorRef='fatal_review_error')"
                .to_string()
        )
    );
    assert_eq!(instance.lifecycle, InstanceLifecycle::Failed);
    assert!(instance.active_tokens.is_empty());
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(
        instance.node_states[0].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[1].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Failed
    );
    assert_eq!(instance.variables, json!({ "approved": false }));
    assert_eq!(instance.updated_at_ms, 42);
}
