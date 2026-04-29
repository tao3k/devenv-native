use super::support::{StubHost, parallel_service_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEngineError, BpmnInstanceInit, BpmnPackage, PendingHostWorkRequest,
    ServiceTaskRequest, advance_instance, build_pending_host_work_request,
    build_pending_host_work_requests, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_parallel_host_work_requires_plural_builder() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![parallel_service_process("dispatch_parallel")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch_parallel",
        BpmnInstanceInit::new("wf_dispatch_parallel", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel split should block on both host tasks");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };

    assert_eq!(pending.len(), 2);
    assert_eq!(
        build_pending_host_work_requests(&instance)
            .must("parallel blocked instance should emit requests"),
        vec![
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: "wf_dispatch_parallel".to_string(),
                token_id: pending[0].token_id,
                node_index: 2,
                variables: json!({ "amount": 7 }),
                inputs: json!({}),
                output_bindings: vec![],
                repeat: None,
            }),
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: "wf_dispatch_parallel".to_string(),
                token_id: pending[1].token_id,
                node_index: 3,
                variables: json!({ "amount": 7 }),
                inputs: json!({}),
                output_bindings: vec![],
                repeat: None,
            }),
        ]
    );

    let error = build_pending_host_work_request(&instance)
        .must_err("singleton builder should reject multiple pending host works");
    assert_eq!(
        error,
        BpmnEngineError::AmbiguousPendingHostWork {
            instance_id: "wf_dispatch_parallel".to_string(),
            count: 2,
        }
    );
}
