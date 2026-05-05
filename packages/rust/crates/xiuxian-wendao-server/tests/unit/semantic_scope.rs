use xiuxian_wendao_server::transport::{
    ANALYSIS_SEMANTIC_SCOPE_ROUTE, WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER,
    WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER, validate_semantic_scope_request,
};

#[test]
fn semantic_scope_route_contract_exposes_stable_headers() {
    assert_eq!(ANALYSIS_SEMANTIC_SCOPE_ROUTE, "/analysis/semantic-scope");
    assert_eq!(
        WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER,
        "x-wendao-semantic-task-id"
    );
    assert_eq!(
        WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER,
        "x-wendao-semantic-object-ids"
    );
}

#[test]
fn semantic_scope_request_accepts_task_and_object_ids() {
    let request = validate_semantic_scope_request(
        Some(" task.semantic-ssot.object-schema-pilot "),
        &[
            "component.wendao.query-substrate".to_string(),
            " invariant.llm-output-is-not-authority ".to_string(),
        ],
    )
    .unwrap_or_else(|error| panic!("semantic scope request should validate: {error}"));

    assert_eq!(
        request.task_id.as_deref(),
        Some("task.semantic-ssot.object-schema-pilot")
    );
    assert_eq!(
        request.object_ids,
        vec![
            "component.wendao.query-substrate",
            "invariant.llm-output-is-not-authority"
        ]
    );
}
