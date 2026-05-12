//! Route execution for `SearchStrategyFlow` Flight materialization.

use std::time::Instant;

use arrow::record_batch::RecordBatch;
use futures::future::try_join_all;
use serde_json::{Value, json};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_ROUTE,
};

use super::client::SearchStrategyFlowFlightClient;
use super::config::SearchStrategyFlowFlightMaterializationConfig;
use super::constants::{GRAPH_HOPS, GRAPH_LIMIT, RELATED_CONTEXT_LIMIT, REPO_SEARCH_LIMIT};
use super::ids::{
    find_node_id_by_anchor_or_title, first_node_id, graph_node_display_id_candidates,
    normalized_repo_search_doc_id, projected_page_id, repo_relative_source_path,
};
use super::metadata::{
    populate_graph_neighbors_headers, populate_page_index_headers, populate_repo_search_headers,
    populate_retrieval_context_headers,
};
use super::query::{RepoSearchAttempt, repo_search_attempts_for_route};
use super::rows::{
    decoded_payload_receipt, first_page_index_repo_search_row, first_string, route_string,
    row_count, timed_route_receipt,
};

#[derive(Debug)]
struct SearchStrategyFlowRouteReceipt {
    materialized_rows: usize,
    resolved_page_id: String,
    resolved_node_id: String,
    resolved_graph_node_id: Option<String>,
    graph_materialization_status: &'static str,
    repo_search_resolution_warning: Option<String>,
    graph_materialization_warning: Option<String>,
    route_receipts: Vec<Value>,
    decoded_payload_receipts: Vec<Value>,
}

#[derive(Debug)]
struct SearchStrategyFlowRouteTimings {
    repo_search_elapsed_ms: u128,
    page_index_elapsed_ms: u128,
    retrieval_context_elapsed_ms: u128,
    graph_elapsed_ms: u128,
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

    let planned_routes = routes.clone();
    let receipts = try_join_all(planned_routes.into_iter().map(|route| {
        let config = config.clone();
        async move {
            let mut client = SearchStrategyFlowFlightClient::connect(&config).await?;
            materialize_route(&mut client, &config, &route).await
        }
    }))
    .await?;
    for (route, receipt) in routes.iter_mut().zip(receipts) {
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
    let repo_search_started_at = Instant::now();
    let (repo_search_batches, repo_search_resolution_warning) =
        match collect_first_non_empty_repo_search(
            client,
            config,
            &repo_search_attempts_for_route(config.repo_id.as_str(), source_path, heading_anchor),
            "SearchStrategyFlow repo search materialization",
        )
        .await
        {
            Ok(batches) => (batches, None),
            Err(error) => (Vec::new(), Some(error)),
        };
    let repo_search_elapsed_ms = elapsed_ms(repo_search_started_at);
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

    let page_index_started_at = Instant::now();
    let page_index_batches = client
        .collect_route_batches(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            "SearchStrategyFlow page-index materialization",
            |metadata| populate_page_index_headers(metadata, &config.repo_id, page_id.as_str()),
        )
        .await?;
    let page_index_elapsed_ms = elapsed_ms(page_index_started_at);
    let roots_json = first_string(&page_index_batches[0], "rootsJson")?;
    let roots = serde_json::from_str::<Value>(&roots_json)
        .map_err(|error| format!("decode SearchStrategyFlow rootsJson: {error}"))?;
    let node_id = heading_anchor
        .and_then(|anchor| find_node_id_by_anchor_or_title(&roots, anchor))
        .or_else(|| first_node_id(&roots))
        .ok_or_else(|| "SearchStrategyFlow page-index tree did not expose a node id".to_owned())?;
    let graph_node_ids = graph_node_display_id_candidates(&config.repo_id, &repo_search_path);

    let retrieval_context_started_at = Instant::now();
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
    let retrieval_context_elapsed_ms = elapsed_ms(retrieval_context_started_at);

    let graph_started_at = Instant::now();
    let graph_substitute_status = graph_relation_substitute_status(repo_search_path.as_str());
    let (graph_node_id, graph_materialization_status, graph_batches, graph_materialization_warning) =
        match collect_first_available_graph_neighbors(client, &graph_node_ids).await {
            Ok((graph_node_id, graph_batches)) => {
                (Some(graph_node_id), "resolved", graph_batches, None)
            }
            Err(error) if is_graph_node_not_found_error(error.as_str()) => {
                (None, graph_substitute_status, Vec::new(), Some(error))
            }
            Err(error) => return Err(error),
        };
    let graph_elapsed_ms = elapsed_ms(graph_started_at);

    let route_receipts = route_receipts(
        &repo_search_batches,
        &page_index_batches,
        &retrieval_context_batches,
        &graph_batches,
        &SearchStrategyFlowRouteTimings {
            repo_search_elapsed_ms,
            page_index_elapsed_ms,
            retrieval_context_elapsed_ms,
            graph_elapsed_ms,
        },
    );
    let decoded_payload_receipts = decoded_payload_receipts(
        repo_search_path.as_str(),
        &repo_search_batches,
        &page_index_batches,
        &retrieval_context_batches,
        &graph_batches,
        node_id.as_str(),
        graph_node_id.as_deref(),
        graph_materialization_status,
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
        graph_materialization_status,
        repo_search_resolution_warning,
        graph_materialization_warning,
        route_receipts,
        decoded_payload_receipts,
    })
}

