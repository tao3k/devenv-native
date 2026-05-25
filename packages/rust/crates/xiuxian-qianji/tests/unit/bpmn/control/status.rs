#![cfg(feature = "duckdb")]

use super::support::{
    BpmnAdvanceOutcome, QianjiBpmnHostBridge, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowStartRequest, QianjiRuntimeEnv, TempDir,
    json, ok_of, write_wait_bundle,
};
#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_loads_checkpoint_status_from_duckdb_store() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("status.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    let run_report = ok_of(
        service
            .start_workflow(
                &QianjiBpmnWorkflowStartRequest {
                    bpmn_path,
                    dmn_paths: Vec::new(),
                    process_id: "wait_flow".to_string().into(),
                    instance_id: "wf_status".to_string().into(),
                    initial_variables: Some(json!({ "risk": "high" })),
                    start_at_node_id: None,
                    checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb),
                },
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "workflow control service should seed one waiting duckdb checkpoint",
    );

    assert_eq!(
        run_report.execution.outcome,
        BpmnAdvanceOutcome::WaitingExternalEvent
    );
    assert!(run_report.execution.checkpoint_saved);

    let status_report = ok_of(
        service
            .load_workflow_status(&crate::QianjiBpmnWorkflowStatusRequest {
                instance_id: "wf_status".to_string().into(),
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
            })
            .await,
        "workflow control service should load checkpoint-first status",
    );

    assert_eq!(status_report.checkpoint_store.backend_name(), "duckdb");
    assert_eq!(status_report.instance.instance_id.as_ref(), "wf_status");
    assert_eq!(
        status_report.instance.process.process_id.as_ref(),
        "wait_flow"
    );
    assert!(matches!(
        status_report.instance.lifecycle,
        xiuxian_qianji_bpmn_engine::InstanceLifecycle::Waiting
    ));
    assert_eq!(status_report.instance.waits.len(), 1);
    assert_eq!(status_report.instance.variables, json!({ "risk": "high" }));
    assert_eq!(
        status_report.checkpoint_sequence,
        status_report.instance.sequence
    );
}
