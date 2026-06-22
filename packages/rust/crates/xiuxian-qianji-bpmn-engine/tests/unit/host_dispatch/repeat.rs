use super::support::{
    StubHost, blocking_parallel_multi_instance_data_binding_process,
    blocking_parallel_multi_instance_process,
    blocking_sequential_multi_instance_data_binding_process,
    blocking_sequential_multi_instance_process, with_token_id,
};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnPackage, ParallelMultiInstanceContext,
    PendingHostWorkRequest, RepeatExecutionContext, SequentialMultiInstanceContext,
    ServiceTaskRequest, advance_instance, build_pending_host_work_request,
    build_pending_host_work_requests, create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_sequential_multi_instance_request_includes_repeat_context() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![blocking_sequential_multi_instance_process(
            "dispatch_multi_instance",
            3,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch_multi_instance",
        BpmnInstanceInit::new("wf_dispatch_multi_instance", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("initial advance should block on host work");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let request =
        build_pending_host_work_request(&instance).must("blocked instance should emit request");
    assert_eq!(
        request,
        with_token_id(
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: ("wf_dispatch_multi_instance".to_string()),
                token_id: 0,
                node_index: 1,
                variables: json!({ "amount": 7 }),
                inputs: json!({}),
                output_bindings: vec![],
                repeat: Some(RepeatExecutionContext::SequentialMultiInstance(
                    SequentialMultiInstanceContext {
                        iteration_index: 0,
                        total_iterations: 3,
                    },
                )),
            }),
            instance.pending_host_work[0].token_id,
        )
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_sequential_multi_instance_data_binding_overlays_input_item() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![blocking_sequential_multi_instance_data_binding_process(
            "dispatch_multi_instance_data_binding",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch_multi_instance_data_binding",
        BpmnInstanceInit::new(
            "wf_dispatch_multi_instance_data_binding",
            json!({ "items": [2, 4, 6] }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(57);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("initial advance should block on data-bound host work");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let request =
        build_pending_host_work_request(&instance).must("blocked instance should emit request");
    assert_eq!(
        request,
        with_token_id(
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: ("wf_dispatch_multi_instance_data_binding".to_string()),
                token_id: 0,
                node_index: 1,
                variables: json!({
                    "items": [2, 4, 6],
                    "item": 2,
                }),
                inputs: json!({}),
                output_bindings: vec![],
                repeat: Some(RepeatExecutionContext::SequentialMultiInstance(
                    SequentialMultiInstanceContext {
                        iteration_index: 0,
                        total_iterations: 3,
                    },
                )),
            }),
            instance.pending_host_work[0].token_id,
        )
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_parallel_multi_instance_requests_include_repeat_context() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![blocking_parallel_multi_instance_process(
            "dispatch_parallel_multi_instance",
            3,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch_parallel_multi_instance",
        BpmnInstanceInit::new(
            "wf_dispatch_parallel_multi_instance",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(56);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("initial advance should block on parallel multi-instance host work");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };
    assert_eq!(pending.len(), 3);

    assert_eq!(
        build_pending_host_work_requests(&instance)
            .must("parallel multi-instance blocked instance should emit requests"),
        vec![
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: ("wf_dispatch_parallel_multi_instance".to_string()),
                token_id: (pending[0].token_id),
                node_index: 1,
                variables: json!({ "amount": 7 }),
                inputs: json!({}),
                output_bindings: vec![],
                repeat: Some(RepeatExecutionContext::ParallelMultiInstance(
                    ParallelMultiInstanceContext {
                        iteration_index: 0,
                        total_iterations: 3,
                    },
                )),
            }),
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: ("wf_dispatch_parallel_multi_instance".to_string()),
                token_id: (pending[1].token_id),
                node_index: 1,
                variables: json!({ "amount": 7 }),
                inputs: json!({}),
                output_bindings: vec![],
                repeat: Some(RepeatExecutionContext::ParallelMultiInstance(
                    ParallelMultiInstanceContext {
                        iteration_index: 1,
                        total_iterations: 3,
                    },
                )),
            }),
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: ("wf_dispatch_parallel_multi_instance".to_string()),
                token_id: (pending[2].token_id),
                node_index: 1,
                variables: json!({ "amount": 7 }),
                inputs: json!({}),
                output_bindings: vec![],
                repeat: Some(RepeatExecutionContext::ParallelMultiInstance(
                    ParallelMultiInstanceContext {
                        iteration_index: 2,
                        total_iterations: 3,
                    },
                )),
            }),
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_parallel_multi_instance_data_binding_overlays_iteration_items() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![blocking_parallel_multi_instance_data_binding_process(
            "dispatch_parallel_multi_instance_data_binding",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch_parallel_multi_instance_data_binding",
        BpmnInstanceInit::new(
            "wf_dispatch_parallel_multi_instance_data_binding",
            json!({ "items": ["alpha", "beta"] }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(58);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("initial advance should block on parallel data-bound host work");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };

    assert_eq!(
        build_pending_host_work_requests(&instance)
            .must("parallel data-bound instance should emit requests"),
        vec![
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: ("wf_dispatch_parallel_multi_instance_data_binding".to_string()),
                token_id: (pending[0].token_id),
                node_index: 1,
                variables: json!({
                    "items": ["alpha", "beta"],
                    "item": "alpha",
                }),
                inputs: json!({}),
                output_bindings: vec![],
                repeat: Some(RepeatExecutionContext::ParallelMultiInstance(
                    ParallelMultiInstanceContext {
                        iteration_index: 0,
                        total_iterations: 2,
                    },
                )),
            }),
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: ("wf_dispatch_parallel_multi_instance_data_binding".to_string()),
                token_id: (pending[1].token_id),
                node_index: 1,
                variables: json!({
                    "items": ["alpha", "beta"],
                    "item": "beta",
                }),
                inputs: json!({}),
                output_bindings: vec![],
                repeat: Some(RepeatExecutionContext::ParallelMultiInstance(
                    ParallelMultiInstanceContext {
                        iteration_index: 1,
                        total_iterations: 2,
                    },
                )),
            }),
        ]
    );
}
