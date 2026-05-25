use crate::host_resume::errors::support::{
    create_blocked_strict_instance, host_task_kinds, host_task_process, result_for_kind,
    service_process,
};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnEngineError, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, PendingHostWorkResult,
    ServiceTaskOutcome,
};

#[tokio::test(flavor = "current_thread")]
async fn host_resume_requires_declared_output_mapping() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![service_process(
            "missing_output_mapping",
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ServiceTask),
        )],
    ));
    let mut instance =
        create_blocked_strict_instance(Arc::clone(&package), "missing_output_mapping").await;
    let token_id = instance.pending_host_work[0].token_id;

    let error = crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must_err("strict Data/IO completion requires a declared output mapping");

    assert_eq!(
        error,
        BpmnEngineError::MissingTaskOutputMapping {
            process_id: ("missing_output_mapping".to_string()).into(),
            activity_id: ("task".to_string()).into(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_requires_declared_output_mapping_for_all_host_task_kinds() {
    for (node_kind, label) in host_task_kinds() {
        let process_id = format!("missing_output_mapping_{label}");
        let package = Arc::new(BpmnPackage::new(
            "pkg_resume",
            vec![host_task_process(&process_id, &node_kind, None)],
        ));
        let mut instance = create_blocked_strict_instance(Arc::clone(&package), &process_id).await;
        let token_id = instance.pending_host_work[0].token_id;

        let error = crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            token_id,
            result_for_kind(&node_kind, json!({ "value": label })),
            100,
        )
        .must_err("strict Data/IO completion requires declared output mappings");

        assert_eq!(
            error,
            BpmnEngineError::MissingTaskOutputMapping {
                process_id: process_id.into(),
                activity_id: ("task".to_string()).into(),
            }
        );
    }
}