async fn collect_first_available_graph_neighbors(
    client: &mut SearchStrategyFlowFlightClient,
    graph_node_ids: &[String],
) -> Result<(String, Vec<RecordBatch>), String> {
    let mut attempted_errors = Vec::new();
    for graph_node_id in graph_node_ids {
        match client
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
            .await
        {
            Ok(batches) => return Ok((graph_node_id.clone(), batches)),
            Err(error) if is_graph_node_not_found_error(error.as_str()) => {
                attempted_errors.push(format!("{graph_node_id}: {error}"));
            }
            Err(error) => return Err(error),
        }
    }

    Err(format!(
        "SearchStrategyFlow graph-neighbor materialization failed for {} graph node candidate(s): {}",
        graph_node_ids.len(),
        attempted_errors.join("; ")
    ))
}

fn is_graph_node_not_found_error(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("graph node") && normalized.contains("not found")
}

fn graph_relation_substitute_status(source_path: &str) -> &'static str {
    if source_path_has_markdown_extension(source_path) {
        "missing"
    } else {
        "structured-code-relation-substitute"
    }
}

fn source_path_has_markdown_extension(source_path: &str) -> bool {
    std::path::Path::new(source_path.trim())
        .extension()
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
}

fn route_receipts(
    repo_search_batches: &[RecordBatch],
    page_index_batches: &[RecordBatch],
    retrieval_context_batches: &[RecordBatch],
    graph_batches: &[RecordBatch],
    timings: &SearchStrategyFlowRouteTimings,
) -> Vec<Value> {
    vec![
        timed_route_receipt(
            REPO_SEARCH_ROUTE,
            repo_search_batches,
            timings.repo_search_elapsed_ms,
        ),
        timed_route_receipt(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            page_index_batches,
            timings.page_index_elapsed_ms,
        ),
        timed_route_receipt(
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
            retrieval_context_batches,
            timings.retrieval_context_elapsed_ms,
        ),
        timed_route_receipt(
            GRAPH_NEIGHBORS_ROUTE,
            graph_batches,
            timings.graph_elapsed_ms,
        ),
    ]
}

fn decoded_payload_receipts(
    repo_search_path: &str,
    repo_search_batches: &[RecordBatch],
    page_index_batches: &[RecordBatch],
    retrieval_context_batches: &[RecordBatch],
    graph_batches: &[RecordBatch],
    node_id: &str,
    graph_node_id: Option<&str>,
    graph_materialization_status: &str,
) -> Result<Vec<Value>, String> {
    let repo_search_evidence_anchor = format!("path:{repo_search_path}");
    let page_index_evidence_anchor = prefixed_evidence_anchor("node", node_id);
    let retrieval_context_evidence_anchor = format!(
        "node-context:{}",
        first_string(&retrieval_context_batches[0], "nodeId")?
    );
    let graph_evidence_anchor =
        graph_evidence_anchor(node_id, graph_node_id, graph_materialization_status);

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

fn graph_evidence_anchor(
    node_id: &str,
    graph_node_id: Option<&str>,
    graph_materialization_status: &str,
) -> String {
    match (graph_materialization_status, graph_node_id) {
        ("resolved", Some(graph_node_id)) => format!("graph-node:{graph_node_id}"),
        ("structured-code-relation-substitute", _) => {
            format!(
                "structured-code-relation:{}",
                prefixed_evidence_anchor("node", node_id)
            )
        }
        _ => "graph-node:missing".to_owned(),
    }
}

fn prefixed_evidence_anchor(prefix: &str, value: &str) -> String {
    let prefix = format!("{prefix}:");
    if value.starts_with(prefix.as_str()) {
        value.to_owned()
    } else {
        format!("{prefix}{value}")
    }
}

fn elapsed_ms(started_at: Instant) -> u128 {
    started_at.elapsed().as_millis()
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
    let graph_materialization_status = receipt.graph_materialization_status;
    if let Some(resolved_graph_node_id) = receipt.resolved_graph_node_id {
        object.insert(
            "resolvedGraphNodeId".to_owned(),
            json!(resolved_graph_node_id),
        );
    }
    object.insert(
        "graphMaterializationStatus".to_owned(),
        json!(graph_materialization_status),
    );
    if let Some(warning) = receipt.graph_materialization_warning {
        object.insert("graphMaterializationWarning".to_owned(), json!(warning));
    }
    if let Some(warning) = receipt.repo_search_resolution_warning {
        object.insert(
            "repoSearchResolutionStatus".to_owned(),
            json!("source-path-fallback"),
        );
        object.insert("repoSearchResolutionWarning".to_owned(), json!(warning));
    } else {
        object.insert(
            "repoSearchResolutionStatus".to_owned(),
            json!("repo-search"),
        );
    }
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
