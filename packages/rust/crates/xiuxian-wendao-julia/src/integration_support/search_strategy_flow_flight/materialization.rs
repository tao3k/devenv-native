//! Route execution for `SearchStrategyFlow` Flight materialization.

use arrow::record_batch::RecordBatch;
use serde_json::{Value, json};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_ROUTE,
};

use super::client::SearchStrategyFlowFlightClient;
use super::config::SearchStrategyFlowFlightMaterializationConfig;
use super::constants::{GRAPH_HOPS, GRAPH_LIMIT, RELATED_CONTEXT_LIMIT, REPO_SEARCH_LIMIT};
use super::ids::{
    find_node_id_by_anchor_or_title, first_node_id, graph_node_display_id,
    normalized_repo_search_doc_id, projected_page_id, repo_relative_source_path,
};
use super::metadata::{
    populate_graph_neighbors_headers, populate_page_index_headers, populate_repo_search_headers,
    populate_retrieval_context_headers,
};
use super::query::{RepoSearchAttempt, repo_search_attempts_for_route};
use super::rows::{
    decoded_payload_receipt, first_page_index_repo_search_row, first_string, route_receipt,
    route_string, row_count,
};

#[derive(Debug)]
struct SearchStrategyFlowRouteReceipt {
    materialized_rows: usize,
    resolved_page_id: String,
    resolved_node_id: String,
    resolved_graph_node_id: String,
    route_receipts: Vec<Value>,
    decoded_payload_receipts: Vec<Value>,
}

/// Executes all `SearchStrategyFlow` retrieval routes in a JSON trace through a
/// real Arrow Flight endpoint.
///
/// # Errors
///
/// Returns an error when the trace route shape is invalid, the endpoint cannot
/// be reached, a Flight route returns no rows, or Arrow decoding fails.
pub async fn materialize_search_strategy_flow_routes(
    trace: &mut Value,
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> Result<(), String> {
    let routes = trace
        .get_mut("retrievalRoutes")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "SearchStrategyFlow trace missing retrievalRoutes".to_owned())?;
    if routes.is_empty() {
        return Ok(());
    }

    let mut client = SearchStrategyFlowFlightClient::connect(config).await?;
    for route in routes {
        let receipt = materialize_route(&mut client, config, route).await?;
        apply_route_receipt(route, receipt)?;
    }
    Ok(())
}

async fn collect_first_non_empty_repo_search(
    client: &mut SearchStrategyFlowFlightClient,
    config: &SearchStrategyFlowFlightMaterializationConfig,
    attempts: &[RepoSearchAttempt],
    context: &str,
) -> Result<Vec<RecordBatch>, String> {
    let mut attempted = Vec::new();
    for attempt in attempts {
        attempted.push(format!(
            "query=`{}` prefix=`{}`",
            attempt.query, attempt.path_prefix
        ));
        let batches = client
            .collect_route_batches_allow_empty(REPO_SEARCH_ROUTE, context, |metadata| {
                populate_repo_search_headers(
                    metadata,
                    &config.repo_id,
                    attempt.query.as_str(),
                    REPO_SEARCH_LIMIT,
                    attempt.path_prefix.as_str(),
                )
            })
            .await?;
        if first_page_index_repo_search_row(&batches, &config.repo_id, None).is_some() {
            return Ok(batches);
        }
    }

    Err(format!(
        "{context} returned zero page-index-ready decoded rows after {} attempts: {}",
        attempted.len(),
        attempted.join("; ")
    ))
}

