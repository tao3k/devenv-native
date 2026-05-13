#![cfg(feature = "duckdb")]

use super::support::{
    BpmnAdvanceOutcome, EventPollOutcome, QianjiBpmnHostBridge,
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowStartRequest, QianjiRuntimeEnv, TempDir, json, ok_of, write_wait_bundle,
};
#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_resumes_checkpointed_session_from_duckdb_store() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("resume.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    let seeded_report = ok_of(
        service
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path: bpmn_path.clone(),
                    dmn_paths: Vec::new(),
                    process_id: "wait_flow".to_string().into(),
                    instance_id: "wf_resume_service".to_string().into(),
                    initial_variables: Some(json!({ "amount": 7 })),
                    start_at_node_id: None,
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb),
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
                    instance_id: "wf_resume_service".to_string().into(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
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
    let duckdb_path = temp_dir.path().join("resume-missing.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    let error = match service
        .resume_workflow(
            &QianjiBpmnWorkflowResumeRequest {
                bpmn_path,
                dmn_paths: Vec::new(),
                instance_id: "wf_resume_missing".to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
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
