use arrow::datatypes::{DataType, Field, Schema};

use super::{
    WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_NAMES, WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_NAMES,
    WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION, WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
    WendaoGraphEvidenceTableKind, is_wendao_graph_link_evidence_route,
    validate_wendao_graph_evidence_request_schema, validate_wendao_graph_evidence_response_schema,
    wendao_graph_evidence_request_table_contract, wendao_graph_evidence_response_table_contract,
    wendao_graph_evidence_table_schema, wendao_graph_link_evidence_route,
};

#[test]
fn wendao_graph_evidence_route_resolves_canonical_path() {
    assert_eq!(
        wendao_graph_link_evidence_route("graph/link/evidence"),
        Ok(WENDAO_GRAPH_LINK_EVIDENCE_ROUTE)
    );
    assert!(is_wendao_graph_link_evidence_route("/graph/link/evidence"));
    assert!(!is_wendao_graph_link_evidence_route(
        "/graph/structural/rerank"
    ));
    assert_eq!(WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION, "v0-draft");
}

#[test]
fn wendao_graph_evidence_contracts_match_julia_table_names() {
    assert_eq!(
        WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_NAMES,
        [
            "nodes",
            "edges",
            "seeds",
            "semantic_neighbors",
            "semantic_overlay",
        ]
    );
    assert_eq!(WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_NAMES.len(), 17);
    assert_eq!(
        WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_NAMES[0],
        "graph_metrics"
    );
    assert_eq!(
        WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_NAMES[16],
        "link_frontier"
    );

    let nodes = match wendao_graph_evidence_request_table_contract("nodes") {
        Ok(contract) => contract,
        Err(error) => panic!("nodes request contract should exist: {error}"),
    };
    assert_eq!(nodes.kind, WendaoGraphEvidenceTableKind::Request);
    assert!(nodes.required);
    assert_eq!(nodes.columns[0].name, "node_id");

    let link_frontier = match wendao_graph_evidence_response_table_contract("link_frontier") {
        Ok(contract) => contract,
        Err(error) => panic!("link_frontier response contract should exist: {error}"),
    };
    assert_eq!(link_frontier.kind, WendaoGraphEvidenceTableKind::Response);
    assert_eq!(link_frontier.columns.len(), 10);
    assert_eq!(link_frontier.columns[9].name, "disclosure_budget");
}

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
        validate_wendao_graph_evidence_response_schema("nodes", &schema),
        Err("unknown WendaoGraph evidence response table `nodes`".to_string())
    );
}
