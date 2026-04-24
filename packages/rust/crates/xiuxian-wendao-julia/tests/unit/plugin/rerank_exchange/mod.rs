use xiuxian_wendao_core::repo_intelligence::{
    julia_arrow_request_schema, julia_arrow_response_schema,
};

#[test]
fn julia_arrow_request_schema_uses_contract_columns() {
    let schema = julia_arrow_request_schema(3);

    assert_eq!(schema.field(0).name(), "doc_id");
    assert_eq!(schema.field(1).name(), "vector_score");
    assert_eq!(schema.field(2).name(), "embedding");
    assert_eq!(schema.field(3).name(), "query_embedding");
}

#[test]
fn julia_arrow_response_schema_optionally_includes_trace_id() {
    let base = julia_arrow_response_schema(false);
    let traced = julia_arrow_response_schema(true);

    assert_eq!(base.fields().len(), 3);
    assert_eq!(traced.fields().len(), 4);
    assert_eq!(traced.field(3).name(), "trace_id");
}
