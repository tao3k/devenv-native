use super::support::{
    BpmnAdvanceOutcome, QianjiBpmnHostBridge, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowStartRequest, QianjiRuntimeEnv,
    SchedulerAgentIdentity, TempDir, TestValkey, json, ok_of, write_linear_bundle,
    write_wait_bundle,
};

#[test]
fn workflow_control_service_prepares_package_and_resolved_paths() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let request = QianjiBpmnWorkflowStartRequest {
        bpmn_path: bpmn_path.clone(),
        dmn_paths: Vec::new(),
        process_id: "wait_flow".to_string().into(),
        instance_id: "wf_prepare".to_string().into(),
        initial_variables: Some(json!({})),
        start_at_node_id: None,
        checkpoint_backend: None,
    };
    let service = QianjiBpmnWorkflowControlService::new();

    let prepared = ok_of(
        service.prepare_start_workflow(&request),
        "workflow control service should prepare a wait bundle",
    );

    assert!(prepared.resolved_bpmn_path.is_absolute());
    assert_eq!(prepared.resolved_bpmn_path, bpmn_path);
    assert!(prepared.resolved_dmn_paths.is_empty());
    assert_eq!(prepared.execution_request.process_id, "wait_flow");
    assert_eq!(prepared.execution_request.instance_id, "wf_prepare");
    assert_eq!(
        prepared.execution_request.initial_variables,
        Some(json!({}))
    );
    assert!(prepared.package.find_process("wait_flow").is_some());
    assert_eq!(prepared.checkpoint_store, None);
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_runs_prepared_linear_bundle() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_linear_bundle(&temp_dir);
    let request = QianjiBpmnWorkflowStartRequest {
        bpmn_path,
        dmn_paths: Vec::new(),
        process_id: "linear".to_string().into(),
        instance_id: "wf_control_linear".to_string().into(),
        initial_variables: Some(json!({ "risk": "low" })),
        start_at_node_id: None,
        checkpoint_backend: None,
    };
    let service = QianjiBpmnWorkflowControlService::new();
    let prepared = ok_of(
        service.prepare_start_workflow(&request),
        "workflow control service should prepare a linear bundle",
    );

    let report = ok_of(
        service
            .start_prepared_workflow(prepared, &QianjiBpmnHostBridge::default())
            .await,
        "workflow control service should execute a prepared linear bundle",
    );

    assert_eq!(report.execution.outcome, BpmnAdvanceOutcome::Completed);
    assert!(!report.execution.resumed_from_checkpoint);
    assert!(!report.execution.checkpoint_saved);
    assert!(!report.execution.checkpoint_deleted);
    assert_eq!(
        report.execution.session.instance().variables,
        json!({ "risk": "low" })
    );
}

#[cfg(feature = "duckdb")]
#[test]
fn workflow_control_service_resolves_local_duckdb_workflow_state_store() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let duckdb_path = temp_dir.path().join("local-state.duckdb");
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path.clone()),
        ..QianjiRuntimeEnv::default()
    });

    let store = ok_of(
        service.resolve_checkpoint_store(Some(&QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb)),
        "workflow control service should resolve local DuckDB workflow-state store",
    )
    .unwrap_or_else(|| panic!("local DuckDB backend should resolve a store"));

    assert_eq!(store, crate::QianjiBpmnCheckpointStore::duckdb(duckdb_path));
}

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_runtime_valkey_scheduler_identity_deletes_terminal_checkpoint() {
    let valkey = ok_of(
        TestValkey::spawn().await,
        "valkey should start for workflow control service runtime checkpoint test",
    );
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_linear_bundle(&temp_dir);
    let runtime_env = QianjiRuntimeEnv {
        qianji_checkpoint_valkey_url: Some(valkey.url().to_string()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new()
        .with_runtime_env(runtime_env)
        .with_scheduler_identity(SchedulerAgentIdentity::new(
            Some("worker-a".to_string()),
            Some("manager".to_string()),
        ));

    let report = ok_of(
        service
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    process_id: "linear".to_string().into(),
                    instance_id: "wf_control_runtime_valkey".to_string().into(),
                    initial_variables: Some(json!({ "risk": "high" })),
                    start_at_node_id: None,
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey),
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "workflow control service should execute the runtime-valkey path",
    );

    assert_eq!(report.execution.outcome, BpmnAdvanceOutcome::Completed);
    assert!(!report.execution.checkpoint_saved);
    assert!(report.execution.checkpoint_deleted);

    let checkpoint_store = report
        .checkpoint_store
        .as_ref()
        .unwrap_or_else(|| panic!("runtime valkey control-service run should resolve a store"));
    let checkpoint = ok_of(
        checkpoint_store.load("wf_control_runtime_valkey").await,
        "terminal workflow control run should load checkpoint after delete",
    );
    assert!(checkpoint.is_none());
}
