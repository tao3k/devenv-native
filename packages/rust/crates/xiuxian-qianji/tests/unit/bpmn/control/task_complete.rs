#![cfg(feature = "sqlite")]

use super::support::*;
use crate::load_bpmn_package_from_files;
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnInstanceInit, ServiceTaskOutcome, advance_instance,
    create_instance, save_checkpoint_sql,
};

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_task_complete_resolves_pending_host_work() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_service_task_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("task-complete-action.sqlite3");
    let service = QianjiBpmnWorkflowControlService::new();

    seed_pending_service_task_checkpoint(&bpmn_path, &sqlite_path).await;

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
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::Sqlite(sqlite_path),
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
    sqlite_path: &std::path::Path,
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
    ok_of(
        save_checkpoint_sql(&BpmnCheckpointEnvelope::from_state(instance), sqlite_path),
        "pending service task checkpoint should persist",
    );
}