async fn materialize_route(
    client: &mut SearchStrategyFlowFlightClient,
    config: &SearchStrategyFlowFlightMaterializationConfig,
    route: &Value,
) -> Result<SearchStrategyFlowRouteReceipt, String> {
    let source_path = route_string(route, "sourcePath")?;
    let heading_anchor = route.get("headingAnchor").and_then(Value::as_str);
    let repo_relative_route_source_path =
        repo_relative_source_path(config.repo_id.as_str(), source_path);
    let repo_search_batches = collect_first_non_empty_repo_search(
        client,
        config,
        &repo_search_attempts_for_route(config.repo_id.as_str(), source_path, heading_anchor),
        "SearchStrategyFlow repo search materialization",
    )
    .await?;
    let (repo_search_path, doc_id) = first_page_index_repo_search_row(
        &repo_search_batches,
        &config.repo_id,
        Some(repo_relative_route_source_path.as_str()),
    )
    .unwrap_or_else(|| (repo_relative_route_source_path.clone(), None));
    let repo_search_path = repo_relative_source_path(config.repo_id.as_str(), &repo_search_path);
    let doc_id = normalized_repo_search_doc_id(
        config.repo_id.as_str(),
        repo_search_path.as_str(),
        doc_id.as_deref(),
    );
    let page_id = projected_page_id(&config.repo_id, &doc_id, &repo_search_path);

    let page_index_batches = client
        .collect_route_batches(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            "SearchStrategyFlow page-index materialization",
            |metadata| populate_page_index_headers(metadata, &config.repo_id, page_id.as_str()),
        )
        .await?;
    let roots_json = first_string(&page_index_batches[0], "rootsJson")?;
    let roots = serde_json::from_str::<Value>(&roots_json)
        .map_err(|error| format!("decode SearchStrategyFlow rootsJson: {error}"))?;
    let node_id = heading_anchor
        .and_then(|anchor| find_node_id_by_anchor_or_title(&roots, anchor))
        .or_else(|| first_node_id(&roots))
        .ok_or_else(|| "SearchStrategyFlow page-index tree did not expose a node id".to_owned())?;
    let graph_node_id = graph_node_display_id(&config.repo_id, &repo_search_path);

    let retrieval_context_batches = client
        .collect_route_batches(
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
            "SearchStrategyFlow retrieval-context materialization",
            |metadata| {
                populate_retrieval_context_headers(
                    metadata,
                    &config.repo_id,
                    page_id.as_str(),
                    node_id.as_str(),
                    RELATED_CONTEXT_LIMIT,
                )
            },
        )
        .await?;

    let graph_batches = client
        .collect_route_batches(
            GRAPH_NEIGHBORS_ROUTE,
            "SearchStrategyFlow graph-neighbor materialization",
            |metadata| {
                populate_graph_neighbors_headers(
                    metadata,
                    graph_node_id.as_str(),
                    "both",
                    GRAPH_HOPS,
                    GRAPH_LIMIT,
                )
            },
        )
        .await?;

    let route_receipts = route_receipts(
        &repo_search_batches,
        &page_index_batches,
        &retrieval_context_batches,
        &graph_batches,
    );
    let decoded_payload_receipts = decoded_payload_receipts(
        repo_search_path.as_str(),
        &repo_search_batches,
        &page_index_batches,
        &retrieval_context_batches,
        &graph_batches,
        node_id.as_str(),
        graph_node_id.as_str(),
    )?;
    let materialized_rows = row_count(&repo_search_batches)
        + row_count(&page_index_batches)
        + row_count(&retrieval_context_batches)
        + row_count(&graph_batches);

    Ok(SearchStrategyFlowRouteReceipt {
        materialized_rows,
        resolved_page_id: page_id,
        resolved_node_id: node_id,
        resolved_graph_node_id: graph_node_id,
        route_receipts,
        decoded_payload_receipts,
    })
}

fn route_receipts(
    repo_search_batches: &[RecordBatch],
    page_index_batches: &[RecordBatch],
    retrieval_context_batches: &[RecordBatch],
    graph_batches: &[RecordBatch],
) -> Vec<Value> {
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
    ]
}

fn decoded_payload_receipts(
    repo_search_path: &str,
    repo_search_batches: &[RecordBatch],
    page_index_batches: &[RecordBatch],
    retrieval_context_batches: &[RecordBatch],
    graph_batches: &[RecordBatch],
    node_id: &str,
    graph_node_id: &str,
) -> Result<Vec<Value>, String> {
    let repo_search_evidence_anchor = format!("path:{repo_search_path}");
    let page_index_evidence_anchor = prefixed_evidence_anchor("node", node_id);
    let retrieval_context_evidence_anchor = format!(
        "node-context:{}",
        first_string(&retrieval_context_batches[0], "nodeId")?
    );
    let graph_evidence_anchor = format!("graph-node:{graph_node_id}");

    Ok(vec![
        decoded_payload_receipt(
            REPO_SEARCH_ROUTE,
            repo_search_batches,
            &[REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_PATH_COLUMN],
            &repo_search_evidence_anchor,
        ),
        decoded_payload_receipt(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            page_index_batches,
            &["pageId", "rootCount", "rootsJson"],
            &page_index_evidence_anchor,
        ),
        decoded_payload_receipt(
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
            retrieval_context_batches,
            &["pageId", "nodeId", "centerJson", "nodeContextJson"],
            &retrieval_context_evidence_anchor,
        ),
        decoded_payload_receipt(
            GRAPH_NEIGHBORS_ROUTE,
            graph_batches,
            &["rowType"],
            &graph_evidence_anchor,
        ),
    ])
}

fn prefixed_evidence_anchor(prefix: &str, value: &str) -> String {
    let prefix = format!("{prefix}:");
    if value.starts_with(prefix.as_str()) {
        value.to_owned()
    } else {
        format!("{prefix}{value}")
    }
}

fn apply_route_receipt(
    route: &mut Value,
    receipt: SearchStrategyFlowRouteReceipt,
) -> Result<(), String> {
    let object = route
        .as_object_mut()
        .ok_or_else(|| "SearchStrategyFlow retrieval route must be an object".to_owned())?;
    object.insert("materializationStatus".to_owned(), json!("executed"));
    object.insert(
        "materializedRows".to_owned(),
        json!(receipt.materialized_rows),
    );
    object.insert("decodedPayloadStatus".to_owned(), json!("decoded"));
    object.insert("resolvedPageId".to_owned(), json!(receipt.resolved_page_id));
    object.insert("resolvedNodeId".to_owned(), json!(receipt.resolved_node_id));
    object.insert(
        "resolvedGraphNodeId".to_owned(),
        json!(receipt.resolved_graph_node_id),
    );
    object.insert(
        "routeReceipts".to_owned(),
        Value::Array(receipt.route_receipts),
    );
    object.insert(
        "decodedPayloadReceipts".to_owned(),
        Value::Array(receipt.decoded_payload_receipts),
    );
    Ok(())
}
