use super::support::*;

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_cancels_checkpointed_session_from_sqlite_store() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("cancel.sqlite3");
    let service = QianjiBpmnWorkflowControlService::new();

    let seeded_report = ok_of(
        service
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    process_id: "wait_flow".to_string(),
                    instance_id: "wf_cancel_sqlite".to_string(),
                    initial_variables: Some(json!({ "amount": 7 })),
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::Sqlite(
                        sqlite_path.clone(),
                    )),
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "workflow control service should seed one waiting checkpoint before cancel",
    );

    assert_eq!(
        seeded_report.execution.outcome,
        BpmnAdvanceOutcome::WaitingExternalEvent
    );

    let cancel_report = ok_of(
        service
            .cancel_workflow(&QianjiBpmnWorkflowCancelRequest {
                instance_id: "wf_cancel_sqlite".to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::Sqlite(sqlite_path),
            })
            .await,
        "workflow control service should delete one sqlite checkpoint",
    );

    assert_eq!(cancel_report.checkpoint_store.backend_name(), "sqlite");
    assert_eq!(
        cancel_report.instance.instance_id.as_ref(),
        "wf_cancel_sqlite"
    );
    assert!(matches!(
        cancel_report.instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Waiting
    ));

    let checkpoint = ok_of(
        cancel_report
            .checkpoint_store
            .load("wf_cancel_sqlite")
            .await,
        "cancelled sqlite checkpoint should be deleted",
    );
    assert!(checkpoint.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_runtime_valkey_cancel_requires_stable_agent_id() {
    let valkey = ok_of(
        TestValkey::spawn().await,
        "valkey should start for workflow control service runtime cancel identity test",
    );
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let runtime_env = QianjiRuntimeEnv {
        qianji_checkpoint_valkey_url: Some(valkey.url().to_string()),
        ..QianjiRuntimeEnv::default()
    };

    let seeded_report = ok_of(
        QianjiBpmnWorkflowControlService::new()
            .with_runtime_env(runtime_env.clone())
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    process_id: "wait_flow".to_string(),
                    instance_id: "wf_cancel_runtime_missing_agent".to_string(),
                    initial_variables: Some(json!({ "risk": "high" })),
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey),
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "runtime valkey path should seed one waiting checkpoint before cancel",
    );

    assert_eq!(
        seeded_report.execution.outcome,
        BpmnAdvanceOutcome::WaitingExternalEvent
    );

    let error = match QianjiBpmnWorkflowControlService::new()
        .with_runtime_env(runtime_env)
        .with_scheduler_identity(SchedulerAgentIdentity::new(
            None,
            Some("manager".to_string()),
        ))
        .cancel_workflow(&QianjiBpmnWorkflowCancelRequest {
            instance_id: "wf_cancel_runtime_missing_agent".to_string(),
            checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey,
        })
        .await
    {
        Ok(report) => panic!(
            "runtime cancel without stable agent id should fail, got checkpoint sequence {}",
            report.checkpoint_sequence
        ),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        QianjiBpmnWorkflowControlError::Orchestration(
            crate::BpmnOrchestrationError::CheckpointLeaseAgentIdRequired
        )
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_runtime_valkey_cancel_deletes_checkpoint_and_releases_lease() {
    let valkey = ok_of(
        TestValkey::spawn().await,
        "valkey should start for workflow control service runtime cancel test",
    );
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let runtime_env = QianjiRuntimeEnv {
        qianji_checkpoint_valkey_url: Some(valkey.url().to_string()),
        ..QianjiRuntimeEnv::default()
    };

    let seeded_report = ok_of(
        QianjiBpmnWorkflowControlService::new()
            .with_runtime_env(runtime_env.clone())
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    process_id: "wait_flow".to_string(),
                    instance_id: "wf_cancel_runtime".to_string(),
                    initial_variables: Some(json!({ "risk": "high" })),
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey),
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "runtime valkey path should seed one waiting checkpoint before cancel",
    );

    assert_eq!(
        seeded_report.execution.outcome,
        BpmnAdvanceOutcome::WaitingExternalEvent
    );
    assert!(seeded_report.execution.checkpoint_saved);

    let service = QianjiBpmnWorkflowControlService::new()
        .with_runtime_env(runtime_env)
        .with_scheduler_identity(SchedulerAgentIdentity::new(
            Some("worker-a".to_string()),
            Some("manager".to_string()),
        ));
    let cancel_report = ok_of(
        service
            .cancel_workflow(&QianjiBpmnWorkflowCancelRequest {
                instance_id: "wf_cancel_runtime".to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey,
            })
            .await,
        "runtime valkey cancel should delete one checkpoint through lease ownership",
    );

    assert_eq!(cancel_report.checkpoint_store.backend_name(), "valkey");
    assert_eq!(
        cancel_report.instance.instance_id.as_ref(),
        "wf_cancel_runtime"
    );
    assert!(matches!(
        cancel_report.instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Waiting
    ));

    let checkpoint = ok_of(
        cancel_report
            .checkpoint_store
            .load("wf_cancel_runtime")
            .await,
        "cancelled runtime checkpoint should be deleted",
    );
    assert!(checkpoint.is_none());

    let reacquired = ok_of(
        cancel_report
            .checkpoint_store
            .try_acquire_lease("wf_cancel_runtime", "bpmn-scheduler:worker-b", 30_000)
            .await,
        "cancelled runtime checkpoint should release the lease for reuse",
    );
    assert!(reacquired);
}
