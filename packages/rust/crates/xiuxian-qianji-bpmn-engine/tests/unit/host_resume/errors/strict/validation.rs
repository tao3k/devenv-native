use crate::host_resume::errors::support::{
    create_blocked_strict_instance, host_task_kinds, host_task_process, result_for_kind,
    service_process,
};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnEngineError, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnTaskIoSpec,
    BpmnTaskOutputBinding, PendingHostWorkResult, ServiceTaskOutcome,
};

#[tokio::test(flavor = "current_thread")]
async fn host_resume_rejects_missing_required_output() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![service_process(
            "missing_required_output",
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ServiceTask).with_task_io(
                BpmnTaskIoSpec::new()
                    .with_output(BpmnTaskOutputBinding::new("approved", "approved")),
            ),
        )],
    ));
    let mut instance =
        create_blocked_strict_instance(Arc::clone(&package), "missing_required_output").await;
    let token_id = instance.pending_host_work[0].token_id;

    let error = crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome { data: json!({}) }),
        100,
    )
    .must_err("strict Data/IO completion requires every required output");

    assert_eq!(
        error,
        BpmnEngineError::MissingTaskCompletionField {
            process_id: ("missing_required_output".to_string()).into(),
            activity_id: ("task".to_string()).into(),
            field: "approved".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_rejects_non_object_completion_data_for_all_host_task_kinds() {
    for (node_kind, label) in host_task_kinds() {
        let process_id = format!("non_object_output_{label}");
        let package = Arc::new(BpmnPackage::new(
            "pkg_resume",
            vec![host_task_process(
                &process_id,
                &node_kind,
                Some(
                    BpmnTaskIoSpec::new()
                        .with_output(BpmnTaskOutputBinding::new("value", "result")),
                ),
            )],
        ));
        let mut instance = create_blocked_strict_instance(Arc::clone(&package), &process_id).await;
        let token_id = instance.pending_host_work[0].token_id;

        let error = crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            token_id,
            result_for_kind(&node_kind, json!(label)),
            100,
        )
        .must_err("strict Data/IO completion requires object data");

        assert_eq!(
            error,
            BpmnEngineError::TaskCompletionDataNotObject {
                process_id: process_id.into(),
                activity_id: ("task".to_string()).into(),
            }
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_rejects_undeclared_output() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![service_process(
            "undeclared_output",
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ServiceTask).with_task_io(
                BpmnTaskIoSpec::new()
                    .with_output(BpmnTaskOutputBinding::new("approved", "approved")),
            ),
        )],
    ));
    let mut instance =
        create_blocked_strict_instance(Arc::clone(&package), "undeclared_output").await;
    let token_id = instance.pending_host_work[0].token_id;

    let error = crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "approved": true, "extra": true }),
        }),
        100,
    )
    .must_err("strict Data/IO completion rejects undeclared output fields");

    assert_eq!(
        error,
        BpmnEngineError::UndeclaredTaskCompletionField {
            process_id: ("undeclared_output".to_string()).into(),
            activity_id: ("task".to_string()).into(),
            field: "extra".to_string(),
        }
    );
}
