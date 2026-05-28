use crate::qianji_test_valkey_support::TestValkey;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use serde_json::Value;
use serde_json::json;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tower::util::ServiceExt;
use xiuxian_qianji_control::{
    ControlLedger, InMemoryControlLedger, InMemoryHotStateStore, RunCreatedJournalRecord, RunId,
    record_admitted_activity_task_schedule_idempotent, record_run_created,
};

use super::support::{must_ok, write_file};
use crate::qianji_server::flowhub_worker::{
    QianjiServerFlowhubServiceWorkerLoopRequest,
    run_qianji_server_flowhub_service_worker_completion_loop,
};
use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::flowhub::resolve_qianji_server_flowhub_root;
use crate::qianji_server_cli::run::{build_qianji_server_router, build_workflow_control_service};
use crate::{
    FlowhubScenarioIdRef, FlowhubServiceActivityHttpScheduleInput, QianjiBpmnHostBridge,
    QianjiBpmnPendingHostWorkHttpResponse, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowHttpState, QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeInstantMs,
    build_flowhub_service_activity_schedule_record_from_http_pending_work,
    build_flowhub_service_task_complete_http_request,
};

#[test]
fn qianji_server_flowhub_root_prefers_explicit_startup_path() {
    let explicit = PathBuf::from("custom-flowhub");

    assert_eq!(
        resolve_qianji_server_flowhub_root(Some(explicit.as_path())),
        explicit
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_flowhub_scenarios_serves_registry_contract() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let flowhub_root = temp_dir.path().join("flowhub");
    copy_agent_coding_pair(&flowhub_root);
    let router = server_router(flowhub_root);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/flowhub/scenarios")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["passed"], true);
    assert!(
        body["sourcePairs"]
            .as_array()
            .unwrap_or_else(|| panic!("sourcePairs should be an array"))
            .iter()
            .any(|source_pair| source_pair["scenarioId"] == "agent-coding"
                && source_pair["orgSha256"]
                    .as_str()
                    .is_some_and(|hash| hash.len() == 64)
                && source_pair["bpmnSha256"]
                    .as_str()
                    .is_some_and(|hash| hash.len() == 64)
                && source_pair["bpmnProcessId"] == "agent_coding"),
        "unexpected registry response: {body}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_flowhub_source_pair_starts_through_workflow_http_route() {
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start for qianji-server proof: {error}"));
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let flowhub_root = temp_dir.path().join("flowhub");
    write_wait_flowhub_pair(&flowhub_root);
    let router = server_router_with_valkey(flowhub_root, valkey.url().to_string());

    let registry = response_json(
        router
            .clone()
            .oneshot(get("/flowhub/scenarios"))
            .await
            .unwrap_or_else(|error| panic!("registry route should respond: {error}")),
    )
    .await;
    let source_pair = registry["sourcePairs"]
        .as_array()
        .unwrap_or_else(|| panic!("sourcePairs should be an array"))
        .iter()
        .find(|source_pair| source_pair["scenarioId"] == "server-wait")
        .unwrap_or_else(|| panic!("server-wait source pair should exist: {registry}"));
    let bpmn_source = source_pair["bpmnSource"]
        .as_str()
        .unwrap_or_else(|| panic!("bpmnSource should be a string"));
    let bpmn_process_id = source_pair["bpmnProcessId"]
        .as_str()
        .unwrap_or_else(|| panic!("bpmnProcessId should be a string"));
    let instance_id = "flowhub_gateway_http_bridge";

    let start_payload = json!({
        "bpmn_path": bpmn_source,
        "process_id": bpmn_process_id,
        "instance_id": instance_id,
        "initial_variables": {
            "flowhubScenarioId": "server-wait"
        }
    });
    let start_response = router
        .clone()
        .oneshot(post_json("/workflows/start", &start_payload))
        .await
        .unwrap_or_else(|error| panic!("workflow start route should respond: {error}"));

    let start_status = start_response.status();
    let start_body = response_json(start_response).await;
    assert_eq!(
        start_status,
        StatusCode::OK,
        "workflow start should succeed: {start_body}"
    );
    assert_eq!(start_body["workflow"]["instance_id"], instance_id);
    assert_eq!(start_body["workflow"]["process_id"], "server_wait");
    assert_eq!(
        start_body["workflow"]["variables"]["flowhubScenarioId"],
        "server-wait"
    );
    assert_eq!(start_body["checkpoint_saved"], true);
    assert_eq!(start_body["workflow"]["wait_registration_count"], 1);

    let status_response = router
        .oneshot(get(format!("/workflows/{instance_id}").as_str()))
        .await
        .unwrap_or_else(|error| panic!("workflow status route should respond: {error}"));

    assert_eq!(status_response.status(), StatusCode::OK);
    let status_body = response_json(status_response).await;
    assert_eq!(status_body["workflow"]["instance_id"], instance_id);
    assert_eq!(status_body["workflow"]["process_id"], "server_wait");
    assert_eq!(status_body["workflow"]["wait_registration_count"], 1);
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_agent_coding_stops_at_service_task_boundary() {
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start for qianji-server proof: {error}"));
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let flowhub_root = temp_dir.path().join("flowhub");
    copy_agent_coding_pair(&flowhub_root);
    let router = server_router_with_valkey(flowhub_root, valkey.url().to_string());
    let source_pair = agent_coding_runtime_source(&router).await;
    let instance_id = "flowhub_agent_coding_service_boundary";

    let (start_status, start_body) =
        start_agent_coding_workflow(&router, &source_pair, instance_id).await;
    assert_eq!(
        start_status,
        StatusCode::OK,
        "agent-coding start should stop at pending service work: {start_body}"
    );
    assert_eq!(start_body["workflow"]["instance_id"], instance_id);
    assert_eq!(start_body["workflow"]["process_id"], "agent_coding");
    assert_eq!(start_body["checkpoint_saved"], true);
    assert_eq!(start_body["workflow"]["pending_host_work_count"], 1);
    assert_eq!(
        start_body["workflow"]["pending_host_work"][0]["kind"],
        "service"
    );
    assert_eq!(
        start_body["workflow"]["pending_host_work"][0]["activity_id"],
        "resolve_project"
    );

    let complete_payload = flowhub_worker_completion_request(
        &source_pair,
        instance_id,
        &start_body["workflow"]["pending_host_work"][0],
        json!({"projectResolved": true}),
    );
    let (complete_status, complete_body) = post_json_response(
        &router,
        format!("/workflows/{instance_id}/tasks/complete").as_str(),
        &serde_json::to_value(complete_payload)
            .unwrap_or_else(|error| panic!("completion request should serialize: {error}")),
    )
    .await;
    assert_eq!(
        complete_status,
        StatusCode::OK,
        "agent-coding service completion should stop at the next service task: {complete_body}"
    );
    assert_eq!(complete_body["resumed_from_checkpoint"], true);
    assert_eq!(
        complete_body["workflow"]["variables"]["flowhub"]["resolveProject"],
        true
    );
    assert_eq!(complete_body["workflow"]["pending_host_work_count"], 1);
    assert_eq!(
        complete_body["workflow"]["pending_host_work"][0]["activity_id"],
        "validate_contract"
    );

    let (status, status_body) = workflow_status(&router, instance_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(status_body["workflow"]["pending_host_work_count"], 1);
    assert_eq!(
        status_body["workflow"]["pending_host_work"][0]["activity_id"],
        "validate_contract"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_agent_coding_fixture_worker_bridge_completes_service_chain() {
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start for qianji-server proof: {error}"));
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let flowhub_root = temp_dir.path().join("flowhub");
    copy_agent_coding_pair(&flowhub_root);
    let command = server_command_with_valkey(flowhub_root, valkey.url().to_string());
    let router = must_ok(
        build_qianji_server_router(&command),
        "qianji-server router should build with explicit Flowhub root",
    );
    let workflow_state = QianjiBpmnWorkflowHttpState::new(
        build_workflow_control_service(&command),
        QianjiBpmnHostBridge::default(),
    );
    let source_pair = agent_coding_runtime_source(&router).await;
    let instance_id = "flowhub_agent_coding_worker_loop";

    let (start_status, body) =
        start_agent_coding_workflow(&router, &source_pair, instance_id).await;
    assert_eq!(
        start_status,
        StatusCode::OK,
        "agent-coding start should stop at pending service work: {body}"
    );

    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new(format!("flowhub-agent-coding-{instance_id}"))
        .unwrap_or_else(|error| panic!("run id should build: {error}"));
    let output = run_qianji_server_flowhub_service_worker_completion_loop(
        &workflow_state,
        &ledger,
        &hot_state,
        &QianjiServerFlowhubServiceWorkerLoopRequest {
            run_id: &run_id,
            scenario_id: "agent-coding",
            instance_id,
            bpmn_source: Path::new(source_pair.bpmn_source.as_str()),
            worker_id: "flowhub-service-worker",
            checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey,
            now_ms: 42,
            lease_ttl_ms: 1_000,
            settled_at_ms: 84,
            max_steps: 12,
        },
    )
    .await
    .unwrap_or_else(|error| panic!("server worker completion loop should run: {error}"));
    let completed: Vec<_> = output
        .completed_steps
        .iter()
        .map(|step| step.activity_id.as_str())
        .collect();

    assert_eq!(
        completed,
        vec![
            "resolve_project",
            "validate_contract",
            "materialize_sdd",
            "materialize_org_task",
            "materialize_execplan",
            "lint_generated_org",
            "bounded_implementation",
            "record_evidence",
            "lint_generated_surface",
        ]
    );
    assert!(output.completed_steps.iter().all(|step| step.released));
    assert_eq!(output.final_pending_host_work_count, 0);
    assert!(output.final_report.is_some());
    let (status, body) = workflow_status(&router, instance_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["workflow"]["pending_host_work_count"], 0);
    assert_eq!(
        body["workflow"]["variables"]["flowhub"]["resolveProject"], true,
        "final workflow variables should retain resolveProject: {body}"
    );
    assert_eq!(
        body["workflow"]["variables"]["flowhub"]["validateContract"], true,
        "final workflow variables should retain validateContract: {body}"
    );
    assert_eq!(
        body["workflow"]["variables"]["flowhub"]["lintGeneratedSurface"], true,
        "final workflow variables should retain lintGeneratedSurface: {body}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_http_completes_mapped_service_task_and_stops_at_next_boundary() {
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start for qianji-server proof: {error}"));
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_mapped_service_boundary_bpmn(temp_dir.path());
    let router = server_router_with_valkey(
        temp_dir.path().join("unused-flowhub"),
        valkey.url().to_string(),
    );
    let instance_id = "qianji_server_mapped_service_boundary";

    let start_payload = json!({
        "bpmn_path": bpmn_path.display().to_string(),
        "process_id": "mapped_service_boundary",
        "instance_id": instance_id,
        "initial_variables": {}
    });
    let start_response = router
        .clone()
        .oneshot(post_json("/workflows/start", &start_payload))
        .await
        .unwrap_or_else(|error| panic!("workflow start route should respond: {error}"));
    let start_status = start_response.status();
    let start_body = response_json(start_response).await;
    assert_eq!(
        start_status,
        StatusCode::OK,
        "mapped service start should stop at first service task: {start_body}"
    );
    assert_eq!(start_body["workflow"]["pending_host_work_count"], 1);
    assert_eq!(
        start_body["workflow"]["pending_host_work"][0]["activity_id"],
        "resolve_project"
    );
    let service_token = start_body["workflow"]["pending_host_work"][0]["token_id"]
        .as_u64()
        .unwrap_or_else(|| panic!("pending service token should be an integer"));

    let complete_payload = json!({
        "bpmn_path": bpmn_path.display().to_string(),
        "completion": {
            "token_id": service_token,
            "process_id": "mapped_service_boundary",
            "activity_id": "resolve_project",
            "kind": "service",
            "data": {
                "resolvedProject": true
            }
        }
    });
    let complete_response = router
        .oneshot(post_json(
            format!("/workflows/{instance_id}/tasks/complete").as_str(),
            &complete_payload,
        ))
        .await
        .unwrap_or_else(|error| panic!("service completion route should respond: {error}"));
    let complete_status = complete_response.status();
    let complete_body = response_json(complete_response).await;

    assert_eq!(
        complete_status,
        StatusCode::OK,
        "mapped service completion should stop at the next service boundary: {complete_body}"
    );
    assert_eq!(complete_body["resumed_from_checkpoint"], true);
    assert_eq!(
        complete_body["workflow"]["variables"]["resolvedProject"],
        true
    );
    assert_eq!(complete_body["workflow"]["pending_host_work_count"], 1);
    assert_eq!(
        complete_body["workflow"]["pending_host_work"][0]["activity_id"],
        "validate_contract"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_flowhub_scenarios_rejects_invalid_root() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let router = server_router(temp_dir.path().join("missing-flowhub"));

    let response = router
        .oneshot(
            Request::builder()
                .uri("/flowhub/scenarios")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = response_json(response).await;
    assert_eq!(body["passed"], false);
    assert!(
        body["validation"]["diagnostics"]
            .as_array()
            .unwrap_or_else(|| panic!("diagnostics should be an array"))
            .iter()
            .any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|message| message.contains("Flowhub root"))),
        "unexpected invalid-root response: {body}"
    );
}

fn server_router(flowhub_root: PathBuf) -> Router {
    server_router_with_valkey(flowhub_root, "not-a-valkey-url".to_string())
}

fn server_router_with_valkey(flowhub_root: PathBuf, valkey_url: String) -> Router {
    let command = server_command_with_valkey(flowhub_root, valkey_url);
    must_ok(
        build_qianji_server_router(&command),
        "qianji-server router should build with explicit Flowhub root",
    )
}

fn server_command_with_valkey(
    flowhub_root: PathBuf,
    valkey_url: String,
) -> QianjiServerServeCommand {
    QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: Some(valkey_url),
        require_valkey_ready: None,
        flowhub_root: Some(flowhub_root),
        control_ledger_path: None,
    }
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("GET request should build: {error}"))
}

fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap_or_else(|error| panic!("POST request should build: {error}"))
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("response body should decode as JSON: {error}"))
}

struct FlowhubRuntimeSource {
    bpmn_source: String,
    bpmn_process_id: String,
}

async fn agent_coding_runtime_source(router: &Router) -> FlowhubRuntimeSource {
    let registry = response_json(
        router
            .clone()
            .oneshot(get("/flowhub/scenarios"))
            .await
            .unwrap_or_else(|error| panic!("registry route should respond: {error}")),
    )
    .await;
    let source_pair = registry["sourcePairs"]
        .as_array()
        .unwrap_or_else(|| panic!("sourcePairs should be an array"))
        .iter()
        .find(|source_pair| source_pair["scenarioId"] == "agent-coding")
        .unwrap_or_else(|| panic!("agent-coding source pair should exist: {registry}"));
    FlowhubRuntimeSource {
        bpmn_source: source_pair["bpmnSource"]
            .as_str()
            .unwrap_or_else(|| panic!("bpmnSource should be a string"))
            .to_string(),
        bpmn_process_id: source_pair["bpmnProcessId"]
            .as_str()
            .unwrap_or_else(|| panic!("bpmnProcessId should be a string"))
            .to_string(),
    }
}

async fn start_agent_coding_workflow(
    router: &Router,
    source_pair: &FlowhubRuntimeSource,
    instance_id: &str,
) -> (StatusCode, Value) {
    let start_payload = json!({
        "bpmn_path": source_pair.bpmn_source.as_str(),
        "process_id": source_pair.bpmn_process_id.as_str(),
        "instance_id": instance_id,
        "initial_variables": {
            "flowhubScenarioId": "agent-coding"
        }
    });
    post_json_response(router, "/workflows/start", &start_payload).await
}

fn flowhub_worker_completion_request(
    source_pair: &FlowhubRuntimeSource,
    instance_id: &str,
    pending_work: &Value,
    data: Value,
) -> crate::QianjiBpmnWorkflowTaskCompleteHttpRequest {
    let worker_task = flowhub_worker_task(source_pair, instance_id, pending_work);
    build_flowhub_service_task_complete_http_request(&worker_task, data)
        .unwrap_or_else(|error| panic!("completion request should build: {error}"))
}

fn flowhub_worker_task(
    source_pair: &FlowhubRuntimeSource,
    instance_id: &str,
    pending_work: &Value,
) -> xiuxian_qianji_control::WorkerActivityTask {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new(format!("flowhub-agent-coding-{instance_id}"))
        .unwrap_or_else(|error| panic!("run id should build: {error}"));
    seed_control_run(&ledger, &run_id);
    let http_work: QianjiBpmnPendingHostWorkHttpResponse =
        serde_json::from_value(pending_work.clone())
            .unwrap_or_else(|error| panic!("pending host work should deserialize: {error}"));
    let schedule_record = build_flowhub_service_activity_schedule_record_from_http_pending_work(
        FlowhubServiceActivityHttpScheduleInput {
            run_id: &run_id,
            occurred_at_ms: QianjiRuntimeInstantMs::from_millis(42),
            scenario_id: FlowhubScenarioIdRef::new("agent-coding"),
            instance_id: QianjiRuntimeBpmnInstanceIdRef::new(instance_id),
            bpmn_source: Path::new(source_pair.bpmn_source.as_str()),
            pending_work: &http_work,
        },
    )
    .unwrap_or_else(|error| panic!("Flowhub service schedule should build: {error}"));
    record_admitted_activity_task_schedule_idempotent(&ledger, schedule_record)
        .unwrap_or_else(|error| panic!("Flowhub service schedule should record: {error}"));
    ledger
        .load_worker_activity_tasks(&run_id, None)
        .unwrap_or_else(|error| panic!("worker task projection should load: {error}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("worker task projection should include Flowhub service task"))
}

fn seed_control_run(ledger: &InMemoryControlLedger, run_id: &RunId) {
    record_run_created(
        ledger,
        RunCreatedJournalRecord::new(run_id.clone(), "flowhub server worker bridge proof", 1),
    )
    .unwrap_or_else(|error| panic!("control run seed should append: {error}"));
}

async fn workflow_status(router: &Router, instance_id: &str) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(get(format!("/workflows/{instance_id}").as_str()))
        .await
        .unwrap_or_else(|error| panic!("workflow status route should respond: {error}"));
    let status = response.status();
    (status, response_json(response).await)
}

async fn post_json_response(router: &Router, uri: &str, body: &Value) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(post_json(uri, body))
        .await
        .unwrap_or_else(|error| panic!("workflow route should respond: {error}"));
    let status = response.status();
    (status, response_json(response).await)
}

fn copy_agent_coding_pair(flowhub_root: &Path) {
    let source_root = flowhub_root.join("plan");
    write_file(
        &source_root.join("agent-coding.org"),
        r#"#+TITLE: Agent Coding Flowhub Source

* Scenario
:PROPERTIES:
:FLOWHUB_SCENARIO_ID: agent-coding
:CANONICAL_SOURCE: org+bpmn
:BPMN_SOURCE: agent-coding.bpmn
:BPMN_PROCESS_ID: agent_coding
:END:

#+begin_src mermaid
flowchart LR
  Start["start"] --> Done["done"]
#+end_src
"#,
    );
    write_file(&source_root.join("agent-coding.bpmn"), AGENT_CODING_BPMN);
}

const AGENT_CODING_BPMN: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="defs_agent_coding">
  <bpmn:process id="agent_coding" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="resolve_project">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="resolve_project_output_projectResolved" name="projectResolved" />
        <bpmn:outputSet id="resolve_project_output_set">
          <bpmn:dataOutputRefs>resolve_project_output_projectResolved</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>resolve_project_output_projectResolved</bpmn:sourceRef>
        <bpmn:targetRef>flowhub.resolveProject</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:serviceTask id="validate_contract">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="validate_contract_output_validateContract" name="validateContract" />
        <bpmn:outputSet id="validate_contract_output_set">
          <bpmn:dataOutputRefs>validate_contract_output_validateContract</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>validate_contract_output_validateContract</bpmn:sourceRef>
        <bpmn:targetRef>flowhub.validateContract</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:serviceTask id="materialize_sdd">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="materialize_sdd_output_materializeSdd" name="materializeSdd" />
        <bpmn:outputSet id="materialize_sdd_output_set">
          <bpmn:dataOutputRefs>materialize_sdd_output_materializeSdd</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>materialize_sdd_output_materializeSdd</bpmn:sourceRef>
        <bpmn:targetRef>flowhub.materializeSdd</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:serviceTask id="materialize_org_task">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="materialize_org_task_output_materializeOrgTask" name="materializeOrgTask" />
        <bpmn:outputSet id="materialize_org_task_output_set">
          <bpmn:dataOutputRefs>materialize_org_task_output_materializeOrgTask</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>materialize_org_task_output_materializeOrgTask</bpmn:sourceRef>
        <bpmn:targetRef>flowhub.materializeOrgTask</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:serviceTask id="materialize_execplan">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="materialize_execplan_output_materializeExecPlan" name="materializeExecPlan" />
        <bpmn:outputSet id="materialize_execplan_output_set">
          <bpmn:dataOutputRefs>materialize_execplan_output_materializeExecPlan</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>materialize_execplan_output_materializeExecPlan</bpmn:sourceRef>
        <bpmn:targetRef>flowhub.materializeExecPlan</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:serviceTask id="lint_generated_org">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="lint_generated_org_output_lintGeneratedOrg" name="lintGeneratedOrg" />
        <bpmn:outputSet id="lint_generated_org_output_set">
          <bpmn:dataOutputRefs>lint_generated_org_output_lintGeneratedOrg</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>lint_generated_org_output_lintGeneratedOrg</bpmn:sourceRef>
        <bpmn:targetRef>flowhub.lintGeneratedOrg</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:serviceTask id="bounded_implementation">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="bounded_implementation_output_boundedImplementation" name="boundedImplementation" />
        <bpmn:outputSet id="bounded_implementation_output_set">
          <bpmn:dataOutputRefs>bounded_implementation_output_boundedImplementation</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>bounded_implementation_output_boundedImplementation</bpmn:sourceRef>
        <bpmn:targetRef>flowhub.boundedImplementation</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:serviceTask id="record_evidence">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="record_evidence_output_recordEvidence" name="recordEvidence" />
        <bpmn:outputSet id="record_evidence_output_set">
          <bpmn:dataOutputRefs>record_evidence_output_recordEvidence</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>record_evidence_output_recordEvidence</bpmn:sourceRef>
        <bpmn:targetRef>flowhub.recordEvidence</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:serviceTask id="lint_generated_surface">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="lint_generated_surface_output_lintGeneratedSurface" name="lintGeneratedSurface" />
        <bpmn:outputSet id="lint_generated_surface_output_set">
          <bpmn:dataOutputRefs>lint_generated_surface_output_lintGeneratedSurface</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>lint_generated_surface_output_lintGeneratedSurface</bpmn:sourceRef>
        <bpmn:targetRef>flowhub.lintGeneratedSurface</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start_resolve" sourceRef="start" targetRef="resolve_project" />
    <bpmn:sequenceFlow id="flow_resolve_validate" sourceRef="resolve_project" targetRef="validate_contract" />
    <bpmn:sequenceFlow id="flow_validate_materialize_sdd" sourceRef="validate_contract" targetRef="materialize_sdd" />
    <bpmn:sequenceFlow id="flow_materialize_sdd_org_task" sourceRef="materialize_sdd" targetRef="materialize_org_task" />
    <bpmn:sequenceFlow id="flow_materialize_org_task_execplan" sourceRef="materialize_org_task" targetRef="materialize_execplan" />
    <bpmn:sequenceFlow id="flow_materialize_execplan_lint_org" sourceRef="materialize_execplan" targetRef="lint_generated_org" />
    <bpmn:sequenceFlow id="flow_lint_org_implementation" sourceRef="lint_generated_org" targetRef="bounded_implementation" />
    <bpmn:sequenceFlow id="flow_implementation_evidence" sourceRef="bounded_implementation" targetRef="record_evidence" />
    <bpmn:sequenceFlow id="flow_evidence_lint_surface" sourceRef="record_evidence" targetRef="lint_generated_surface" />
    <bpmn:sequenceFlow id="flow_lint_surface_done" sourceRef="lint_generated_surface" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#;

fn write_wait_flowhub_pair(flowhub_root: &Path) {
    let source_root = flowhub_root.join("server");
    write_file(
        &source_root.join("server-wait.org"),
        r#"#+TITLE: Server Wait Flowhub Scenario
#+FILETAGS: :qianji:flowhub:test:

* Scenario: Server Wait
:PROPERTIES:
:FLOWHUB_SCENARIO_ID: server-wait
:CANONICAL_SOURCE: org+bpmn
:BPMN_SOURCE: server-wait.bpmn
:BPMN_PROCESS_ID: server_wait
:END:

** Intent

Prove qianji-server can start a Flowhub-selected BPMN through its workflow
HTTP route and persist the wait-state checkpoint.

** Mermaid

#+begin_src mermaid
flowchart LR
  Start["start"] --> Wait["message wait"]
  Wait --> Done["done"]
#+end_src
"#,
    );
    write_file(
        &source_root.join("server-wait.bpmn"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="defs_server_wait">
  <bpmn:process id="server_wait" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:intermediateCatchEvent id="wait_message">
      <bpmn:messageEventDefinition messageRef="flowhub_signal" name="FlowhubSignal" />
    </bpmn:intermediateCatchEvent>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start_wait" sourceRef="start" targetRef="wait_message" />
    <bpmn:sequenceFlow id="flow_wait_done" sourceRef="wait_message" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
}

fn write_mapped_service_boundary_bpmn(root: &Path) -> PathBuf {
    let bpmn_path = root.join("mapped-service-boundary.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="defs_mapped_service_boundary">
  <bpmn:process id="mapped_service_boundary" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="resolve_project">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="resolve_project_output_resolvedProject" name="resolvedProject" />
        <bpmn:outputSet id="resolve_project_output_set">
          <bpmn:dataOutputRefs>resolve_project_output_resolvedProject</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>resolve_project_output_resolvedProject</bpmn:sourceRef>
        <bpmn:targetRef>resolvedProject</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:serviceTask id="validate_contract" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start_resolve" sourceRef="start" targetRef="resolve_project" />
    <bpmn:sequenceFlow id="flow_resolve_validate" sourceRef="resolve_project" targetRef="validate_contract" />
    <bpmn:sequenceFlow id="flow_validate_done" sourceRef="validate_contract" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}
