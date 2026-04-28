#![cfg(feature = "duckdb")]

use super::support::*;

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_event_poll_resumes_checkpointed_wait() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("event-poll-action.duckdb");
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
                    process_id: "wait_flow".to_string(),
                    instance_id: "wf_event_poll_action".to_string(),
                    initial_variables: Some(json!({ "amount": 7 })),
                    start_at_node_id: None,
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb),
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "workflow control service should seed one waiting checkpoint before event poll",
    );

    assert_eq!(
        seeded_report.execution.outcome,
        BpmnAdvanceOutcome::WaitingExternalEvent
    );

    let host = QianjiBpmnHostBridge::builder()
        .on_event_poll(|request| async move {
            assert_eq!(request.instance_id, "wf_event_poll_action");
            Ok(EventPollOutcome {
                ready: true,
                winning_wait_node_index: None,
                data: json!({ "approved": true }),
            })
        })
        .build();

    let poll_report = ok_of(
        service
            .poll_workflow_events(
                &QianjiBpmnWorkflowEventPollRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    instance_id: "wf_event_poll_action".to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                },
                &host,
            )
            .await,
        "workflow control service should poll one checkpointed wait",
    );

    assert_eq!(poll_report.execution.outcome, BpmnAdvanceOutcome::Completed);
    assert!(poll_report.execution.resumed_from_checkpoint);
    assert!(poll_report.execution.checkpoint_saved);
    assert_eq!(
        poll_report.execution.session.instance().variables,
        json!({
            "amount": 7,
            "approved": true,
        })
    );
}
