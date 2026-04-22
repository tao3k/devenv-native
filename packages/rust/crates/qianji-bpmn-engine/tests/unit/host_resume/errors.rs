use super::support::{StubHost, blocking_process, create_blocked_instance};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnInstanceInit, BpmnNodeKind, BpmnPackage, PendingHostWorkResult,
    ServiceTaskOutcome, UserTaskOutcome, apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn host_resume_requires_pending_work() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![blocking_process("resume", &BpmnNodeKind::ServiceTask)],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "resume",
        BpmnInstanceInit::new("wf_resume", json!({}), 10),
    )
    .must("instance should be created");

    let error = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        1,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must_err("host completion without pending work should fail");

    assert_eq!(
        error,
        BpmnEngineError::MissingPendingHostWork {
            instance_id: "wf_resume".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_rejects_kind_mismatch() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![blocking_process("resume", &BpmnNodeKind::ServiceTask)],
    ));
    let mut instance =
        create_blocked_instance(Arc::clone(&package), "resume", &StubHost::new(55)).await;
    let token_id = instance.pending_host_work[0].token_id;

    let error = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must_err("host completion kind should match the pending work kind");

    assert_eq!(
        error,
        BpmnEngineError::HostResultKindMismatch {
            node_index: 1,
            expected: "service",
            actual: "user",
        }
    );
}
