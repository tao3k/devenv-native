use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use xiuxian_qianji_bpmn_engine::BpmnAdvanceOutcome;

use crate::{
    QianjiBpmnCheckpointStore, QianjiBpmnExecutionFacade, QianjiBpmnExecutionMode,
    QianjiBpmnExecutionRequest, QianjiBpmnHostBridge, SchedulerAgentIdentity,
    load_bpmn_package_from_files,
};

use super::valkey_support::TestValkey;

#[test]
fn execution_facade_selects_scheduler_lifecycle_only_for_valkey_agent_identity() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_linear_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bpmn_path, &[]),
        "linear bundle should load from disk",
    );

    let no_store =
        QianjiBpmnExecutionFacade::new(Arc::clone(&package), None).with_scheduler_identity(
            SchedulerAgentIdentity::new(Some("worker-a".to_string()), Some("manager".to_string())),
        );
    assert_eq!(no_store.execution_mode(), QianjiBpmnExecutionMode::Driver);

    let role_only = QianjiBpmnExecutionFacade::new(
        Arc::clone(&package),
        Some(QianjiBpmnCheckpointStore::valkey(
            "redis://127.0.0.1:6379/0".to_string(),
        )),
    )
    .with_scheduler_identity(SchedulerAgentIdentity::new(
        None,
        Some("manager".to_string()),
    ));
    assert_eq!(role_only.execution_mode(), QianjiBpmnExecutionMode::Driver);

    let scheduler = QianjiBpmnExecutionFacade::new(
        package,
        Some(QianjiBpmnCheckpointStore::valkey(
            "redis://127.0.0.1:6379/0".to_string(),
        )),
    )
    .with_scheduler_identity(SchedulerAgentIdentity::new(
        Some("worker-a".to_string()),
        Some("manager".to_string()),
    ));
    assert_eq!(
        scheduler.execution_mode(),
        QianjiBpmnExecutionMode::SchedulerLifecycle
    );
}

#[tokio::test(flavor = "current_thread")]
async fn execution_facade_runtime_valkey_with_scheduler_identity_deletes_terminal_checkpoint() {
    let valkey = ok_of(
        TestValkey::spawn().await,
        "valkey should start for execution facade selector test",
    );
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_linear_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bpmn_path, &[]),
        "linear bundle should load from disk",
    );
    let store = QianjiBpmnCheckpointStore::valkey(valkey.url().to_string());
    let execution = ok_of(
        QianjiBpmnExecutionFacade::new(Arc::clone(&package), Some(store.clone()))
            .with_scheduler_identity(SchedulerAgentIdentity::new(
                Some("worker-a".to_string()),
                Some("manager".to_string()),
            ))
            .run(
                &QianjiBpmnExecutionRequest::new(
                    "linear",
                    "wf_runtime_selector",
                    Some(json!({ "risk": "high" })),
                    11,
                ),
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "scheduler-backed execution facade should complete linear run",
    );

    assert_eq!(execution.outcome, BpmnAdvanceOutcome::Completed);
    assert!(!execution.checkpoint_saved);
    assert!(execution.checkpoint_deleted);

    let stored = ok_of(
        store.load("wf_runtime_selector").await,
        "selector-backed terminal run should load checkpoint cleanly after delete",
    );
    assert!(stored.is_none());
}

fn write_linear_bundle(temp_dir: &TempDir) -> std::path::PathBuf {
    let bpmn_path = temp_dir.path().join("linear.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_linear">
  <bpmn:process id="linear" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        ok_of(
            fs::create_dir_all(parent),
            "selector test should create fixture parent directory",
        );
    }
    ok_of(
        fs::write(path, content),
        "selector test should write fixture file",
    );
}

fn ok_of<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}
