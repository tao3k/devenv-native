use super::support::*;

#[cfg(feature = "duckdb")]
#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_interrupts_checkpointed_session_from_duckdb_store() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("interrupt.duckdb");
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
                    instance_id: "wf_interrupt_duckdb".to_string(),
                    initial_variables: Some(json!({ "amount": 7 })),
                    start_at_node_id: None,
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb),
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "workflow control service should seed one waiting checkpoint before interrupt",
    );

    assert_eq!(
        seeded_report.execution.outcome,
        BpmnAdvanceOutcome::WaitingExternalEvent
    );

    let interrupt_report = ok_of(
        service
            .interrupt_workflow(&QianjiBpmnWorkflowInterruptRequest {
                instance_id: "wf_interrupt_duckdb".to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
            })
            .await,
        "workflow control service should preserve one interrupted duckdb checkpoint",
    );

    assert_eq!(interrupt_report.checkpoint_store.backend_name(), "duckdb");
    assert_eq!(
        interrupt_report.instance.instance_id.as_ref(),
        "wf_interrupt_duckdb"
    );
    assert!(matches!(
        interrupt_report.instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Suspended
    ));
    assert!(matches!(
        interrupt_report.instance.suspend_reason,
        Some(qianji_bpmn_engine::SuspendReason::HostRequested)
    ));

    let checkpoint = ok_of(
        interrupt_report
            .checkpoint_store
            .load("wf_interrupt_duckdb")
            .await,
        "interrupted duckdb checkpoint should be preserved",
    )
    .unwrap_or_else(|| panic!("interrupted duckdb checkpoint should still exist"));
    assert_eq!(checkpoint.sequence, interrupt_report.checkpoint_sequence);
    assert!(matches!(
        checkpoint.state.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Suspended
    ));

    let resume_report = ok_of(
        service
            .resume_workflow(
                &QianjiBpmnWorkflowResumeRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    instance_id: "wf_interrupt_duckdb".to_string(),
                    checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "resume should clear host-requested interrupt and continue from preserved checkpoint",
    );
    assert_eq!(
        resume_report.execution.outcome,
        BpmnAdvanceOutcome::WaitingExternalEvent
    );
    assert!(resume_report.execution.checkpoint_saved);
    assert!(
        resume_report
            .execution
            .session
            .instance()
            .suspend_reason
            .is_none()
    );
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_runtime_valkey_interrupt_preserves_checkpoint_and_releases_lease()
{
    let valkey = ok_of(
        TestValkey::spawn().await,
        "valkey should start for workflow control service runtime interrupt test",
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
                    instance_id: "wf_interrupt_runtime".to_string(),
                    initial_variables: Some(json!({ "risk": "high" })),
                    start_at_node_id: None,
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey),
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "runtime valkey path should seed one waiting checkpoint before interrupt",
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
    let interrupt_report = ok_of(
        service
            .interrupt_workflow(&QianjiBpmnWorkflowInterruptRequest {
                instance_id: "wf_interrupt_runtime".to_string(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey,
            })
            .await,
        "runtime valkey interrupt should preserve one checkpoint through lease ownership",
    );

    assert_eq!(interrupt_report.checkpoint_store.backend_name(), "valkey");
    assert_eq!(
        interrupt_report.instance.instance_id.as_ref(),
        "wf_interrupt_runtime"
    );
    assert!(matches!(
        interrupt_report.instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Suspended
    ));
    assert!(matches!(
        interrupt_report.instance.suspend_reason,
        Some(qianji_bpmn_engine::SuspendReason::HostRequested)
    ));

    let checkpoint = ok_of(
        interrupt_report
            .checkpoint_store
            .load("wf_interrupt_runtime")
            .await,
        "interrupted runtime checkpoint should be preserved",
    )
    .unwrap_or_else(|| panic!("interrupted runtime checkpoint should still exist"));
    assert_eq!(checkpoint.sequence, interrupt_report.checkpoint_sequence);

    let reacquired = ok_of(
        interrupt_report
            .checkpoint_store
            .try_acquire_lease("wf_interrupt_runtime", "bpmn-scheduler:worker-b", 30_000)
            .await,
        "interrupted runtime checkpoint should release the lease for reuse",
    );
    assert!(reacquired);
}
