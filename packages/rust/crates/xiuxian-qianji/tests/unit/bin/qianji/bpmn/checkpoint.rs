use super::*;

#[cfg(feature = "sqlite")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_command_resumes_waiting_session_from_sqlite_checkpoint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_waiting_bundle(&temp_dir);
    let sqlite_path = temp_dir.path().join("bpmn.sqlite3");

    let fresh_output = must_ok(
        run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path: bpmn_path.clone(),
            dmn_paths: Vec::new(),
            process_id: "wait_flow".to_string(),
            instance_id: "wf_wait".to_string(),
            context_json: Some("{}".to_string()),
            checkpoint_backend: Some(BpmnCliCheckpointBackend::Sqlite(sqlite_path.clone())),
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
        }))
        .await,
        "fresh bpmn run should save waiting checkpoint",
    );

    assert_eq!(fresh_output.exit_code, 0);
    assert!(
        fresh_output
            .rendered
            .contains("Outcome: waiting_external_event")
    );
    assert!(fresh_output.rendered.contains("Checkpoint source: fresh"));
    assert!(fresh_output.rendered.contains("Checkpoint saved: yes"));

    let resumed_output = must_ok(
        run_bpmn_command(BpmnCliCommand::Run(BpmnRunCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "wait_flow".to_string(),
            instance_id: "wf_wait".to_string(),
            context_json: None,
            checkpoint_backend: Some(BpmnCliCheckpointBackend::Sqlite(sqlite_path)),
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: false,
        }))
        .await,
        "resumed bpmn run should load waiting checkpoint",
    );

    assert_eq!(resumed_output.exit_code, 0);
    assert!(
        resumed_output
            .rendered
            .contains("Outcome: waiting_external_event")
    );
    assert!(
        resumed_output
            .rendered
            .contains("Checkpoint source: resumed")
    );
    assert!(
        resumed_output
            .rendered
            .contains("Checkpoint backend: sqlite")
    );
    assert!(resumed_output.rendered.contains("Checkpoint deleted: no"));
}

#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_command_runtime_valkey_with_scheduler_identity_deletes_terminal_checkpoint() {
    let valkey = must_ok(
        TestValkey::spawn().await,
        "valkey should start for scheduler-backed CLI run",
    );
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_linear_bundle(&temp_dir);
    let runtime_env = QianjiRuntimeEnv {
        qianji_checkpoint_valkey_url: Some(valkey.url().to_string()),
        ..QianjiRuntimeEnv::default()
    };
    let scheduler_identity =
        SchedulerAgentIdentity::new(Some("worker-a".to_string()), Some("manager".to_string()));

    let output = must_ok(
        run_bpmn_run_command_with_runtime_env(
            &BpmnRunCliCommand {
                bpmn_path,
                dmn_paths: Vec::new(),
                process_id: "linear".to_string(),
                instance_id: "wf_cli_runtime_valkey".to_string(),
                context_json: Some("{\"risk\":\"high\"}".to_string()),
                checkpoint_backend: Some(BpmnCliCheckpointBackend::RuntimeValkey),
                host_fixture_path: None,
                event_fixture_path: None,
                trace_stream: false,
                external_host: false,
            },
            Some(&runtime_env),
            Some(&scheduler_identity),
        )
        .await,
        "identity-backed runtime valkey run should succeed",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("Outcome: completed"));
    assert!(
        output
            .rendered
            .contains("Checkpoint backend: runtime_valkey")
    );
    assert!(output.rendered.contains("Checkpoint saved: no"));
    assert!(output.rendered.contains("Checkpoint deleted: yes"));

    let store = must_some(
        must_ok(
            resolve_bpmn_checkpoint_store_with_env(
                Some(&BpmnCliCheckpointBackend::RuntimeValkey),
                Some(&runtime_env),
            ),
            "runtime valkey store should resolve from explicit runtime env",
        ),
        "runtime valkey store should exist",
    );
    let checkpoint = must_ok(
        store.load("wf_cli_runtime_valkey").await,
        "terminal CLI run should load checkpoint cleanly after delete",
    );
    assert!(checkpoint.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_command_runtime_valkey_role_only_identity_falls_back_to_driver() {
    let valkey = must_ok(
        TestValkey::spawn().await,
        "valkey should start for role-only fallback CLI run",
    );
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_linear_bundle(&temp_dir);
    let runtime_env = QianjiRuntimeEnv {
        qianji_checkpoint_valkey_url: Some(valkey.url().to_string()),
        ..QianjiRuntimeEnv::default()
    };
    let scheduler_identity = SchedulerAgentIdentity::new(None, Some("manager".to_string()));

    let output = must_ok(
        run_bpmn_run_command_with_runtime_env(
            &BpmnRunCliCommand {
                bpmn_path,
                dmn_paths: Vec::new(),
                process_id: "linear".to_string(),
                instance_id: "wf_cli_runtime_valkey_role_only".to_string(),
                context_json: Some("{\"risk\":\"high\"}".to_string()),
                checkpoint_backend: Some(BpmnCliCheckpointBackend::RuntimeValkey),
                host_fixture_path: None,
                event_fixture_path: None,
                trace_stream: false,
                external_host: false,
            },
            Some(&runtime_env),
            Some(&scheduler_identity),
        )
        .await,
        "role-only identity should keep CLI on the driver fallback path",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("Outcome: completed"));
    assert!(
        output
            .rendered
            .contains("Checkpoint backend: runtime_valkey")
    );
    assert!(output.rendered.contains("Checkpoint saved: yes"));
    assert!(output.rendered.contains("Checkpoint deleted: no"));

    let store = must_some(
        must_ok(
            resolve_bpmn_checkpoint_store_with_env(
                Some(&BpmnCliCheckpointBackend::RuntimeValkey),
                Some(&runtime_env),
            ),
            "runtime valkey store should resolve from explicit runtime env",
        ),
        "runtime valkey store should exist",
    );
    let checkpoint = must_ok(
        store.load("wf_cli_runtime_valkey_role_only").await,
        "role-only fallback run should keep the terminal checkpoint",
    );
    assert!(checkpoint.is_some());
}
