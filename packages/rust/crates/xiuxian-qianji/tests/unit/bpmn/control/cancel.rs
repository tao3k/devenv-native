use super::support::{
    BpmnAdvanceOutcome, QianjiBpmnHostBridge, QianjiBpmnWorkflowCancelRequest,
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowStartRequest, QianjiRuntimeEnv,
    SchedulerAgentIdentity, TempDir, TestValkey, json, ok_of, unique_instance_id,
    write_wait_bundle,
};

#[cfg(feature = "duckdb")]
#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_cancels_checkpointed_session_from_duckdb_store() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("cancel.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    let seeded_report = ok_of(
        service
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    process_id: "wait_flow".to_string().into(),
                    instance_id: "wf_cancel_duckdb".to_string().into(),
                    initial_variables: Some(json!({ "amount": 7 })),
                    start_at_node_id: None,
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb),
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
                instance_id: "wf_cancel_duckdb".to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
            })
            .await,
        "workflow control service should delete one duckdb checkpoint",
    );

    assert_eq!(cancel_report.checkpoint_store.backend_name(), "duckdb");
    assert_eq!(
        cancel_report.instance.instance_id.as_ref(),
        "wf_cancel_duckdb"
    );
    assert!(matches!(
        cancel_report.instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Waiting
    ));

    let checkpoint = ok_of(
        cancel_report
            .checkpoint_store
            .load("wf_cancel_duckdb")
            .await,
        "cancelled duckdb checkpoint should be deleted",
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
    let instance_id = unique_instance_id("wf_cancel_runtime_missing_agent");

    let seeded_report = ok_of(
        QianjiBpmnWorkflowControlService::new()
            .with_runtime_env(runtime_env.clone())
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    process_id: "wait_flow".to_string().into(),
                    instance_id: instance_id.clone().into(),
                    initial_variables: Some(json!({ "risk": "high" })),
                    start_at_node_id: None,
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
            instance_id: instance_id.into(),
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
    let instance_id = unique_instance_id("wf_cancel_runtime");

    let seeded_report = ok_of(
        QianjiBpmnWorkflowControlService::new()
            .with_runtime_env(runtime_env.clone())
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    process_id: "wait_flow".to_string().into(),
                    instance_id: instance_id.clone().into(),
                    initial_variables: Some(json!({ "risk": "high" })),
                    start_at_node_id: None,
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
                instance_id: instance_id.clone().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey,
            })
            .await,
        "runtime valkey cancel should delete one checkpoint through lease ownership",
    );

    assert_eq!(cancel_report.checkpoint_store.backend_name(), "valkey");
    assert_eq!(cancel_report.instance.instance_id.as_ref(), instance_id);
    assert!(matches!(
        cancel_report.instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Waiting
    ));

    let checkpoint = ok_of(
        cancel_report
            .checkpoint_store
            .load(instance_id.as_str())
            .await,
        "cancelled runtime checkpoint should be deleted",
    );
    assert!(checkpoint.is_none());

    let reacquired = ok_of(
        cancel_report
            .checkpoint_store
            .try_acquire_lease(instance_id.as_str(), "bpmn-scheduler:worker-b", 30_000)
            .await,
        "cancelled runtime checkpoint should release the lease for reuse",
    );
    assert!(reacquired);
}
