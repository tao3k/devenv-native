#![cfg(feature = "sqlite")]

use super::support::*;
#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_resumes_checkpointed_session_from_sqlite_store() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("resume.sqlite3");
    let service = QianjiBpmnWorkflowControlService::new();

    let seeded_report = ok_of(
        service
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path: bpmn_path.clone(),
                    dmn_paths: Vec::new(),
                    process_id: "wait_flow".to_string(),
                    instance_id: "wf_resume_service".to_string(),
                    initial_variables: Some(json!({ "amount": 7 })),
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::Sqlite(
                        sqlite_path.clone(),
                    )),
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "workflow control service should seed one waiting checkpoint before resume",
    );

    assert_eq!(
        seeded_report.execution.outcome,
        BpmnAdvanceOutcome::WaitingExternalEvent
    );

    let host = QianjiBpmnHostBridge::builder()
        .on_event_poll(|request| async move {
            assert_eq!(request.instance_id, "wf_resume_service");
            Ok(EventPollOutcome {
                ready: true,
                winning_wait_node_index: None,
                data: json!({ "approved": true }),
            })
        })
        .build();

    let resumed_report = ok_of(
        service
            .resume_workflow(
                &QianjiBpmnWorkflowResumeRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    instance_id: "wf_resume_service".to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::Sqlite(sqlite_path),
                },
                &host,
            )
            .await,
        "workflow control service should resume one checkpointed wait",
    );

    assert_eq!(
        resumed_report.execution.outcome,
        BpmnAdvanceOutcome::Completed
    );
    assert!(resumed_report.execution.resumed_from_checkpoint);
    assert!(resumed_report.execution.checkpoint_saved);
    assert_eq!(
        resumed_report.execution.session.instance().variables,
        json!({
            "amount": 7,
            "approved": true,
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_resume_requires_existing_checkpoint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("resume-missing.sqlite3");
    let service = QianjiBpmnWorkflowControlService::new();

    let error = match service
        .resume_workflow(
            &QianjiBpmnWorkflowResumeRequest {
                bpmn_path,
                dmn_paths: Vec::new(),
                instance_id: "wf_resume_missing".to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::Sqlite(sqlite_path),
            },
            &QianjiBpmnHostBridge::default(),
        )
        .await
    {
        Ok(report) => panic!(
            "missing resume checkpoint should fail, got outcome {:?}",
            report.execution.outcome
        ),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        QianjiBpmnWorkflowControlError::CheckpointMissing { ref instance_id }
            if instance_id == "wf_resume_missing"
    ));
}
