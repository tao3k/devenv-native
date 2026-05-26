use crate::qianji_cli::test_exports::worker_once_with_hot_state;
use crate::qianji_cli::tests::control_cli::support::must_ok;
use tempfile::TempDir;
use xiuxian_qianji_control::{DuckDbControlLedger, InMemoryHotStateStore};

use super::support::{
    episteme_openai_compatible_worker_request, spawn_openai_compatible_server,
    write_episteme_openai_compatible_fixture, write_episteme_service_catalog_context_fixture,
};
use crate::qianji_cli::tests::control_cli::run_activity_queue::worker_once::support::{
    append_control_run_with_episteme_openai_compatible_local_prompt, enqueue_worker_task,
};

#[tokio::test]
async fn worker_once_with_hot_state_executes_episteme_reasoning_openai_compatible_to_review_artifact()
-> Result<(), String> {
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"id":"chatcmpl-test","choices":[{"message":{"role":"assistant","content":"```json\n{\"schema\":\"xiuxian.wendao.episteme.reasoning_fill_review.v1\",\"status\":\"review_only\",\"fillItemId\":\"fill.ltc.policy.001\",\"targetLedgerFieldGroup\":\"object_proposal\",\"candidatePatchCount\":0,\"candidatePatches\":[],\"rdfMutation\":false}\n```"},"finish_reason":"stop"}]}"#,
    )
    .await?;
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let (ledger_path, prompt_path, context_path, output_path) =
        write_episteme_openai_compatible_fixture(temp_dir.path())?;
    let run_id = append_control_run_with_episteme_openai_compatible_local_prompt(
        &ledger_path,
        &prompt_path,
        &context_path,
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(
        &ledger,
        &hot_state,
        &run_id,
        "activity-episteme-reasoning-openai-compatible-llm",
    )
    .await?;

    let output = must_ok(
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &episteme_openai_compatible_worker_request(base_url.as_str(), output_path.as_path()),
        )
        .await,
        "Episteme OpenAI-compatible worker once should execute and render",
    );
    let request = request_rx
        .await
        .map_err(|error| format!("should receive provider request: {error}"))?;
    let artifact: serde_json::Value = must_ok(
        serde_json::from_str(
            &std::fs::read_to_string(&output_path)
                .map_err(|error| format!("should read provider artifact: {error}"))?,
        ),
        "provider artifact should be valid json",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );

    assert!(
        request.contains("Return an Episteme review artifact."),
        "provider request should include prompt text: {request}"
    );
    assert!(
        request.contains("fill.ltc.policy.001"),
        "provider request should include context text: {request}"
    );
    assert!(
        request.contains("Policy evidence body for LTC review."),
        "provider request should include materialized context evidence: {request}"
    );
    assert!(
        request.contains("targetContract"),
        "provider request should include target patch contract: {request}"
    );
    assert_eq!(
        artifact["schema"],
        "qianji.openai_compatible_llm_response.v1"
    );
    assert_eq!(
        artifact["episteme_review"]["schema"],
        "xiuxian.wendao.episteme.reasoning_fill_review.v1"
    );
    assert_eq!(artifact["episteme_review"]["candidatePatchCount"], 0);
    assert_eq!(
        json["claimed"]["activity_task"]["task"]["activity_type"],
        "episteme.ontology.reasoning_fill"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["output_ref"]["artifact_kind"],
        "episteme.reasoning_fill_review"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["result"]["metadata"]["executor"],
        "openai-compatible-llm"
    );
    assert_eq!(json["released"], true);
    Ok(())
}

#[tokio::test]
async fn worker_once_with_hot_state_accepts_episteme_service_catalog_object_candidate()
-> Result<(), String> {
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"id":"chatcmpl-test","choices":[{"message":{"role":"assistant","content":"{\"schema\":\"xiuxian.wendao.episteme.reasoning_fill_review.v1\",\"status\":\"review_only\",\"fillItemId\":\"fill.ltc.service.001\",\"targetLedgerFieldGroup\":\"service_catalog_review\",\"reviewSummary\":\"Service catalog object candidate extracted for review.\",\"candidatePatchCount\":1,\"candidatePatches\":[{\"patchKind\":\"object_candidate\",\"fillItemId\":\"fill.ltc.service.001\",\"targetLedgerFieldGroup\":\"service_catalog_review\",\"provisionalObjectKey\":\"ltc.service_item.home_nursing_001\",\"label\":\"Home nursing service\",\"ontologyClassKey\":\"ltc.service_item\",\"sourceEvidence\":[{\"fileId\":\"ltc.file.service.001\",\"relativePath\":\"service/source.txt\",\"quote\":\"Home nursing service\",\"reason\":\"supports the service item candidate\"}]}],\"blockers\":[],\"rdfMutation\":false}"},"finish_reason":"stop"}]}"#,
    )
    .await?;
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let (ledger_path, prompt_path, context_path, output_path) =
        write_episteme_openai_compatible_fixture(temp_dir.path())?;
    write_episteme_service_catalog_context_fixture(&context_path)?;
    let run_id = append_control_run_with_episteme_openai_compatible_local_prompt(
        &ledger_path,
        &prompt_path,
        &context_path,
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();
    enqueue_worker_task(
        &ledger,
        &hot_state,
        &run_id,
        "activity-episteme-reasoning-openai-compatible-llm",
    )
    .await?;

    let output = must_ok(
        worker_once_with_hot_state(
            &ledger,
            &hot_state,
            &episteme_openai_compatible_worker_request(base_url.as_str(), output_path.as_path()),
        )
        .await,
        "Episteme service catalog worker once should complete",
    );
    let _request = request_rx
        .await
        .map_err(|error| format!("should receive provider request: {error}"))?;
    let artifact: serde_json::Value = must_ok(
        serde_json::from_str(
            &std::fs::read_to_string(&output_path)
                .map_err(|error| format!("should read provider artifact: {error}"))?,
        ),
        "provider artifact should be valid json",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );

    assert_eq!(
        artifact["episteme_review"]["targetLedgerFieldGroup"],
        "service_catalog_review"
    );
    assert_eq!(
        artifact["episteme_review"]["candidatePatches"][0]["patchKind"],
        "object_candidate"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["event"],
        "activity_completed"
    );
    assert_eq!(json["released"], true);
    Ok(())
}
