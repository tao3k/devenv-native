use super::{
    DataType, WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_NAMES,
    WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_NAMES, WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_ROUTE, WENDAO_GRAPH_PAGE_INDEX_REASONING_REQUEST_TABLE_NAMES,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_RESPONSE_TABLE_NAMES, WendaoGraphEvidenceTableKind,
    is_wendao_graph_link_evidence_route, wendao_graph_evidence_request_table_contract,
    wendao_graph_evidence_response_table_contract, wendao_graph_link_evidence_route,
    wendao_graph_page_index_reasoning_request_table_contract,
    wendao_graph_page_index_reasoning_table_schema,
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
fn wendao_graph_page_index_reasoning_contracts_match_julia_table_names() {
    assert_eq!(
        WENDAO_GRAPH_PAGE_INDEX_REASONING_REQUEST_TABLE_NAMES,
        ["page_index_nodes", "page_index_edges", "page_index_seeds"]
    );
    assert_eq!(
        WENDAO_GRAPH_PAGE_INDEX_REASONING_RESPONSE_TABLE_NAMES,
        [
            "page_index_nodes",
            "page_index_edges",
            "page_index_seeds",
            "reasoning_frontier",
            "disclosure_trace",
            "page_index_planner_actions",
        ]
    );

    let nodes = match wendao_graph_page_index_reasoning_request_table_contract("page_index_nodes") {
        Ok(contract) => contract,
        Err(error) => panic!("page_index_nodes request contract should exist: {error}"),
    };
    assert_eq!(nodes.kind, WendaoGraphEvidenceTableKind::Request);
    assert!(nodes.required);
    assert_eq!(
        nodes
            .columns
            .iter()
            .map(|column| column.name)
            .collect::<Vec<_>>(),
        vec![
            "node_id",
            "page_id",
            "parent_id",
            "depth",
            "rank",
            "title",
            "summary",
            "line_start",
            "line_end",
            "token_count",
        ]
    );

    let trace = match wendao_graph_page_index_reasoning_table_schema(
        WendaoGraphEvidenceTableKind::Response,
        "disclosure_trace",
    ) {
        Ok(schema) => schema,
        Err(error) => panic!("disclosure_trace response schema should build: {error}"),
    };
    assert_eq!(trace.field(0).name(), "tree_id");
    assert_eq!(trace.field(7).name(), "reason");

    let actions = match wendao_graph_page_index_reasoning_table_schema(
        WendaoGraphEvidenceTableKind::Response,
        "page_index_planner_actions",
    ) {
        Ok(schema) => schema,
        Err(error) => panic!("page_index_planner_actions response schema should build: {error}"),
    };
    assert_eq!(
        actions
            .fields()
            .iter()
            .map(|field| field.name().as_str())
            .collect::<Vec<_>>(),
        vec![
            "tree_id",
            "action_id",
            "source_step_id",
            "action_kind",
            "target_step_id",
            "target_node_id",
            "score",
            "reason",
        ]
    );
    assert_eq!(actions.field(6).data_type(), &DataType::Float64);
}
