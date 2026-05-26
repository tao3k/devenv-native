use std::path::Path;

use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerOnceStoreRequest,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub(super) fn write_episteme_openai_compatible_fixture(
    root: &Path,
) -> Result<
    (
        std::path::PathBuf,
        std::path::PathBuf,
        std::path::PathBuf,
        std::path::PathBuf,
    ),
    String,
> {
    let prompt_path = root.join("prompt.txt");
    let context_path = root.join("context.json");
    std::fs::write(&prompt_path, "Return an Episteme review artifact.")
        .map_err(|error| format!("should write prompt artifact: {error}"))?;
    std::fs::write(
        &context_path,
        r#"{
  "schema": "xiuxian.wendao.episteme.reasoning_fill_context.v1",
  "fillItem": {
    "fillItemId": "fill.ltc.policy.001"
  },
  "targetContract": {
    "schema": "xiuxian.wendao.episteme.reasoning_target_contract.v1",
    "objectModelSchemaRef": "https://wendao.ai/schema/episteme/object-model-v1.schema.json",
    "objectModelCompatibility": "foundry_style_object_model_v1",
    "operationalTargetLayer": "object_model",
    "semanticSourceAuthority": "rdf",
    "targetLedgerFieldGroup": "object_proposal",
    "patchKind": "object_model_object_type_candidate",
    "allowedPatchKinds": ["object_model_object_type_candidate"],
    "reviewMode": "proposal_patch_only",
    "runtimeMutationAllowed": false,
    "rdfMutationAllowed": false,
    "candidatePatchShape": {
      "patchKind": "object_model_object_type_candidate",
      "fillItemId": "fill.ltc.policy.001",
      "targetLedgerFieldGroup": "object_proposal",
      "objectType": {
        "domain": "episteme://private/medical/ltc",
        "apiName": "LtcPolicyDocument",
        "displayName": "LTC policy document",
        "pluralDisplayName": "LTC policy documents",
        "status": "preview",
        "rdfClass": "https://wendao.ai/private/medical/ltc#PolicyDocument",
        "primaryKey": ["sourceId"],
        "displayNameProperty": "name",
        "titleProperty": "name",
        "interfaces": [],
        "visibility": "private"
      },
      "sourceEvidence": [
        {
          "fileId": "ltc.file.policy.001",
          "relativePath": "policy/source.txt",
          "quote": "Policy evidence body for LTC review.",
          "reason": "supports the object candidate"
        }
      ]
    }
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
    .map_err(|error| format!("should write context artifact: {error}"))?;
    Ok((
        root.join("control.duckdb"),
        prompt_path,
        context_path,
        root.join("artifacts/episteme-reasoning-review.json"),
    ))
}

pub(super) fn write_episteme_service_catalog_context_fixture(
    context_path: &Path,
) -> Result<(), String> {
    std::fs::write(
        context_path,
        r#"{
  "schema": "xiuxian.wendao.episteme.reasoning_fill_context.v1",
  "fillItem": {
    "fillItemId": "fill.ltc.service.001"
  },
  "targetContract": {
    "schema": "xiuxian.wendao.episteme.reasoning_target_contract.v1",
    "objectModelSchemaRef": "https://wendao.ai/schema/episteme/object-model-v1.schema.json",
    "objectModelCompatibility": "foundry_style_object_model_v1",
    "operationalTargetLayer": "object_model",
    "semanticSourceAuthority": "rdf",
    "targetLedgerFieldGroup": "service_catalog_review",
    "patchKind": "object_candidate",
    "allowedPatchKinds": ["object_candidate"],
    "reviewMode": "proposal_patch_only",
    "runtimeMutationAllowed": false,
    "rdfMutationAllowed": false,
    "candidatePatchShape": {
      "patchKind": "object_candidate",
      "fillItemId": "fill.ltc.service.001",
      "targetLedgerFieldGroup": "service_catalog_review",
      "provisionalObjectKey": "ltc.service_item.home_nursing_001",
      "label": "Home nursing service",
      "ontologyClassKey": "ltc.service_item",
      "sourceEvidence": [
        {
          "fileId": "ltc.file.service.001",
          "relativePath": "service/source.txt",
          "quote": "Home nursing service",
          "reason": "supports the service item candidate"
        }
      ]
    }
  },
  "contextEvidence": [
    {
      "extractionRunId": "ltc.extract.test",
      "queueId": "ltc.queue.service.001",
      "fileId": "ltc.file.service.001",
      "relativePath": "service/source.txt",
      "sourceSha256": "sha256-source",
      "textSha256": "sha256-text",
      "textCharCount": 21,
      "extractedText": "Home nursing service"
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
    .map_err(|error| format!("should write service catalog context artifact: {error}"))
}

pub(super) fn episteme_openai_compatible_worker_request<'a>(
    base_url: &'a str,
    output_path: &'a Path,
) -> ActivityWorkerOnceStoreRequest<'a> {
    ActivityWorkerOnceStoreRequest {
        worker_id: "worker-episteme-openrouter",
        task_queue: Some("episteme.ontology.reasoning"),
        now_ms: 8_000,
        lease_ttl_ms: 500,
        heartbeat_ttl_ms: None,
        executor: ActivityExecutorKindArg::OpenAiCompatibleLlm,
        outcome: ActivitySettleOutcomeArg::Complete,
        settled_at_ms: 9_000,
        output_ref_json: None,
        output_hash: None,
        output_artifact_path: Some(output_path),
        output_artifact_dir: None,
        output_artifact_content: None,
        output_artifact_id: Some("artifact-episteme-reasoning-review"),
        output_artifact_kind: Some("episteme.reasoning_fill_review"),
        openai_compatible_base_url: Some(base_url),
        openai_compatible_api_key: Some("test-key"),
        openai_compatible_timeout_ms: Some(5_000),
        error_code: None,
        message: None,
        retryable: None,
        metadata: None,
        json: true,
    }
}

pub(super) async fn spawn_openai_compatible_server(
    status: &'static str,
    body: &'static str,
) -> Result<(String, tokio::sync::oneshot::Receiver<String>), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("should bind test provider server: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("should read test provider address: {error}"))?;
    let (request_tx, request_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        if let Ok((mut stream, _peer)) = listener.accept().await
            && let Ok(request) = read_http_request(&mut stream).await
        {
            let _ = request_tx.send(request);
            let response = format!(
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    Ok((format!("http://{address}/v1"), request_rx))
}

async fn read_http_request(stream: &mut tokio::net::TcpStream) -> Result<String, std::io::Error> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if request_complete(&buffer) {
            break;
        }
    }
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn request_complete(buffer: &[u8]) -> bool {
    let Some(header_end) = find_header_end(buffer) else {
        return false;
    };
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length: "))
        .or_else(|| {
            headers
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
        })
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    buffer.len().saturating_sub(header_end + 4) >= content_length
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}
