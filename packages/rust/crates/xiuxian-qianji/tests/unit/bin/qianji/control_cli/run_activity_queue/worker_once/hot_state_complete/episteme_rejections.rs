use crate::qianji_cli::test_exports::worker_once_with_hot_state;
use crate::qianji_cli::tests::control_cli::support::must_ok;
use tempfile::TempDir;
use xiuxian_qianji_control::{DuckDbControlLedger, InMemoryHotStateStore};

use super::support::{
    episteme_openai_compatible_worker_request, spawn_openai_compatible_server,
    write_episteme_openai_compatible_fixture,
};
use crate::qianji_cli::tests::control_cli::run_activity_queue::worker_once::support::{
    append_control_run_with_episteme_openai_compatible_local_prompt, enqueue_worker_task,
};

#[tokio::test]
async fn worker_once_with_hot_state_rejects_episteme_reasoning_legacy_patch_kind()
-> Result<(), String> {
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"id":"chatcmpl-test","choices":[{"message":{"role":"assistant","content":"{\"schema\":\"xiuxian.wendao.episteme.reasoning_fill_review.v1\",\"status\":\"review_only\",\"fillItemId\":\"fill.ltc.policy.001\",\"targetLedgerFieldGroup\":\"object_proposal\",\"candidatePatchCount\":1,\"candidatePatches\":[{\"patchKind\":\"object_candidate\",\"fillItemId\":\"fill.ltc.policy.001\",\"targetLedgerFieldGroup\":\"object_proposal\",\"label\":\"Legacy object\",\"sourceEvidence\":[{\"fileId\":\"ltc.file.policy.001\",\"relativePath\":\"policy/source.txt\",\"quote\":\"Policy evidence body for LTC review.\"}]}],\"rdfMutation\":false}"},"finish_reason":"stop"}]}"#,
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
        "Episteme OpenAI-compatible worker once should fail invalid contract durably",
    );
    let _request = request_rx
        .await
        .map_err(|error| format!("should receive provider request: {error}"))?;
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );

    assert!(!output_path.exists());
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["event"],
        "activity_failed"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["error_code"],
        "provider_contract_invalid"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["retryable"],
        false
    );
    assert!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("is not allowed by targetContract"),
        "unexpected failure message: {json}"
    );
    Ok(())
}

#[tokio::test]
async fn worker_once_with_hot_state_marks_truncated_episteme_json_retryable() -> Result<(), String>
{
    let (base_url, request_rx) = spawn_openai_compatible_server(
        "200 OK",
        r#"{"id":"chatcmpl-test","choices":[{"message":{"role":"assistant","content":"{\"schema\":\"xiuxian.wendao.episteme.reasoning_fill_review.v1\",\"status\":\"review_only\""},"finish_reason":"length"}]}"#,
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
        "truncated Episteme provider JSON should fail durably",
    );
    let _request = request_rx
        .await
        .map_err(|error| format!("should receive provider request: {error}"))?;
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );

    assert!(!output_path.exists());
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["error_code"],
        "provider_contract_invalid"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["retryable"],
        true
    );
    Ok(())
}

#[tokio::test]
async fn worker_once_with_hot_state_rejects_episteme_reasoning_without_context_evidence()
-> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let (ledger_path, prompt_path, context_path, output_path) =
        write_episteme_openai_compatible_fixture(temp_dir.path())?;
    std::fs::write(
        &context_path,
        r#"{
  "schema": "xiuxian.wendao.episteme.reasoning_fill_context.v1",
  "fillItem": {
    "fillItemId": "fill.ltc.policy.001"
  },
  "contextEvidence": [],
  "safety": {
    "sourceTextRead": false,
    "sourceMutationAllowed": false,
    "rdfMutationAllowed": false,
    "ontologyTruth": false
  }
}"#,
    )
    .map_err(|error| format!("should write invalid context artifact: {error}"))?;
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
            &episteme_openai_compatible_worker_request("http://127.0.0.1:9/v1", &output_path),
        )
        .await,
        "Episteme worker once should settle invalid context as durable failure",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );

    assert!(!output_path.exists());
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["event"],
        "activity_failed"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["error_code"],
        "input_artifact_read_failed"
    );
    assert!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("contextEvidence must not be empty"),
        "unexpected failure message: {json}"
    );
    Ok(())
}

#[tokio::test]
async fn worker_once_with_hot_state_rejects_episteme_reasoning_without_target_contract()
-> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let (ledger_path, prompt_path, context_path, output_path) =
        write_episteme_openai_compatible_fixture(temp_dir.path())?;
    std::fs::write(
        &context_path,
        r#"{
  "schema": "xiuxian.wendao.episteme.reasoning_fill_context.v1",
  "fillItem": {
    "fillItemId": "fill.ltc.policy.001"
  },
  "contextEvidence": [
    {
      "extractionRunId": "ltc.extract.test",
      "queueId": "ltc.queue.policy.001",
      "fileId": "ltc.file.policy.001",
      "relativePath": "policy/source.txt",
      "sourceSha256": "sha256-source",
      "textSha256": "sha256-text",
      "textCharCount": 37,
      "extractedText": "Policy evidence body for LTC review."
    }
  ],
  "safety": {
    "sourceTextRead": false,
    "sourceMutationAllowed": false,
    "rdfMutationAllowed": false,
    "ontologyTruth": false
  }
}"#,
    )
    .map_err(|error| format!("should write invalid context artifact: {error}"))?;
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
            &episteme_openai_compatible_worker_request("http://127.0.0.1:9/v1", &output_path),
        )
        .await,
        "Episteme worker once should settle invalid target contract as durable failure",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity worker once output should be valid json",
    );

    assert!(!output_path.exists());
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["event"],
        "activity_failed"
    );
    assert_eq!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["error_code"],
        "input_artifact_read_failed"
    );
    assert!(
        json["terminal"]["record"]["event"]["kind"]["failure"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("requires targetContract"),
        "unexpected failure message: {json}"
    );
    Ok(())
}
