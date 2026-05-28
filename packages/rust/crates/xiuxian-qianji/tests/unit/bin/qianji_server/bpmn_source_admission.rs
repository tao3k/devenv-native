use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use tower::util::ServiceExt;
use xiuxian_qianji_bpmn_engine::{
    BpmnParseOptions, BpmnSourceFile, lint_bpmn_source, parse_bpmn_package,
};
use xiuxian_qianji_control::{
    ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore, RunId,
};

use crate::qianji_test_valkey_support::TestValkey;
use crate::runtime_config::QianjiRuntimeEnv;
use crate::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowHttpState,
    qianji_bpmn_workflow_router,
};

const WORKFLOW_SOURCE_REPAIR_BPMN: &str =
    include_str!("../../../../resources/workflows/workflow_source_repair_v1.bpmn");

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_admits_valid_bpmn_source_under_server_cache() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let router = server_router(temp_dir.path());

    let response = router
        .oneshot(post_json(
            "/control/bpmn-source/admit",
            &json!({
                "source_id": "wendao ai/example run",
                "process_id": "mapped_service_boundary",
                "bpmn_xml": MAPPED_SERVICE_BOUNDARY_BPMN,
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("admission route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["source_id"], "wendao_ai_example_run");
    assert_eq!(body["process_id"], "mapped_service_boundary");
    assert_eq!(body["media_type"], "application/bpmn+xml");
    assert_eq!(body["lint_issue_count"], 0);
    let bpmn_path = PathBuf::from(
        body["bpmn_path"]
            .as_str()
            .unwrap_or_else(|| panic!("response should include bpmn_path: {body}")),
    );
    assert!(
        bpmn_path.starts_with(temp_dir.path().join(".cache/qianji/bpmn-sources")),
        "admitted source should be server-owned under the configured project cache: {body}",
    );
    assert_eq!(
        fs::read_to_string(&bpmn_path)
            .unwrap_or_else(|error| panic!("admitted BPMN should be readable: {error}")),
        MAPPED_SERVICE_BOUNDARY_BPMN,
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_admits_markdown_workflow_source_as_server_owned_bpmn() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let router = server_router(temp_dir.path());

    let response = router
        .oneshot(post_json(
            "/control/workflow-source/admit",
            &json!({
                "source_id": "daily report/run 1",
                "process_id": "Process_wf-1",
                "source_media_type": "text/markdown",
                "source_text": "# Daily Report Generator\n\n## Step 1: Gather Inputs\nRead the source metrics.\n\n## Step 2: Draft Summary\nReturn a concise report.",
                "workflow_name": "Daily Report Generator",
                "workflow_description": "Creates a durable daily report.",
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("workflow-source admission route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["source_id"], "daily_report_run_1");
    assert_eq!(body["process_id"], "Process_wf-1");
    assert_eq!(body["media_type"], "application/bpmn+xml");
    assert_eq!(body["authoring_media_type"], "text/markdown");
    assert_eq!(body["compiler"], "qianji-server-markdown-step-compiler-v1");
    assert_eq!(body["lint_issue_count"], 0);
    let bpmn_path = PathBuf::from(
        body["bpmn_path"]
            .as_str()
            .unwrap_or_else(|| panic!("response should include bpmn_path: {body}")),
    );
    assert!(
        bpmn_path.starts_with(temp_dir.path().join(".cache/qianji/bpmn-sources")),
        "admitted source should be server-owned under the configured project cache: {body}",
    );
    let bpmn_xml = fs::read_to_string(&bpmn_path)
        .unwrap_or_else(|error| panic!("admitted BPMN should be readable: {error}"));
    assert!(bpmn_xml.contains("<bpmn:process id=\"Process_wf-1\""));
    assert!(bpmn_xml.contains("<bpmn:serviceTask id=\"step-1\" name=\"Gather Inputs\">"));
    assert!(bpmn_xml.contains("<bpmn:serviceTask id=\"step-2\" name=\"Draft Summary\">"));
    assert!(bpmn_xml.contains("Workflow goal: Creates a durable daily report."));
    assert!(bpmn_xml.contains("Instructions:\nRead the source metrics."));
    assert!(bpmn_xml.contains("<bpmn:dataOutput id=\"step-1_output_result\" name=\"result\" />"));
    assert!(bpmn_xml.contains("<bpmn:targetRef>step-1_result</bpmn:targetRef>"));
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_requires_repair_for_markdown_without_explicit_steps() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let router = server_router(temp_dir.path());

    let response = router
        .oneshot(post_json(
            "/control/workflow-source/admit",
            &json!({
                "source_id": "daily report/freeform",
                "process_id": "Process_wf_freeform",
                "source_media_type": "text/markdown",
                "source_text": "# Daily Report Generator\n\nGather metrics and write a report.",
                "workflow_name": "Daily Report Generator",
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("workflow-source admission route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["code"], "workflow_source_repair_required");
    assert!(
        !temp_dir.path().join(".cache/qianji/bpmn-sources").exists(),
        "repair-required authoring sources must not be silently written as admitted BPMN",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_requires_durable_runtime_for_server_repair_mode() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let router = server_router(temp_dir.path());

    let response = router
        .oneshot(post_json(
            "/control/workflow-source/admit",
            &json!({
                "source_id": "daily report/repair",
                "process_id": "Process_wf_repair",
                "source_media_type": "text/markdown",
                "source_text": "# Daily Report Generator\n\nGather metrics and write a report.",
                "workflow_name": "Daily Report Generator",
                "compiler_mode": "server_repair",
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("workflow-source admission route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = response_json(response).await;
    assert_eq!(
        body["code"],
        "workflow_source_repair_control_ledger_unavailable"
    );
    let message = body["message"]
        .as_str()
        .unwrap_or_else(|| panic!("response should include message: {body}"));
    assert!(message.contains("durable control ledger"));
    assert!(!message.contains("prompt_schema"));
    assert!(
        !temp_dir.path().join(".cache/qianji/bpmn-sources").exists(),
        "unavailable repair compiler must not write an admitted BPMN source",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_starts_durable_repair_flow_when_runtime_substrates_exist() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start: {error}"));
    let router = server_router_with_repair_runtime(temp_dir.path(), valkey.url().to_string());

    let response = router
        .oneshot(post_json(
            "/control/workflow-source/admit",
            &json!({
                "source_id": "daily report/repair",
                "process_id": "Process_wf_repair",
                "source_media_type": "text/markdown",
                "source_text": "# Daily Report Generator\n\nGather metrics and write a report.",
                "workflow_name": "Daily Report Generator",
                "workflow_description": "Repair this free-form workflow source.",
                "compiler_mode": "server_repair",
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("workflow-source admission route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = response_json(response).await;
    assert_eq!(body["status"], "repair_started");
    assert_eq!(body["source_id"], "daily report/repair");
    assert_eq!(body["target_process_id"], "Process_wf_repair");
    assert_eq!(body["compiler"], "qianji-server-skill-repair-compiler-v1");
    assert_eq!(
        body["repair_run"]["process_id"],
        "qianji_workflow_source_repair_v1"
    );
    assert_eq!(body["repair_run"]["pending_host_work_count"], 1);
    assert_eq!(
        body["repair_run"]["output_contract"],
        "qianji_workflow_source_repair_result"
    );
    let bpmn_path = PathBuf::from(
        body["repair_run"]["bpmn_path"]
            .as_str()
            .unwrap_or_else(|| panic!("repair response should include bpmn path: {body}")),
    );
    assert!(bpmn_path.exists(), "repair BPMN resource should be written");
    let bpmn_xml = fs::read_to_string(&bpmn_path)
        .unwrap_or_else(|error| panic!("repair BPMN should be readable: {error}"));
    assert!(bpmn_xml.contains("qianji_workflow_source_repair_v1"));
    assert!(bpmn_xml.contains("reason_lint_diagnostics"));

    let run_id = RunId::new(
        body["repair_run"]["run_id"]
            .as_str()
            .unwrap_or_else(|| panic!("repair response should include run id: {body}")),
    )
    .unwrap_or_else(|error| panic!("run id should be valid: {error}"));
    let ledger = DuckDbControlLedger::open(temp_dir.path().join("control-ledger.duckdb"))
        .unwrap_or_else(|error| panic!("control ledger should reopen: {error}"));
    let inventory = ledger
        .load_llm_activity_inventory_projection(&run_id)
        .unwrap_or_else(|error| panic!("LLM inventory should project: {error}"));
    assert_eq!(inventory.summary.total, 1);
    let scheduled_activity = &inventory.items[0];
    assert_eq!(
        scheduled_activity.request_audit_metadata["request_metadata"]["activity_id"], "draft_bpmn",
        "server-owned source_intake must be completed deterministically before LLM scheduling",
    );
    assert_ne!(
        scheduled_activity.request_audit_metadata["request_metadata"]["activity_id"],
        "source_intake",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_autocompletes_repair_lint_after_draft_llm_completion() {
    let repair = start_repair_lint_flow().await;
    let draft_token = llm_activity_token(&repair, "draft_bpmn");
    let complete_body = complete_repair_service_task(
        &repair,
        draft_token,
        "draft_bpmn",
        json!({
            "candidateBpmn": repair_candidate_bpmn("Process_wf_repair_lint"),
        }),
    )
    .await;

    assert_eq!(complete_body["workflow"]["pending_host_work_count"], 1);
    assert_eq!(
        complete_body["workflow"]["pending_host_work"][0]["activity_id"], "reason_lint_diagnostics",
        "run_qianji_lint is server-owned and should complete before the next LLM boundary",
    );
    assert!(
        complete_body["workflow"]["variables"]["lintDiagnostics"]["ok"]
            .as_bool()
            .unwrap_or(false),
        "valid candidate BPMN should produce passing deterministic lint evidence",
    );
    let history_after_draft = repair_control_history(&repair).await;
    let history_wire = history_after_draft.to_string();
    assert!(
        history_wire.contains("reason_lint_diagnostics")
            && history_wire.contains("activity_scheduled"),
        "reason_lint_diagnostics should be projected into server-owned durable history after deterministic lint: {history_after_draft}"
    );

    let reason_token = complete_body["workflow"]["pending_host_work"][0]["token_id"]
        .as_u64()
        .unwrap_or_else(|| panic!("reasoning lint token should be present in runtime snapshot"));
    let final_body = complete_repair_service_task(
        &repair,
        reason_token,
        "reason_lint_diagnostics",
        json!({
            "lintPassed": true,
            "repairRequired": false,
            "repairPlan": "admit lint-clean candidate BPMN",
        }),
    )
    .await;

    assert_eq!(final_body["workflow"]["pending_host_work_count"], 0);
    assert_eq!(final_body["workflow"]["lifecycle"], "completed");
    let admitted_ref = final_body["workflow"]["variables"]["admittedBpmnSourceRef"]
        .as_str()
        .unwrap_or_else(|| panic!("final repair variables should include admitted source ref"));
    assert!(
        PathBuf::from(admitted_ref).exists(),
        "admit_bpmn_source should persist the lint-clean repaired BPMN source",
    );
}

#[test]
fn qianji_server_embeds_lint_clean_workflow_source_repair_bpmn_flow() {
    let source = BpmnSourceFile::new(
        "workflow_source_repair_v1.bpmn",
        WORKFLOW_SOURCE_REPAIR_BPMN,
    );
    let lint_report = lint_bpmn_source(&source);
    assert!(
        lint_report.ok,
        "workflow-source repair BPMN must lint clean: {lint_report:?}",
    );
    let package = parse_bpmn_package(&[source], &BpmnParseOptions::default())
        .unwrap_or_else(|error| panic!("workflow-source repair BPMN should parse: {error:?}"));
    let process = package
        .processes
        .iter()
        .find(|process| process.key.process_id.as_ref() == "qianji_workflow_source_repair_v1")
        .unwrap_or_else(|| panic!("repair BPMN should expose the expected process id"));
    assert_eq!(process.nodes.len(), 9);
    assert!(
        WORKFLOW_SOURCE_REPAIR_BPMN.contains("source_intake")
            && WORKFLOW_SOURCE_REPAIR_BPMN.contains("draft_bpmn")
            && WORKFLOW_SOURCE_REPAIR_BPMN.contains("run_qianji_lint")
            && WORKFLOW_SOURCE_REPAIR_BPMN.contains("reason_lint_diagnostics")
            && WORKFLOW_SOURCE_REPAIR_BPMN.contains("repair_bpmn")
            && WORKFLOW_SOURCE_REPAIR_BPMN.contains("admit_bpmn_source")
            && WORKFLOW_SOURCE_REPAIR_BPMN.contains("candidateBpmn")
            && WORKFLOW_SOURCE_REPAIR_BPMN.contains("repairRequired"),
        "repair BPMN should model intake, draft, lint evidence, reasoning lint, repair, and admission steps",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_rejects_bpmn_admission_when_process_id_is_missing() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let router = server_router(temp_dir.path());

    let response = router
        .oneshot(post_json(
            "/control/bpmn-source/admit",
            &json!({
                "source_id": "bad-process",
                "process_id": "missing_process",
                "bpmn_xml": MAPPED_SERVICE_BOUNDARY_BPMN,
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("admission route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["code"], "bpmn_source_process_missing");
    assert!(
        !temp_dir.path().join(".cache/qianji/bpmn-sources").exists(),
        "rejected sources must not be written into the server cache",
    );
}

struct RepairRunFixture {
    router: Router,
    _temp_dir: TempDir,
    _valkey: TestValkey,
    ledger_path: PathBuf,
    run_id: RunId,
    instance_id: String,
    bpmn_path: String,
}

async fn start_repair_lint_flow() -> RepairRunFixture {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let ledger_path = temp_dir.path().join("control-ledger.duckdb");
    let valkey = TestValkey::spawn()
        .await
        .unwrap_or_else(|error| panic!("valkey should start: {error}"));
    let router = server_router_with_repair_runtime(temp_dir.path(), valkey.url().to_string());
    let response = router
        .clone()
        .oneshot(post_json(
            "/control/workflow-source/admit",
            &json!({
                "source_id": "daily report/repair lint",
                "process_id": "Process_wf_repair_lint",
                "source_media_type": "text/markdown",
                "source_text": "# Daily Report Generator\n\nGather metrics and write a report.",
                "workflow_name": "Daily Report Generator",
                "workflow_description": "Repair this free-form workflow source.",
                "compiler_mode": "server_repair",
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("workflow-source admission route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = response_json(response).await;
    let run_id = RunId::new(
        body["repair_run"]["run_id"]
            .as_str()
            .unwrap_or_else(|| panic!("repair response should include run id: {body}")),
    )
    .unwrap_or_else(|error| panic!("run id should be valid: {error}"));
    let instance_id = body["repair_run"]["instance_id"]
        .as_str()
        .unwrap_or_else(|| panic!("repair response should include instance id: {body}"))
        .to_owned();
    let bpmn_path = body["repair_run"]["bpmn_path"]
        .as_str()
        .unwrap_or_else(|| panic!("repair response should include bpmn path: {body}"))
        .to_owned();

    RepairRunFixture {
        router,
        _temp_dir: temp_dir,
        _valkey: valkey,
        ledger_path,
        run_id,
        instance_id,
        bpmn_path,
    }
}

async fn complete_repair_service_task(
    repair: &RepairRunFixture,
    token_id: u64,
    activity_id: &str,
    data: Value,
) -> Value {
    let response = repair
        .router
        .clone()
        .oneshot(post_json(
            format!("/workflows/{}/tasks/complete", repair.instance_id).as_str(),
            &json!({
                "bpmn_path": repair.bpmn_path,
                "completion": {
                    "token_id": token_id,
                    "process_id": "qianji_workflow_source_repair_v1",
                    "activity_id": activity_id,
                    "kind": "service",
                    "data": data,
                },
            }),
        ))
        .await
        .unwrap_or_else(|error| panic!("repair completion route should respond: {error}"));
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response).await
}

fn llm_activity_token(repair: &RepairRunFixture, activity_id: &str) -> u64 {
    let inventory = repair_llm_inventory(repair);
    let activity = inventory
        .items
        .iter()
        .find(|item| item.request_audit_metadata["request_metadata"]["activity_id"] == activity_id)
        .unwrap_or_else(|| panic!("LLM activity `{activity_id}` should be scheduled"));
    activity.request_audit_metadata["request_metadata"]["token_id"]
        .as_u64()
        .unwrap_or_else(|| panic!("LLM activity `{activity_id}` token should be recorded"))
}

fn repair_llm_inventory(
    repair: &RepairRunFixture,
) -> xiuxian_qianji_control::LlmActivityInventoryProjection {
    let ledger = DuckDbControlLedger::open(&repair.ledger_path)
        .unwrap_or_else(|error| panic!("control ledger should reopen: {error}"));
    ledger
        .load_llm_activity_inventory_projection(&repair.run_id)
        .unwrap_or_else(|error| panic!("LLM inventory should project: {error}"))
}

async fn repair_control_history(repair: &RepairRunFixture) -> Value {
    let response = repair
        .router
        .clone()
        .oneshot(get(format!(
            "/control/runs/{}/history",
            repair.run_id.as_str()
        )
        .as_str()))
        .await
        .unwrap_or_else(|error| panic!("repair history route should respond: {error}"));
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response).await
}

fn server_router(project_root: &Path) -> Router {
    let runtime_env = QianjiRuntimeEnv {
        prj_root: Some(project_root.to_path_buf()),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env.clone());
    let state = QianjiBpmnWorkflowHttpState::new(service, QianjiBpmnHostBridge::default())
        .with_runtime_env(runtime_env);
    qianji_bpmn_workflow_router(state)
}

fn server_router_with_repair_runtime(project_root: &Path, valkey_url: String) -> Router {
    let runtime_env = QianjiRuntimeEnv {
        prj_root: Some(project_root.to_path_buf()),
        qianji_checkpoint_valkey_url: Some(valkey_url),
        ..QianjiRuntimeEnv::default()
    };
    let service = QianjiBpmnWorkflowControlService::new().with_runtime_env(runtime_env.clone());
    let ledger = DuckDbControlLedger::open(project_root.join("control-ledger.duckdb"))
        .unwrap_or_else(|error| panic!("control ledger should open: {error}"));
    let control_ledger: Arc<dyn ControlLedger> = Arc::new(ledger);
    let hot_state: Arc<dyn HotStateStore> = Arc::new(InMemoryHotStateStore::new());
    let state = QianjiBpmnWorkflowHttpState::new(service, QianjiBpmnHostBridge::default())
        .with_activity_evidence_ledger(control_ledger)
        .with_recovery_hot_state(hot_state)
        .with_runtime_env(runtime_env);
    qianji_bpmn_workflow_router(state)
}

fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap_or_else(|error| panic!("POST request should build: {error}"))
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("GET request should build: {error}"))
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("response body should decode as JSON: {error}"))
}

fn repair_candidate_bpmn(process_id: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="defs_{process_id}">
  <bpmn:process id="{process_id}" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="step-1" name="Gather Inputs" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start_step_1" sourceRef="start" targetRef="step-1" />
    <bpmn:sequenceFlow id="flow_step_1_done" sourceRef="step-1" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
    )
}

const MAPPED_SERVICE_BOUNDARY_BPMN: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="defs_mapped_service_boundary">
  <bpmn:process id="mapped_service_boundary" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="resolve_project" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start_resolve" sourceRef="start" targetRef="resolve_project" />
    <bpmn:sequenceFlow id="flow_resolve_done" sourceRef="resolve_project" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#;
