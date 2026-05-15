use std::sync::Arc;

use crate::studio::arrow_types::{
    LanceArray, LanceRecordBatch, LanceStringArray, LanceUInt64Array,
};
use crate::studio::search_strategy_flow::materialization::{
    RouteDecodedPayloadReceipt, RouteMaterializationReceipt,
    SearchStrategyFlowMaterializationReceipt,
};
use crate::transport::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_ROUTE,
};

use super::{
    collect_route_batches, first_string, make_gateway_state_with_search_strategy_flow_routes,
    populate_graph_neighbors_headers, populate_repo_projected_page_index_tree_headers,
    populate_repo_projected_retrieval_context_headers, populate_repo_search_headers,
};

const REPO_ID: &str = "gateway-sync";
const PAGE_ID: &str =
    "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md";

#[tokio::test]
async fn search_strategy_flow_materializes_native_flight_route_sequence() {
    let fixture = make_gateway_state_with_search_strategy_flow_routes().await;
    let service = crate::studio::build_studio_flight_service(
        Arc::new(fixture.state.studio.search_plane.clone()),
        fixture.state.clone(),
        "v2",
        3,
    )
    .unwrap_or_else(|error| panic!("build materialization Flight service: {error}"));

    let repo_search_batches = collect_route_batches(
        &service,
        REPO_SEARCH_ROUTE,
        "SearchStrategyFlow repo search materialization",
        |metadata| populate_repo_search_headers(metadata, REPO_ID, "solve anchors", 5),
    )
    .await;
    assert!(
        string_values(&repo_search_batches[0], REPO_SEARCH_PATH_COLUMN)
            .iter()
            .any(|path| path.contains("solve")),
        "repo search should materialize a solve-related repository hit"
    );

    let page_index_batches = collect_route_batches(
        &service,
        ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
        "SearchStrategyFlow page-index materialization",
        |metadata| populate_repo_projected_page_index_tree_headers(metadata, REPO_ID, PAGE_ID),
    )
    .await;
    assert_eq!(first_string(&page_index_batches[0], "pageId"), PAGE_ID);
    assert!(first_u64(&page_index_batches[0], "rootCount") > 0);
    let roots_json = first_string(&page_index_batches[0], "rootsJson");
    assert!(
        roots_json.contains("Anchors"),
        "page-index tree should expose section-level anchors for agent traversal"
    );
    let node_id = find_node_id_by_title(
        &serde_json::from_str::<serde_json::Value>(roots_json.as_str())
            .unwrap_or_else(|error| panic!("rootsJson should decode: {error}")),
        "Anchors",
    )
    .unwrap_or_else(|| panic!("page-index tree should contain an Anchors node"));

    let retrieval_context_batches = collect_route_batches(
        &service,
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
        "SearchStrategyFlow retrieval-context materialization",
        |metadata| {
            populate_repo_projected_retrieval_context_headers(
                metadata,
                REPO_ID,
                PAGE_ID,
                Some(node_id.as_str()),
                3,
            );
        },
    )
    .await;
    assert_eq!(
        first_string(&retrieval_context_batches[0], "pageId"),
        PAGE_ID
    );
    assert_eq!(
        first_string(&retrieval_context_batches[0], "nodeId"),
        node_id
    );
    assert!(
        first_string(&retrieval_context_batches[0], "centerJson").contains("Anchors"),
        "retrieval context should preserve requested section content through the center page"
    );
    assert!(
        first_string(&retrieval_context_batches[0], "nodeContextJson").contains("Documentation"),
        "retrieval context should preserve the requested section neighborhood"
    );

    let graph_batches = collect_route_batches(
        &service,
        GRAPH_NEIGHBORS_ROUTE,
        "SearchStrategyFlow graph-neighbor materialization",
        |metadata| {
            populate_graph_neighbors_headers(metadata, "kernel/docs/alpha.md", "both", 1, 20);
        },
    )
    .await;
    assert!(
        string_values(&graph_batches[0], "rowType")
            .iter()
            .any(|row_type| row_type == "node"),
        "graph-neighbors route should materialize node rows"
    );

    assert_decoded_receipt(
        &repo_search_batches,
        &page_index_batches,
        &retrieval_context_batches,
        &graph_batches,
        node_id.as_str(),
    );
}

