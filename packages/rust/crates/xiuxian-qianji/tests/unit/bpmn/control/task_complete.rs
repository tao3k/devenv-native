#![cfg(feature = "duckdb")]

use super::support::*;
use crate::{QianjiBpmnCheckpointStore, load_bpmn_package_from_files};
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnInstanceInit, ServiceTaskOutcome, advance_instance, create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_resolves_pending_host_work() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_service_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("task-complete-action.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    seed_pending_service_task_checkpoint(&bpmn_path, &duckdb_path).await;

    let host = QianjiBpmnHostBridge::builder()
        .on_service_task(|request| async move {
            assert_eq!(request.instance_id, "wf_task_complete_action");
            Ok(ServiceTaskOutcome {
                data: json!({
                    "approved": true,
                    "source": "workflow_control_task_complete",
                }),
            })
        })
        .build();

    let complete_report = ok_of(
        service
            .complete_workflow_task(
                &QianjiBpmnWorkflowTaskCompleteRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    instance_id: "wf_task_complete_action".to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                },
                &host,
            )
            .await,
        "workflow control service should complete one pending host task",
    );

    assert_eq!(
        complete_report.execution.outcome,
        BpmnAdvanceOutcome::Completed
    );
    assert!(complete_report.execution.resumed_from_checkpoint);
    assert!(complete_report.execution.checkpoint_saved);
    assert_eq!(
        complete_report.execution.session.instance().variables,
        json!({
            "risk": "high",
            "approved": true,
            "source": "workflow_control_task_complete",
        })
    );
}

async fn seed_pending_service_task_checkpoint(
    bpmn_path: &std::path::Path,
    duckdb_path: &std::path::Path,
) {
    let package = ok_of(
        load_bpmn_package_from_files(bpmn_path, &[]),
        "service task package should load for checkpoint seed",
    );
    let mut instance = ok_of(
        create_instance(
            package.clone(),
            "review",
            BpmnInstanceInit::new("wf_task_complete_action", json!({ "risk": "high" }), 10),
        ),
        "service task instance should seed",
    );
    let outcome = ok_of(
        advance_instance(
            package.as_ref(),
            &mut instance,
            &QianjiBpmnHostBridge::default(),
        )
        .await,
        "service task instance should block on pending host work",
    );

    assert!(matches!(outcome, BpmnAdvanceOutcome::BlockedOnHost(pending) if pending.len() == 1));
    assert_eq!(instance.pending_host_work.len(), 1);
    let store = QianjiBpmnCheckpointStore::duckdb(duckdb_path);
    ok_of(
        store
            .save(&BpmnCheckpointEnvelope::from_state(instance))
            .await,
        "pending service task checkpoint should persist",
    );
}
