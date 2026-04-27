use super::*;

#[cfg(feature = "duckdb")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_start_at_user_task_saves_waiting_checkpoint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_user_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("start-at.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path),
        ..QianjiRuntimeEnv::default()
    };

    let output = must_ok(
        run_bpmn_start_at_command_with_runtime_env(
            &BpmnStartAtCliCommand {
                bpmn_path: bpmn_path.clone(),
                dmn_paths: Vec::new(),
                process_id: "question_flow".to_string(),
                instance_id: "wf_start_at_question".to_string(),
                context_json: Some("{\"currentQuestion\":\"Ready?\"}".to_string()),
                start_at_node_id: Some("Task_Question".to_string()),
                checkpoint_backend: Some(BpmnCliCheckpointBackend::LocalDuckDb),
                host_fixture_path: None,
                event_fixture_path: None,
                trace_stream: false,
                external_host: true,
                continue_until_human_boundary: false,
            },
            Some(&runtime_env),
            None,
        )
        .await,
        "start-at should create a waiting user-task checkpoint",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# BPMN Start At"));
    assert!(output.rendered.contains("Outcome: blocked_on_host"));
    assert!(output.rendered.contains("Checkpoint source: fresh"));
    assert!(output.rendered.contains("Checkpoint saved: yes"));
    assert!(
        output
            .rendered
            .contains("- Task_Question | token#1 | kind=user")
    );

    let store = must_some(
        must_ok(
            resolve_bpmn_checkpoint_store_with_env(
                Some(&BpmnCliCheckpointBackend::LocalDuckDb),
                Some(&runtime_env),
            ),
            "local DuckDB store should resolve",
        ),
        "local DuckDB store should exist",
    );
    let checkpoint = must_some(
        must_ok(
            store.load("wf_start_at_question").await,
            "start-at checkpoint should load",
        ),
        "start-at checkpoint should exist",
    );
    assert_eq!(checkpoint.state.pending_host_work.len(), 1);
    assert_eq!(
        checkpoint.state.pending_host_work[0].kind,
        qianji_bpmn_engine::PendingHostWorkKind::User
    );

    let collision = run_bpmn_start_at_command_with_runtime_env(
        &BpmnStartAtCliCommand {
            bpmn_path,
            dmn_paths: Vec::new(),
            process_id: "question_flow".to_string(),
            instance_id: "wf_start_at_question".to_string(),
            context_json: Some("{\"currentQuestion\":\"Again?\"}".to_string()),
            start_at_node_id: Some("Task_Question".to_string()),
            checkpoint_backend: Some(BpmnCliCheckpointBackend::LocalDuckDb),
            host_fixture_path: None,
            event_fixture_path: None,
            trace_stream: false,
            external_host: true,
            continue_until_human_boundary: false,
        },
        Some(&runtime_env),
        None,
    )
    .await;
    let error: Box<dyn std::error::Error> = match collision {
        Ok(output) => panic!("checkpoint collision should fail, got {output:?}"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("start-at requires a fresh instance id")
    );
}

fn write_user_task_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("user-task.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_question">
  <bpmn:process id="question_flow" isExecutable="true">
    <bpmn:startEvent id="Start_1" />
    <bpmn:userTask id="Task_Question" />
    <bpmn:endEvent id="End_1" />
    <bpmn:sequenceFlow id="Flow_1" sourceRef="Start_1" targetRef="Task_Question" />
    <bpmn:sequenceFlow id="Flow_2" sourceRef="Task_Question" targetRef="End_1" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}
