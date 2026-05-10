use super::{
    DataType, Field, Schema, WendaoGraphEvidenceTableKind,
    validate_wendao_graph_evidence_request_schema, validate_wendao_graph_evidence_response_schema,
    validate_wendao_graph_page_index_reasoning_request_schema,
    validate_wendao_graph_page_index_reasoning_response_schema, wendao_graph_evidence_table_schema,
    wendao_graph_page_index_reasoning_table_schema,
};

#[test]
fn wendao_graph_evidence_schema_validation_accepts_request_tables() {
    let schema = match wendao_graph_evidence_table_schema(
        WendaoGraphEvidenceTableKind::Request,
        "semantic_neighbors",
    ) {
        Ok(schema) => schema,
        Err(error) => panic!("semantic neighbor request schema should build: {error}"),
    };
    assert_eq!(
        validate_wendao_graph_evidence_request_schema("semantic_neighbors", schema.as_ref()),
        Ok(())
    );
    assert_eq!(schema.field(0).name(), "query_id");
    assert_eq!(schema.field(2).data_type(), &DataType::Int64);
    assert_eq!(schema.field(5).data_type(), &DataType::Float64);
}

#[test]
fn wendao_graph_page_index_reasoning_schema_validation_accepts_tables() {
    let nodes = match wendao_graph_page_index_reasoning_table_schema(
        WendaoGraphEvidenceTableKind::Request,
        "page_index_nodes",
    ) {
        Ok(schema) => schema,
        Err(error) => panic!("page_index_nodes request schema should build: {error}"),
    };
    assert_eq!(
        validate_wendao_graph_page_index_reasoning_request_schema(
            "page_index_nodes",
            nodes.as_ref()
        ),
        Ok(())
    );
    assert_eq!(nodes.field(0).name(), "node_id");
    assert_eq!(nodes.field(3).data_type(), &DataType::Int64);
    assert_eq!(nodes.field(9).name(), "token_count");

    let frontier = match wendao_graph_page_index_reasoning_table_schema(
        WendaoGraphEvidenceTableKind::Response,
        "reasoning_frontier",
    ) {
        Ok(schema) => schema,
        Err(error) => panic!("reasoning_frontier response schema should build: {error}"),
    };
    assert_eq!(
        validate_wendao_graph_page_index_reasoning_response_schema(
            "reasoning_frontier",
            frontier.as_ref()
        ),
        Ok(())
    );
    assert_eq!(frontier.field(7).name(), "score");
    assert_eq!(frontier.field(7).data_type(), &DataType::Float64);

    let actions = match wendao_graph_page_index_reasoning_table_schema(
        WendaoGraphEvidenceTableKind::Response,
        "page_index_planner_actions",
    ) {
        Ok(schema) => schema,
        Err(error) => panic!("page_index_planner_actions response schema should build: {error}"),
    };
    assert_eq!(
        validate_wendao_graph_page_index_reasoning_response_schema(
            "page_index_planner_actions",
            actions.as_ref(),
        ),
        Ok(())
    );
    assert_eq!(actions.field(3).name(), "action_kind");
    assert_eq!(actions.field(6).name(), "score");
    assert_eq!(actions.field(6).data_type(), &DataType::Float64);
}

#[test]
fn wendao_graph_evidence_schema_validation_accepts_response_tables() {
    let schema = match wendao_graph_evidence_table_schema(
        WendaoGraphEvidenceTableKind::Response,
        "diffusion_scores",
    ) {
        Ok(schema) => schema,
        Err(error) => panic!("diffusion score response schema should build: {error}"),
    };
    assert_eq!(
        validate_wendao_graph_evidence_response_schema("diffusion_scores", schema.as_ref()),
        Ok(())
    );
    assert_eq!(schema.field(0).name(), "node_id");
    assert_eq!(schema.field(6).name(), "iteration_count");
    assert_eq!(schema.field(6).data_type(), &DataType::Int64);
}

#[test]
fn wendao_graph_evidence_schema_validation_rejects_wrong_order_type_and_nullability() {
    let wrong_order = Schema::new(vec![
        Field::new("target_id", DataType::Utf8, false),
        Field::new("source_id", DataType::Utf8, false),
    ]);
    assert_eq!(
        validate_wendao_graph_evidence_request_schema("edges", &wrong_order),
        Err(
            "WendaoGraph evidence table `edges` column 0 must be `source_id`, got `target_id`"
                .to_string()
        )
    );

    let wrong_type = Schema::new(vec![
        Field::new("node_id", DataType::Utf8, false),
        Field::new("weight", DataType::Int64, false),
    ]);
    assert_eq!(
        validate_wendao_graph_evidence_request_schema("seeds", &wrong_type),
        Err(
            "WendaoGraph evidence table `seeds` column `weight` must be Float64, got Int64"
                .to_string()
        )
    );

    let nullable = Schema::new(vec![Field::new("node_id", DataType::Utf8, true)]);
    assert_eq!(
        validate_wendao_graph_evidence_request_schema("nodes", &nullable),
        Err("WendaoGraph evidence table `nodes` column `node_id` must be non-nullable".to_string())
    );
}

#[test]
fn wendao_graph_evidence_schema_validation_rejects_unknown_tables() {
    let schema = Schema::new(vec![Field::new("node_id", DataType::Utf8, false)]);
    assert_eq!(
        validate_wendao_graph_evidence_request_schema("page_index_nodes", &schema),
        Err("unknown WendaoGraph evidence request table `page_index_nodes`".to_string())
    );
    assert_eq!(
        validate_wendao_graph_page_index_reasoning_request_schema("nodes", &schema),
        Err("unknown WendaoGraph evidence PageIndex reasoning request table `nodes`".to_string())
    );
    assert_eq!(
        validate_wendao_graph_evidence_response_schema("nodes", &schema),
        Err("unknown WendaoGraph evidence response table `nodes`".to_string())
    );
}