fn assert_decoded_receipt(
    repo_search_batches: &[LanceRecordBatch],
    page_index_batches: &[LanceRecordBatch],
    retrieval_context_batches: &[LanceRecordBatch],
    graph_batches: &[LanceRecordBatch],
    node_id: &str,
) {
    let receipt = SearchStrategyFlowMaterializationReceipt::executed(
        "studio-flight-proof",
        vec![
            route_receipt(REPO_SEARCH_ROUTE, repo_search_batches),
            route_receipt(
                ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
                page_index_batches,
            ),
            route_receipt(
                ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
                retrieval_context_batches,
            ),
            route_receipt(GRAPH_NEIGHBORS_ROUTE, graph_batches),
        ],
        vec![
            decoded_payload_receipt(
                REPO_SEARCH_ROUTE,
                repo_search_batches,
                vec![REPO_SEARCH_PATH_COLUMN],
                format!(
                    "path:{}",
                    first_string(&repo_search_batches[0], REPO_SEARCH_PATH_COLUMN)
                ),
            ),
            decoded_payload_receipt(
                ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
                page_index_batches,
                vec!["pageId", "rootCount", "rootsJson"],
                format!("node:{node_id}"),
            ),
            decoded_payload_receipt(
                ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
                retrieval_context_batches,
                vec!["pageId", "nodeId", "centerJson", "nodeContextJson"],
                format!(
                    "node-context:{}",
                    first_string(&retrieval_context_batches[0], "nodeId")
                ),
            ),
            decoded_payload_receipt(
                GRAPH_NEIGHBORS_ROUTE,
                graph_batches,
                vec!["rowType"],
                "row-type:node".to_owned(),
            ),
        ],
    );
    assert_eq!(receipt.materialization_status, "executed");
    assert_eq!(receipt.primary_transport, "arrow-flight");
    assert!(!receipt.direct_file_read_allowed);
    assert_eq!(receipt.route_receipts.len(), 4);
    assert_eq!(receipt.decoded_payload_receipts.len(), 4);
    assert!(receipt.materialized_rows >= 4);
    let receipt_json = receipt
        .to_json()
        .unwrap_or_else(|error| panic!("serialize decoded materialization receipt: {error}"));
    assert_eq!(
        receipt_json.get("decodedPayloadStatus"),
        Some(&serde_json::json!("decoded"))
    );
    assert_eq!(
        receipt_json
            .get("decodedPayloadReceipts")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(4)
    );
    assert!(
        serde_json::to_string(&receipt_json)
            .unwrap_or_else(|error| panic!("serialize decoded materialization receipt: {error}"))
            .contains(node_id),
        "decoded receipt should carry the section node anchor"
    );
}

fn route_receipt(route: &'static str, batches: &[LanceRecordBatch]) -> RouteMaterializationReceipt {
    let row_count = batches.iter().map(LanceRecordBatch::num_rows).sum();
    RouteMaterializationReceipt::new(route, row_count)
        .unwrap_or_else(|error| panic!("create route receipt: {error}"))
}

fn decoded_payload_receipt(
    route: &'static str,
    batches: &[LanceRecordBatch],
    decoded_columns: Vec<&'static str>,
    evidence_anchor: String,
) -> RouteDecodedPayloadReceipt {
    let row_count = batches.iter().map(LanceRecordBatch::num_rows).sum();
    RouteDecodedPayloadReceipt::new(
        route,
        row_count,
        decoded_columns.into_iter().map(str::to_string).collect(),
        evidence_anchor,
    )
    .unwrap_or_else(|error| panic!("create decoded payload receipt: {error}"))
}

fn string_values(batch: &LanceRecordBatch, column: &str) -> Vec<String> {
    let array = batch
        .column_by_name(column)
        .unwrap_or_else(|| panic!("missing column `{column}`"))
        .as_any()
        .downcast_ref::<LanceStringArray>()
        .unwrap_or_else(|| panic!("column `{column}` should be utf8"));
    (0..array.len())
        .filter(|index| !array.is_null(*index))
        .map(|index| array.value(index).to_string())
        .collect()
}

fn first_u64(batch: &LanceRecordBatch, column: &str) -> u64 {
    batch
        .column_by_name(column)
        .unwrap_or_else(|| panic!("missing column `{column}`"))
        .as_any()
        .downcast_ref::<LanceUInt64Array>()
        .unwrap_or_else(|| panic!("column `{column}` should be uint64"))
        .value(0)
}

fn find_node_id_by_title(value: &serde_json::Value, title: &str) -> Option<String> {
    match value {
        serde_json::Value::Array(nodes) => nodes
            .iter()
            .find_map(|node| find_node_id_by_title(node, title)),
        serde_json::Value::Object(node) => {
            if node
                .get("title")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|node_title| node_title == title)
            {
                return node
                    .get("node_id")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string);
            }
            node.get("children")
                .and_then(|children| find_node_id_by_title(children, title))
        }
        _ => None,
    }
}
