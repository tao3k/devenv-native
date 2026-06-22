use axum::http::StatusCode;
use serde_json::json;
use std::{fs, path::PathBuf};
use tempfile::TempDir;
use tower::util::ServiceExt;
use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

use crate::qianji_test_valkey_support::TestValkey;

use super::support::{
    complete_repair_service_task, llm_activity_token, post_json, repair_candidate_bpmn,
    repair_control_history, response_json, server_router_with_repair_runtime,
    start_repair_lint_flow,
};

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
