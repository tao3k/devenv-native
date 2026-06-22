use crate::host_resume::errors::support::{
    create_blocked_strict_instance, host_task_kinds, host_task_process, result_for_kind,
    service_process,
};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec,
    BpmnTaskIoSpec, BpmnTaskOutputBinding, PendingHostWorkResult, ProcessKey, ServiceTaskOutcome,
    advance_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn host_resume_maps_declared_output_for_all_host_task_kinds() {
    for (node_kind, label) in host_task_kinds() {
        let process_id = format!("mapped_output_{label}");
        let target_ref = format!("results.{label}");
        let package = Arc::new(BpmnPackage::new(
            "pkg_resume",
            vec![host_task_process(
                &process_id,
                &node_kind,
                Some(
                    BpmnTaskIoSpec::new()
                        .with_output(BpmnTaskOutputBinding::new("value", &target_ref)),
                ),
            )],
        ));
        let mut instance = create_blocked_strict_instance(Arc::clone(&package), &process_id).await;
        let token_id = instance.pending_host_work[0].token_id;

        crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            token_id,
            result_for_kind(&node_kind, json!({ "value": label })),
            100,
        )
        .must("declared output should map to the target workflow variable");

        assert_eq!(instance.variables["results"][label], json!(label));
        assert!(instance.variables.get("value").is_none());
    }
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_maps_declared_output_to_target_variable() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![service_process(
            "mapped_output",
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ServiceTask).with_task_io(
                BpmnTaskIoSpec::new()
                    .with_output(BpmnTaskOutputBinding::new("approved", "review.approved")),
            ),
        )],
    ));
    let mut instance = create_blocked_strict_instance(Arc::clone(&package), "mapped_output").await;
    let token_id = instance.pending_host_work[0].token_id;

    crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must("declared output should map to the target workflow variable");

    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "review": { "approved": true } })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_deep_merges_repeated_mapped_output_targets() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![BpmnProcessSpec::new(
            ProcessKey::new(
                "pkg_resume",
                "mapped_output_chain",
                "digest_mapped_output_chain",
            ),
            vec![
                BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
                BpmnNodeSpec::new(1, "first_task", BpmnNodeKind::ServiceTask).with_task_io(
                    BpmnTaskIoSpec::new()
                        .with_output(BpmnTaskOutputBinding::new("first", "review.first")),
                ),
                BpmnNodeSpec::new(2, "second_task", BpmnNodeKind::ServiceTask).with_task_io(
                    BpmnTaskIoSpec::new()
                        .with_output(BpmnTaskOutputBinding::new("second", "review.second")),
                ),
                BpmnNodeSpec::new(3, "end", BpmnNodeKind::EndEvent),
            ],
            vec![
                BpmnEdgeSpec::new(0, 1, None::<&str>),
                BpmnEdgeSpec::new(1, 2, None::<&str>),
                BpmnEdgeSpec::new(2, 3, None::<&str>),
            ],
            Vec::new(),
        )],
    ));
    let mut instance =
        create_blocked_strict_instance(Arc::clone(&package), "mapped_output_chain").await;

    let first_token_id = instance.pending_host_work[0].token_id;
    crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        first_token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "first": true }),
        }),
        100,
    )
    .must("first declared output should map to nested target");
    let blocked = advance_instance(
        package.as_ref(),
        &mut instance,
        &crate::host_resume::support::StubHost::new(101),
    )
    .await
    .must("second service task should block");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let second_token_id = instance.pending_host_work[0].token_id;
    crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        second_token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "second": true }),
        }),
        102,
    )
    .must("second declared output should merge into nested target");

    assert_eq!(
        instance.variables,
        json!({
            "amount": 7,
            "review": {
                "first": true,
                "second": true
            }
        })
    );
}
