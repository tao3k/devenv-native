use std::time::Instant;

use super::{
    STDIO_SESSION_RESPONSE_KIND, StdioSessionInput, parse_stdio_session_input,
    parse_stdio_session_request, stdio_session_response,
};

#[test]
fn parse_stdio_session_request_trims_intent_and_keeps_request_id() {
    let Ok(request) = parse_stdio_session_request(
        r#"{"requestId":"req-1","intent":"  find ownership evidence  "}"#,
    ) else {
        panic!("parse stdio request");
    };

    assert_eq!(request.request_id.as_deref(), Some("req-1"));
    assert_eq!(request.intent, "find ownership evidence");
    assert!(request.query_understanding_arrow_ipc_path.is_none());
    assert!(request.branch_judgements_arrow_ipc_path.is_none());
}

#[test]
fn parse_stdio_session_request_accepts_query_understanding_arrow_ipc_path() {
    let Ok(request) = parse_stdio_session_request(
        r#"{"requestId":"req-1","intent":"find ownership evidence","queryUnderstandingArrowIpcPath":"/tmp/query-understanding.arrow"}"#,
    ) else {
        panic!("parse stdio request with query-understanding Arrow IPC path");
    };

    assert_eq!(
        request.query_understanding_arrow_ipc_path.as_deref(),
        Some("/tmp/query-understanding.arrow")
    );
}

#[test]
fn parse_stdio_session_request_accepts_branch_judgements_arrow_ipc_path() {
    let Ok(request) = parse_stdio_session_request(
        r#"{"requestId":"req-1","intent":"find ownership evidence","branchJudgementsArrowIpcPath":"/tmp/branch-judgements.arrow"}"#,
    ) else {
        panic!("parse stdio request with branch judgement Arrow IPC path");
    };

    assert_eq!(
        request.branch_judgements_arrow_ipc_path.as_deref(),
        Some("/tmp/branch-judgements.arrow")
    );
}

#[test]
fn parse_stdio_session_request_accepts_ontology_registry_arrow_ipc_path() {
    let Ok(request) = parse_stdio_session_request(
        r#"{"requestId":"req-1","intent":"find PatientRecord evidence","ontologyRegistryArrowIpcPath":"/tmp/ontology-registry.arrow"}"#,
    ) else {
        panic!("parse stdio request with ontology registry Arrow IPC path");
    };

    assert_eq!(
        request.ontology_registry_arrow_ipc_path.as_deref(),
        Some("/tmp/ontology-registry.arrow")
    );
}

#[test]
fn parse_stdio_session_input_accepts_batch_requests() {
    let Ok(input) = parse_stdio_session_input(
        r#"{"kind":"xiuxian_wendao.wendaograph.search_strategy_flow.persistent_stdio_batch_request.v1","requests":[{"requestId":"req-1","intent":"  find ownership evidence  "},{"requestId":"req-2","intent":"find validation evidence"}]}"#,
    ) else {
        panic!("parse stdio batch request");
    };

    let StdioSessionInput::Batch(batch) = input else {
        panic!("stdio input should be a batch");
    };
    assert_eq!(batch.requests.len(), 2);
    assert_eq!(batch.requests[0].request_id.as_deref(), Some("req-1"));
    assert_eq!(batch.requests[0].intent, "find ownership evidence");
    assert_eq!(batch.requests[1].request_id.as_deref(), Some("req-2"));
    assert_eq!(batch.requests[1].intent, "find validation evidence");
}

#[test]
fn parse_stdio_session_input_rejects_empty_batch() {
    let Err(error) = parse_stdio_session_input(r#"{"requests":[]}"#) else {
        panic!("empty stdio batch must fail");
    };

    assert_eq!(
        error,
        "SearchStrategyFlow stdio batch request must not be empty"
    );
}

#[test]
fn parse_stdio_session_request_rejects_blank_intent() {
    let Err(error) = parse_stdio_session_request(r#"{"requestId":"req-1","intent":"   "}"#) else {
        panic!("blank stdio intent must fail");
    };

    assert_eq!(
        error,
        "SearchStrategyFlow stdio request intent must not be blank"
    );
}

#[test]
fn stdio_session_response_embeds_trace_json_control_receipt() {
    let response = stdio_session_response(
        Some("req-1"),
        Instant::now(),
        Ok(r#"{"validation":{"requiredEvidenceCovered":true}}"#.to_owned()),
    );

    assert_eq!(response["kind"], STDIO_SESSION_RESPONSE_KIND);
    assert_eq!(response["requestId"], "req-1");
    assert_eq!(response["ok"], true);
    let legacy_trace_wrapper = ["trace", "Arrow", "Ipc", "Base64"].join("");
    assert!(response.get(legacy_trace_wrapper.as_str()).is_none());
    assert_eq!(
        response["trace"]["validation"]["requiredEvidenceCovered"],
        true
    );
}
