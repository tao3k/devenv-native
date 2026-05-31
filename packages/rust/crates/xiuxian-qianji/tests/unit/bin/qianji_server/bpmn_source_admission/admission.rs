use axum::http::StatusCode;
use serde_json::json;
use std::{fs, path::PathBuf};
use tempfile::TempDir;
use tower::util::ServiceExt;
use xiuxian_qianji_bpmn_engine::{
    BpmnParseOptions, BpmnSourceFile, lint_bpmn_source, parse_bpmn_package,
};

use super::support::{
    MAPPED_SERVICE_BOUNDARY_BPMN, WORKFLOW_SOURCE_REPAIR_BPMN, post_json, response_json,
    server_router,
};

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
