#![cfg(feature = "duckdb")]

use super::support::*;

#[tokio::test(flavor = "current_thread")]
async fn workflow_control_service_lists_duckdb_checkpoint_instances() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("instances.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env);

    for instance_id in ["wf_instances_a", "wf_instances_b"] {
        let report = ok_of(
            service
                .start_workflow(
                    &QianjiBpmnWorkflowStartRequest {
                        bpmn_path: bpmn_path.clone(),
                        dmn_paths: Vec::new(),
                        process_id: "wait_flow".to_string(),
                        instance_id: instance_id.to_string(),
                        initial_variables: Some(json!({ "instance": instance_id })),
                        checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb),
                    },
                    &QianjiBpmnHostBridge::default(),
                )
                .await,
            "workflow control service should seed waiting duckdb checkpoints",
        );
        assert_eq!(
            report.execution.outcome,
            BpmnAdvanceOutcome::WaitingExternalEvent
        );
    }

    let instances_report = ok_of(
        service
            .list_workflow_instances(&QianjiBpmnWorkflowInstancesRequest {
                checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
            })
            .await,
        "workflow control service should list duckdb checkpoint instances",
    );

    let instance_ids = instances_report
        .instances
        .iter()
        .map(|instance| instance.instance_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(instance_ids.len(), 2);
    assert!(instance_ids.contains(&"wf_instances_a"));
    assert!(instance_ids.contains(&"wf_instances_b"));
    assert!(instances_report.instances.iter().all(|instance| {
        instance.process_id == "wait_flow"
            && instance.package_id == "pkg_wait"
            && instance.pending_host_work_count == 0
            && instance.wait_registration_count == 1
    }));
}
