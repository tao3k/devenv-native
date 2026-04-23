#![cfg(feature = "sqlite")]

use super::support::*;
#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_loads_checkpoint_status_from_sqlite_store() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("status.sqlite3");
    let service = QianjiBpmnWorkflowControlService::new();

    let run_report = ok_of(
        service
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    process_id: "wait_flow".to_string(),
                    instance_id: "wf_status".to_string(),
                    initial_variables: Some(json!({ "risk": "high" })),
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::Sqlite(
                        sqlite_path.clone(),
                    )),
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "workflow control service should seed one waiting sqlite checkpoint",
    );

    assert_eq!(
        run_report.execution.outcome,
        BpmnAdvanceOutcome::WaitingExternalEvent
    );
    assert!(run_report.execution.checkpoint_saved);

    let status_report = ok_of(
        service
            .load_workflow_status(&crate::QianjiBpmnWorkflowStatusRequest {
                instance_id: "wf_status".to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::Sqlite(sqlite_path),
            })
            .await,
        "workflow control service should load checkpoint-first status",
    );

    assert_eq!(status_report.checkpoint_store.backend_name(), "sqlite");
    assert_eq!(status_report.instance.instance_id.as_ref(), "wf_status");
    assert_eq!(
        status_report.instance.process.process_id.as_ref(),
        "wait_flow"
    );
    assert!(matches!(
        status_report.instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Waiting
    ));
    assert_eq!(status_report.instance.waits.len(), 1);
    assert_eq!(status_report.instance.variables, json!({ "risk": "high" }));
    assert_eq!(
        status_report.checkpoint_sequence,
        status_report.instance.sequence
    );
}
