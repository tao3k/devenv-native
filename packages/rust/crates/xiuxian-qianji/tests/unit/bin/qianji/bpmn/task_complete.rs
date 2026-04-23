#[cfg(feature = "sqlite")]
use super::*;

#[cfg(feature = "sqlite")]
use crate::test_exports::BpmnTaskCompleteCliCommand;

#[cfg(feature = "sqlite")]
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnCheckpointEnvelope, BpmnHostBridge, BpmnInstanceInit,
    BusinessRuleTaskOutcome, BusinessRuleTaskRequest, EventPollOutcome, EventPollRequest,
    HostBridgeError, ManualTaskOutcome, ManualTaskRequest, ScriptTaskOutcome, ScriptTaskRequest,
    SendTaskOutcome, SendTaskRequest, ServiceTaskOutcome, ServiceTaskRequest, UserTaskOutcome,
    UserTaskRequest, advance_instance, create_instance, save_checkpoint_sql,
};
#[cfg(feature = "sqlite")]
use xiuxian_qianji::load_bpmn_package_from_files;

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_tasks_complete_command_resolves_pending_service_task_checkpoint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_service_task_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("task-complete.sqlite3");
    let fixture_path = write_json_fixture(
        temp_dir.path().join("task-complete-fixture.json"),
        &json!({
            "service_tasks": {
                "review_task": {
                    "data": {
                        "approved": true,
                        "source": "task_complete_command"
                    }
                }
            }
        }),
    );

    seed_pending_service_task_checkpoint(&bpmn_path, &sqlite_path).await;

    let complete_output = must_ok(
        run_bpmn_command(BpmnCliCommand::TaskComplete(BpmnTaskCompleteCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            instance_id: "wf_task_complete".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(sqlite_path),
            host_fixture_path: Some(fixture_path.clone()),
            event_fixture_path: None,
        }))
        .await,
        "bpmn tasks complete should resolve the pending service task checkpoint",
    );

    assert_eq!(complete_output.exit_code, 0);
    assert!(complete_output.rendered.starts_with("# BPMN Task Complete"));
    assert!(complete_output.rendered.contains("Outcome: completed"));
    assert!(
        complete_output
            .rendered
            .contains("Checkpoint source: resumed")
    );
    assert!(
        complete_output
            .rendered
            .contains(&format!("Host fixture: {}", fixture_path.display()))
    );
    assert!(complete_output.rendered.contains("\"approved\": true"));
    assert!(
        complete_output
            .rendered
            .contains("\"source\": \"task_complete_command\"")
    );
}

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_tasks_complete_command_renders_missing_sqlite_checkpoint_cleanly() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_service_task_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("task-complete-missing.sqlite3");

    let output = must_ok(
        run_bpmn_command(BpmnCliCommand::TaskComplete(BpmnTaskCompleteCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            instance_id: "wf_missing_task_complete".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::Sqlite(sqlite_path),
            host_fixture_path: None,
            event_fixture_path: None,
        }))
        .await,
        "bpmn tasks complete should render missing checkpoint cleanly",
    );

    assert_eq!(output.exit_code, 1);
    assert!(output.rendered.starts_with("# BPMN Task Complete"));
    assert!(output.rendered.contains("Checkpoint backend: sqlite"));
    assert!(output.rendered.contains("Checkpoint status: missing"));
}

#[cfg(feature = "sqlite")]
async fn seed_pending_service_task_checkpoint(
    bpmn_path: &std::path::Path,
    sqlite_path: &std::path::Path,
) {
    let package = must_ok(
        load_bpmn_package_from_files(bpmn_path, &[]),
        "service task package should load for checkpoint seed",
    );
    let mut instance = must_ok(
        create_instance(
            package.clone(),
            "review",
            BpmnInstanceInit::new("wf_task_complete", json!({ "risk": "high" }), 10),
        ),
        "service task instance should seed",
    );
    let outcome = must_ok(
        advance_instance(
            package.as_ref(),
            &mut instance,
            &BlockingCheckpointSeedHost { now_ms: 55 },
        )
        .await,
        "service task instance should block on pending host work",
    );

    assert!(matches!(outcome, BpmnAdvanceOutcome::BlockedOnHost(pending) if pending.len() == 1));
    assert_eq!(instance.pending_host_work.len(), 1);
    must_ok(
        save_checkpoint_sql(&BpmnCheckpointEnvelope::from_state(instance), sqlite_path),
        "pending service task checkpoint should persist",
    );
}

#[cfg(feature = "sqlite")]
struct BlockingCheckpointSeedHost {
    now_ms: u64,
}

#[cfg(feature = "sqlite")]
#[async_trait::async_trait]
impl BpmnHostBridge for BlockingCheckpointSeedHost {
    async fn dispatch_send_task(
        &self,
        _request: SendTaskRequest,
    ) -> std::result::Result<SendTaskOutcome, HostBridgeError> {
        unsupported_host_operation("dispatch_send_task")
    }

    async fn dispatch_service_task(
        &self,
        _request: ServiceTaskRequest,
    ) -> std::result::Result<ServiceTaskOutcome, HostBridgeError> {
        unsupported_host_operation("dispatch_service_task")
    }

    async fn dispatch_script_task(
        &self,
        _request: ScriptTaskRequest,
    ) -> std::result::Result<ScriptTaskOutcome, HostBridgeError> {
        unsupported_host_operation("dispatch_script_task")
    }

    async fn dispatch_user_task(
        &self,
        _request: UserTaskRequest,
    ) -> std::result::Result<UserTaskOutcome, HostBridgeError> {
        unsupported_host_operation("dispatch_user_task")
    }

    async fn dispatch_manual_task(
        &self,
        _request: ManualTaskRequest,
    ) -> std::result::Result<ManualTaskOutcome, HostBridgeError> {
        unsupported_host_operation("dispatch_manual_task")
    }

    async fn dispatch_business_rule_task(
        &self,
        _request: BusinessRuleTaskRequest,
    ) -> std::result::Result<BusinessRuleTaskOutcome, HostBridgeError> {
        unsupported_host_operation("dispatch_business_rule_task")
    }

    async fn poll_external_event(
        &self,
        _request: EventPollRequest,
    ) -> std::result::Result<EventPollOutcome, HostBridgeError> {
        unsupported_host_operation("poll_external_event")
    }

    fn now_unix_ms(&self) -> u64 {
        self.now_ms
    }
}

#[cfg(feature = "sqlite")]
fn unsupported_host_operation<T>(
    operation: &'static str,
) -> std::result::Result<T, HostBridgeError> {
    Err(HostBridgeError::UnsupportedOperation { operation })
}
