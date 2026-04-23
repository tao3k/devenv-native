use qianji_bpmn_engine::{BpmnAdvanceOutcome, DmnEvaluationResult, InstanceLifecycle};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

use crate::{
    BpmnOrchestrationError, QianjiBpmnCheckpointStore, QianjiBpmnExecutionRequest,
    QianjiBpmnExecutionScheduler, QianjiBpmnHostBridge, QianjiBpmnSchedulerLeaseConfig,
    SchedulerAgentIdentity, load_bpmn_package_from_files,
};

use super::valkey_support::TestValkey;

#[tokio::test(flavor = "current_thread")]
async fn execution_scheduler_lease_conflict_rejects_competing_owner() {
    let valkey = ok_of(
        TestValkey::spawn().await,
        "valkey should start for lease-conflict test",
    );
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bpmn_path, &[]),
        "wait bundle should load from disk",
    );
    let store = QianjiBpmnCheckpointStore::valkey(valkey.url().to_string());

    assert!(
        ok_of(
            store
                .try_acquire_lease("wf_wait_lease_conflict", "owner-a", 30_000)
                .await,
            "owner-a should acquire the lease directly",
        ),
        "owner-a should hold the checkpoint lease before scheduler run",
    );

    let scheduler = QianjiBpmnExecutionScheduler::new(Arc::clone(&package), Some(store.clone()))
        .with_checkpoint_lease(QianjiBpmnSchedulerLeaseConfig::new("owner-b", 30_000));
    let request = QianjiBpmnExecutionRequest::new(
        "wait_flow",
        "wf_wait_lease_conflict",
        Some(json!({ "amount": 7 })),
        11,
    );
    let host = QianjiBpmnHostBridge::default();

    let error = match scheduler.run(&request, &host).await {
        Ok(report) => panic!("competing owner should be rejected before running: {report:?}"),
        Err(error) => error,
    };
    match error {
        BpmnOrchestrationError::CheckpointLeaseConflict {
            instance_id,
            owner_token,
        } => {
            assert_eq!(instance_id, "wf_wait_lease_conflict");
            assert_eq!(owner_token, "owner-b");
        }
        other => panic!("unexpected error: {other:?}"),
    }

    assert!(
        ok_of(
            store
                .release_lease("wf_wait_lease_conflict", "owner-a")
                .await,
            "owner-a should release the direct lease cleanly",
        ),
        "owner-a should release the lease after the conflict proof",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn execution_scheduler_lease_waiting_run_saves_and_releases_owner_guardedly() {
    let valkey = ok_of(
        TestValkey::spawn().await,
        "valkey should start for waiting lease test",
    );
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bpmn_path, &[]),
        "wait bundle should load from disk",
    );
    let store = QianjiBpmnCheckpointStore::valkey(valkey.url().to_string());
    let scheduler_identity =
        SchedulerAgentIdentity::new(Some("owner-a".to_string()), Some("manager".to_string()));
    let scheduler = ok_of(
        QianjiBpmnExecutionScheduler::new(Arc::clone(&package), Some(store.clone()))
            .with_scheduler_identity(&scheduler_identity, 30_000),
        "agent-backed scheduler identity should derive one lease owner config",
    );
    let request = QianjiBpmnExecutionRequest::new(
        "wait_flow",
        "wf_wait_lease_waiting",
        Some(json!({ "amount": 7 })),
        11,
    );
    let host = QianjiBpmnHostBridge::default();

    let execution = ok_of(
        scheduler.run(&request, &host).await,
        "lease-owned waiting run should succeed",
    );

    assert_eq!(execution.outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(
        execution.session.instance().lifecycle,
        InstanceLifecycle::Waiting
    );
    assert!(execution.checkpoint_saved);
    assert!(!execution.checkpoint_deleted);

    let Some(stored) = ok_of(
        store.load("wf_wait_lease_waiting").await,
        "waiting run should persist checkpoint state",
    ) else {
        panic!("waiting checkpoint should exist");
    };
    assert_eq!(stored.state.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(stored.state.variables, json!({ "amount": 7 }));

    assert!(
        ok_of(
            store
                .try_acquire_lease("wf_wait_lease_waiting", "owner-b", 30_000)
                .await,
            "owner-b should reacquire after scheduler release",
        ),
        "scheduler should release the lease after the waiting run",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn execution_scheduler_lease_terminal_run_deletes_and_releases_owner_guardedly() {
    let valkey = ok_of(
        TestValkey::spawn().await,
        "valkey should start for terminal lease test",
    );
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bundle = write_business_rule_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bundle.bpmn_path, std::slice::from_ref(&bundle.dmn_path)),
        "business-rule bundle should load from disk",
    );
    let store = QianjiBpmnCheckpointStore::valkey(valkey.url().to_string());
    let scheduler = QianjiBpmnExecutionScheduler::new(Arc::clone(&package), Some(store.clone()))
        .with_checkpoint_lease(QianjiBpmnSchedulerLeaseConfig::new("owner-a", 30_000));
    let request = QianjiBpmnExecutionRequest::new(
        "review",
        "wf_terminal_lease",
        Some(json!({ "risk": "low" })),
        11,
    );
    let host = QianjiBpmnHostBridge::builder()
        .on_business_rule_task(|request| async move {
            Ok(qianji_bpmn_engine::BusinessRuleTaskOutcome {
                evaluation: DmnEvaluationResult::new(
                    request.evaluation.decision.decision_id.as_ref(),
                    json!({ "approval": "approve" }),
                    vec![Arc::<str>::from("rule_approve")],
                ),
            })
        })
        .clock(|| 100)
        .build();

    let execution = ok_of(
        scheduler.run(&request, &host).await,
        "lease-owned terminal run should succeed",
    );

    assert_eq!(execution.outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        execution.session.instance().lifecycle,
        InstanceLifecycle::Completed
    );
    assert!(!execution.checkpoint_saved);
    assert!(execution.checkpoint_deleted);

    let stored = ok_of(
        store.load("wf_terminal_lease").await,
        "terminal run should load checkpoint state cleanly after delete",
    );
    assert!(stored.is_none());

    assert!(
        ok_of(
            store
                .try_acquire_lease("wf_terminal_lease", "owner-b", 30_000)
                .await,
            "owner-b should reacquire after scheduler release",
        ),
        "scheduler should release the lease after terminal cleanup",
    );
}

struct BusinessRuleBundlePaths {
    bpmn_path: std::path::PathBuf,
    dmn_path: std::path::PathBuf,
}

fn write_business_rule_bundle(temp_dir: &TempDir) -> BusinessRuleBundlePaths {
    let bpmn_path = temp_dir.path().join("review.bpmn");
    let dmn_path = temp_dir.path().join("loan-decision.dmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_review">
  <bpmn:process id="review" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:businessRuleTask id="review_task" decisionRef="loan-decision" decisionRefSource="loan-decision.dmn" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review_task" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review_task" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    write_file(
        &dmn_path,
        r#"<definitions xmlns="https://www.omg.org/spec/DMN/20191111/MODEL/"
  id="Definitions_loan"
  name="Loan DRD"
  namespace="http://example.com/dmn">
  <decision id="loan-decision" name="Loan Decision">
    <decisionTable id="decision_table_1" hitPolicy="UNIQUE">
      <input id="input_1" label="risk">
        <inputExpression id="input_expression_1" typeRef="string">
          <text>risk</text>
        </inputExpression>
      </input>
      <output id="output_1" name="approval" label="approval" typeRef="string" />
      <rule id="rule_approve">
        <inputEntry id="input_entry_1">
          <text>"low"</text>
        </inputEntry>
        <outputEntry id="output_entry_1">
          <text>"approve"</text>
        </outputEntry>
      </rule>
      <rule id="rule_review">
        <inputEntry id="input_entry_2">
          <text>-</text>
        </inputEntry>
        <outputEntry id="output_entry_2">
          <text>"review"</text>
        </outputEntry>
      </rule>
    </decisionTable>
  </decision>
</definitions>"#,
    );

    BusinessRuleBundlePaths {
        bpmn_path,
        dmn_path,
    }
}

fn write_wait_bundle(temp_dir: &TempDir) -> std::path::PathBuf {
    let bpmn_path = temp_dir.path().join("wait.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_wait">
  <bpmn:process id="wait_flow" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:intermediateCatchEvent id="wait_message">
      <bpmn:messageEventDefinition messageRef="invoice_received" name="InvoiceReceived" />
    </bpmn:intermediateCatchEvent>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="wait_message" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="wait_message" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

fn write_file(path: &Path, contents: &str) {
    fs::write(path, contents)
        .unwrap_or_else(|error| panic!("should write bundle file {}: {error}", path.display()));
}

fn ok_of<T, E: std::fmt::Debug>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error:?}"),
    }
}
