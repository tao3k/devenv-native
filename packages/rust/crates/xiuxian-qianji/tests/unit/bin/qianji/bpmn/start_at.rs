use super::*;
use crate::bpmn_cli::render::render_bpmn_pending_host_work_stream_lines;
use qianji_bpmn_engine::{BpmnAdvanceOutcome, BpmnCheckpointEnvelope, BpmnInstanceInit};
use std::sync::Arc;
use xiuxian_qianji::{QianjiBpmnHostBridge, QianjiBpmnSession, load_bpmn_package_from_files};

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

#[cfg(feature = "duckdb")]
#[tokio::test(flavor = "current_thread")]
async fn run_bpmn_start_at_and_status_render_human_task_interaction_contract() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_form_manual_task_bundle(&temp_dir);
    let duckdb_path = temp_dir.path().join("form-status.duckdb");
    let runtime_env = QianjiRuntimeEnv {
        qianji_workflow_state_duckdb_path: Some(duckdb_path),
        ..QianjiRuntimeEnv::default()
    };

    let start_output = must_ok(
        run_bpmn_start_at_command_with_runtime_env(
            &BpmnStartAtCliCommand {
                bpmn_path: bpmn_path.clone(),
                dmn_paths: Vec::new(),
                process_id: "question_flow".to_string(),
                instance_id: "wf_status_form".to_string(),
                context_json: Some(
                    json!({
                        "currentQuestion": "Ready?",
                        "currentChoices": [{"value": "yes", "label": "Yes"}],
                    })
                    .to_string(),
                ),
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
        "start-at should render pending form manual task",
    );

    assert_eq!(start_output.exit_code, 0);
    assert!(start_output.rendered.contains("activity=Task_Question"));
    assert!(
        start_output
            .rendered
            .contains("form=choice_input result=answer fields=feedback?")
    );
    assert!(
        start_output
            .rendered
            .contains("assignment=human_performer:reviewer:expr=users.alice;potential_owner:review_team:ref=reviewers")
    );

    let status_output = must_ok(
        run_bpmn_status_command_with_runtime_env(
            &BpmnStatusCliCommand {
                instance_id: "wf_status_form".to_string(),
                checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
                bpmn_path: Some(bpmn_path),
                dmn_paths: Vec::new(),
            },
            Some(&runtime_env),
        )
        .await,
        "status should render pending form manual task",
    );

    assert_eq!(status_output.exit_code, 0);
    assert!(status_output.rendered.contains("activity=Task_Question"));
    assert!(
        status_output
            .rendered
            .contains("form=choice_input result=answer fields=feedback?")
    );
    assert!(
        status_output
            .rendered
            .contains("assignment=human_performer:reviewer:expr=users.alice;potential_owner:review_team:ref=reviewers")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn pending_host_work_stream_includes_human_task_form_contract() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_form_manual_task_bundle(&temp_dir);
    let package = must_ok(
        load_bpmn_package_from_files(&bpmn_path, &[]),
        "form manual task package should load",
    );
    let mut instance = must_ok(
        qianji_bpmn_engine::create_instance(
            Arc::clone(&package),
            "question_flow",
            BpmnInstanceInit::new(
                "wf_stream_form",
                json!({
                    "currentQuestion": "Ready?",
                    "currentChoices": [{"value": "yes", "label": "Yes"}],
                }),
                10,
            ),
        ),
        "form manual task instance should seed",
    );
    let outcome = must_ok(
        qianji_bpmn_engine::advance_instance(
            package.as_ref(),
            &mut instance,
            &QianjiBpmnHostBridge::default(),
        )
        .await,
        "form manual task instance should block on pending host work",
    );
    assert!(matches!(outcome, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let session = must_ok(
        QianjiBpmnSession::from_checkpoint(
            Arc::clone(&package),
            BpmnCheckpointEnvelope::from_state(instance),
        ),
        "checkpointed form manual task session should load",
    );
    let lines = render_bpmn_pending_host_work_stream_lines(&session);
    assert_eq!(lines.len(), 1);
    let Some(payload) = lines[0].strip_prefix("@@QIANJI_HOST_WORK ") else {
        panic!("pending host work stream should use marker prefix");
    };
    let value: serde_json::Value = must_ok(
        serde_json::from_str(payload),
        "pending host work stream should be JSON",
    );

    assert_eq!(value["kind"], json!("manual"));
    assert_eq!(value["process_id"], json!("question_flow"));
    assert_eq!(value["activity_id"], json!("Task_Question"));
    assert_eq!(value["form"]["interaction_type"], json!("choice_input"));
    assert_eq!(value["form"]["question_ref"], json!("currentQuestion"));
    assert_eq!(value["form"]["choices_ref"], json!("currentChoices"));
    assert_eq!(value["form"]["result_output"], json!("answer"));
    assert_eq!(
        value["form"]["free_text_fields"][0]["name"],
        json!("feedback")
    );
    assert_eq!(
        value["assignment"]["human_performers"][0]["assignment_expression"],
        json!("users.alice")
    );
    assert_eq!(
        value["assignment"]["potential_owners"][0]["resource_ref"],
        json!("reviewers")
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

fn write_form_manual_task_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("form-manual-task.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:qianji="https://qianji.dev/bpmn/extensions" id="pkg_question">
  <bpmn:process id="question_flow" isExecutable="true">
    <bpmn:startEvent id="Start_1" />
    <bpmn:manualTask id="Task_Question">
      <bpmn:extensionElements>
        <qianji:interaction type="choice_input">
          <qianji:question ref="currentQuestion"/>
          <qianji:choices ref="currentChoices"/>
          <qianji:freeText name="feedback" optional="true"/>
          <qianji:result output="answer"/>
        </qianji:interaction>
      </bpmn:extensionElements>
      <bpmn:humanPerformer name="reviewer">
        <bpmn:resourceAssignmentExpression>
          <bpmn:formalExpression>users.alice</bpmn:formalExpression>
        </bpmn:resourceAssignmentExpression>
      </bpmn:humanPerformer>
      <bpmn:potentialOwner name="review_team">
        <bpmn:resourceRef>reviewers</bpmn:resourceRef>
      </bpmn:potentialOwner>
    </bpmn:manualTask>
    <bpmn:endEvent id="End_1" />
    <bpmn:sequenceFlow id="Flow_1" sourceRef="Start_1" targetRef="Task_Question" />
    <bpmn:sequenceFlow id="Flow_2" sourceRef="Task_Question" targetRef="End_1" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}
