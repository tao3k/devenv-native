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
use crate::integration_support::search_strategy_flow_candidates::CODE_INTELLIGENCE_CANDIDATE_SOURCE;

#[derive(Debug)]
struct SearchStrategyFlowRouteReceipt {
    materialized_rows: usize,
    resolved_page_id: String,
    resolved_node_id: String,
    resolved_graph_node_id: Option<String>,
    graph_materialization_status: &'static str,
    repo_search_resolution_status: &'static str,
    repo_search_resolution_warning: Option<String>,
    page_index_materialization_warning: Option<String>,
    graph_materialization_warning: Option<String>,
    route_receipts: Vec<Value>,
    decoded_payload_receipts: Vec<Value>,
}

#[derive(Debug)]
struct SearchStrategyFlowRouteTimings {
    repo_search: u128,
    page_index: u128,
    retrieval_context: u128,
    graph: u128,
}

#[derive(Debug)]
struct RepoSearchMaterialization {
    batches: Vec<RecordBatch>,
    path: String,
    page_id: String,
    graph_substitute_status: &'static str,
    resolution_status: &'static str,
    resolution_warning: Option<String>,
    elapsed_ms: u128,
}

#[derive(Debug)]
struct PageIndexMaterialization {
    batches: Vec<RecordBatch>,
    node_id: String,
    elapsed_ms: u128,
}

#[derive(Debug)]
struct RetrievalContextMaterialization {
    batches: Vec<RecordBatch>,
    elapsed_ms: u128,
}

#[derive(Debug)]
struct GraphMaterialization {
    node_id: Option<String>,
    status: &'static str,
    batches: Vec<RecordBatch>,
    warning: Option<String>,
    elapsed_ms: u128,
}

#[derive(Debug)]
struct StructuredCodeRelationSubstituteReceiptInput<'a> {
    repo_search_batches: &'a [RecordBatch],
    repo_search_path: &'a str,
    page_id: String,
    heading_anchor: Option<&'a str>,
    repo_search_resolution_status: &'static str,
    repo_search_resolution_warning: Option<String>,
    timings: SearchStrategyFlowRouteTimings,
    page_index_materialization_warning: String,
}

#[derive(Clone, Copy)]
struct DecodedPayloadReceiptInput<'a> {
    repo_search_path: &'a str,
    repo_search_batches: &'a [RecordBatch],
    page_index_batches: &'a [RecordBatch],
    retrieval_context_batches: &'a [RecordBatch],
    graph_batches: &'a [RecordBatch],
    node_id: &'a str,
    graph_node_id: Option<&'a str>,
    graph_materialization_status: &'a str,
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
    let repo_search =
        materialize_repo_search(client, config, route, source_path, heading_anchor).await?;
    if should_skip_page_index_for_structured_code_relation(
        route,
        &repo_search.batches,
        repo_search.graph_substitute_status,
    ) {
        return structured_code_relation_receipt_for_repo_search(
            &repo_search,
            heading_anchor,
            SearchStrategyFlowRouteTimings {
                repo_search: repo_search.elapsed_ms,
                page_index: 0,
                retrieval_context: 0,
                graph: 0,
            },
            "SearchStrategyFlow page-index materialization skipped for structured code relation substitute route class"
                .to_owned(),
        );
    }

    let page_index = match materialize_page_index(client, config, &repo_search, heading_anchor)
        .await
    {
        Ok(page_index) => page_index,
        Err(error)
            if repo_search.graph_substitute_status == "structured-code-relation-substitute"
                && is_projected_page_not_found_error(error.message.as_str()) =>
        {
            return structured_code_relation_receipt_for_repo_search(
                &repo_search,
                heading_anchor,
                SearchStrategyFlowRouteTimings {
                    repo_search: repo_search.elapsed_ms,
                    page_index: error.elapsed_ms,
                    retrieval_context: 0,
                    graph: 0,
                },
                format!(
                    "SearchStrategyFlow page-index materialization skipped for structured code relation substitute: {}",
                    error.message
                ),
            );
        }
        Err(error) => return Err(error.message),
    };

    let retrieval_context =
        materialize_retrieval_context(client, config, &repo_search.page_id, &page_index.node_id)
            .await?;
    let graph_node_ids = graph_node_display_id_candidates(&config.repo_id, &repo_search.path);
    let graph =
        materialize_graph_neighbors(client, &graph_node_ids, repo_search.graph_substitute_status)
            .await?;
    let timings = SearchStrategyFlowRouteTimings {
        repo_search: repo_search.elapsed_ms,
        page_index: page_index.elapsed_ms,
        retrieval_context: retrieval_context.elapsed_ms,
        graph: graph.elapsed_ms,
    };
    let route_receipts = route_receipts(
        &repo_search.batches,
        &page_index.batches,
        &retrieval_context.batches,
        &graph.batches,
        &timings,
    );
    let decoded_payload_receipts = decoded_payload_receipts(&DecodedPayloadReceiptInput {
        repo_search_path: repo_search.path.as_str(),
        repo_search_batches: &repo_search.batches,
        page_index_batches: &page_index.batches,
        retrieval_context_batches: &retrieval_context.batches,
        graph_batches: &graph.batches,
        node_id: page_index.node_id.as_str(),
        graph_node_id: graph.node_id.as_deref(),
        graph_materialization_status: graph.status,
    })?;
    let materialized_rows = row_count(&repo_search.batches)
        + row_count(&page_index.batches)
        + row_count(&retrieval_context.batches)
        + row_count(&graph.batches);

    Ok(SearchStrategyFlowRouteReceipt {
        materialized_rows,
        resolved_page_id: repo_search.page_id,
        resolved_node_id: page_index.node_id,
        resolved_graph_node_id: graph.node_id,
        graph_materialization_status: graph.status,
        repo_search_resolution_status: repo_search.resolution_status,
        repo_search_resolution_warning: repo_search.resolution_warning,
        page_index_materialization_warning: None,
        graph_materialization_warning: graph.warning,
        route_receipts,
        decoded_payload_receipts,
    })
}

#[derive(Debug)]
struct MaterializationErrorWithTiming {
    message: String,
    elapsed_ms: u128,
}

async fn materialize_page_index(
    client: &mut SearchStrategyFlowFlightClient,
    config: &SearchStrategyFlowFlightMaterializationConfig,
    repo_search: &RepoSearchMaterialization,
    heading_anchor: Option<&str>,
) -> Result<PageIndexMaterialization, MaterializationErrorWithTiming> {
    let page_index_started_at = Instant::now();
    let page_index_result = client
        .collect_route_batches(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            "SearchStrategyFlow page-index materialization",
            |metadata| {
                populate_page_index_headers(metadata, &config.repo_id, repo_search.page_id.as_str())
            },
        )
        .await;
    let elapsed_ms = elapsed_ms(page_index_started_at);
    let batches = page_index_result.map_err(|message| MaterializationErrorWithTiming {
        message,
        elapsed_ms,
    })?;
    let roots_json = first_string(&batches[0], "rootsJson").map_err(|message| {
        MaterializationErrorWithTiming {
            message,
            elapsed_ms,
        }
    })?;
    let roots = serde_json::from_str::<Value>(&roots_json).map_err(|error| {
        MaterializationErrorWithTiming {
            message: format!("decode SearchStrategyFlow rootsJson: {error}"),
            elapsed_ms,
        }
    })?;
    let node_id = heading_anchor
        .and_then(|anchor| find_node_id_by_anchor_or_title(&roots, anchor))
        .or_else(|| first_node_id(&roots))
        .ok_or_else(|| MaterializationErrorWithTiming {
            message: "SearchStrategyFlow page-index tree did not expose a node id".to_owned(),
            elapsed_ms,
        })?;
    Ok(PageIndexMaterialization {
        batches,
        node_id,
        elapsed_ms,
    })
}

async fn materialize_retrieval_context(
    client: &mut SearchStrategyFlowFlightClient,
    config: &SearchStrategyFlowFlightMaterializationConfig,
    page_id: &str,
    node_id: &str,
) -> Result<RetrievalContextMaterialization, String> {
    let retrieval_context_started_at = Instant::now();
    let batches = client
        .collect_route_batches(
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
            "SearchStrategyFlow retrieval-context materialization",
            |metadata| {
                populate_retrieval_context_headers(
                    metadata,
                    &config.repo_id,
                    page_id,
                    node_id,
                    RELATED_CONTEXT_LIMIT,
                )
            },
        )
        .await?;
    Ok(RetrievalContextMaterialization {
        batches,
        elapsed_ms: elapsed_ms(retrieval_context_started_at),
    })
}

async fn materialize_graph_neighbors(
    client: &mut SearchStrategyFlowFlightClient,
    graph_node_ids: &[String],
    graph_substitute_status: &'static str,
) -> Result<GraphMaterialization, String> {
    let graph_started_at = Instant::now();
    let (node_id, status, batches, warning) =
        match collect_first_available_graph_neighbors(client, graph_node_ids).await {
            Ok((node_id, batches)) => (Some(node_id), "resolved", batches, None),
            Err(error) if is_graph_node_not_found_error(error.as_str()) => {
                (None, graph_substitute_status, Vec::new(), Some(error))
            }
            Err(error) => return Err(error),
        };
    Ok(GraphMaterialization {
        node_id,
        status,
        batches,
        warning,
        elapsed_ms: elapsed_ms(graph_started_at),
    })
}

fn structured_code_relation_receipt_for_repo_search(
    repo_search: &RepoSearchMaterialization,
    heading_anchor: Option<&str>,
    timings: SearchStrategyFlowRouteTimings,
    page_index_materialization_warning: String,
) -> Result<SearchStrategyFlowRouteReceipt, String> {
    structured_code_relation_substitute_receipt(StructuredCodeRelationSubstituteReceiptInput {
        repo_search_batches: &repo_search.batches,
        repo_search_path: repo_search.path.as_str(),
        page_id: repo_search.page_id.clone(),
        heading_anchor,
        repo_search_resolution_status: repo_search.resolution_status,
        repo_search_resolution_warning: repo_search.resolution_warning.clone(),
        timings,
        page_index_materialization_warning,
    })
}

async fn materialize_repo_search(
    client: &mut SearchStrategyFlowFlightClient,
    config: &SearchStrategyFlowFlightMaterializationConfig,
    route: &Value,
    source_path: &str,
    heading_anchor: Option<&str>,
) -> Result<RepoSearchMaterialization, String> {
    let route_source_path = repo_relative_source_path(config.repo_id.as_str(), source_path);
    if should_use_structured_source_path_for_repo_search(route, source_path) {
        return Ok(repo_search_from_structured_source_path(
            config,
            route_source_path,
        ));
    }

    let repo_search_started_at = Instant::now();
    let (batches, resolution_warning) = match collect_first_non_empty_repo_search(
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
    let elapsed_ms = elapsed_ms(repo_search_started_at);
    let resolution_status = if resolution_warning.is_some() {
        "source-path-fallback"
    } else {
        "repo-search"
    };
    Ok(repo_search_from_batches(
        config,
        batches,
        route_source_path.as_str(),
        resolution_status,
        resolution_warning,
        elapsed_ms,
    ))
}

fn repo_search_from_structured_source_path(
    config: &SearchStrategyFlowFlightMaterializationConfig,
    source_path: String,
) -> RepoSearchMaterialization {
    let page_id = projected_page_id_from_source_path(config, source_path.as_str(), None);
    RepoSearchMaterialization {
        batches: Vec::new(),
        graph_substitute_status: graph_relation_substitute_status(source_path.as_str()),
        path: source_path,
        page_id,
        resolution_status: "structured-source-path",
        resolution_warning: Some(
            "SearchStrategyFlow repo-search materialization skipped for structured source-path"
                .to_owned(),
        ),
        elapsed_ms: 0,
    }
}

fn repo_search_from_batches(
    config: &SearchStrategyFlowFlightMaterializationConfig,
    batches: Vec<RecordBatch>,
    route_source_path: &str,
    resolution_status: &'static str,
    resolution_warning: Option<String>,
    elapsed_ms: u128,
) -> RepoSearchMaterialization {
    let (path, doc_id) =
        first_page_index_repo_search_row(&batches, &config.repo_id, Some(route_source_path))
            .unwrap_or_else(|| (route_source_path.to_owned(), None));
    let path = repo_relative_source_path(config.repo_id.as_str(), &path);
    let page_id = projected_page_id_from_source_path(config, path.as_str(), doc_id.as_deref());
    RepoSearchMaterialization {
        graph_substitute_status: graph_relation_substitute_status(path.as_str()),
        batches,
        path,
        page_id,
        resolution_status,
        resolution_warning,
        elapsed_ms,
    }
}

fn projected_page_id_from_source_path(
    config: &SearchStrategyFlowFlightMaterializationConfig,
    source_path: &str,
    doc_id: Option<&str>,
) -> String {
    let doc_id = normalized_repo_search_doc_id(config.repo_id.as_str(), source_path, doc_id);
    projected_page_id(&config.repo_id, &doc_id, source_path)
}

fn structured_code_relation_substitute_receipt(
    input: StructuredCodeRelationSubstituteReceiptInput<'_>,
) -> Result<SearchStrategyFlowRouteReceipt, String> {
    let resolved_node_id = input
        .heading_anchor
        .filter(|anchor| !anchor.trim().is_empty())
        .map_or_else(|| input.repo_search_path.to_owned(), str::to_owned);
    let empty_batches = Vec::new();
    let route_receipts = route_receipts(
        input.repo_search_batches,
        &empty_batches,
        &empty_batches,
        &empty_batches,
        &input.timings,
    );
    let decoded_payload_receipts = decoded_payload_receipts(&DecodedPayloadReceiptInput {
        repo_search_path: input.repo_search_path,
        repo_search_batches: input.repo_search_batches,
        page_index_batches: &empty_batches,
        retrieval_context_batches: &empty_batches,
        graph_batches: &empty_batches,
        node_id: resolved_node_id.as_str(),
        graph_node_id: None,
        graph_materialization_status: "structured-code-relation-substitute",
    })?;
    Ok(SearchStrategyFlowRouteReceipt {
        materialized_rows: row_count(input.repo_search_batches),
        resolved_page_id: input.page_id,
        resolved_node_id,
        resolved_graph_node_id: None,
        graph_materialization_status: "structured-code-relation-substitute",
        repo_search_resolution_status: input.repo_search_resolution_status,
        repo_search_resolution_warning: input.repo_search_resolution_warning,
        page_index_materialization_warning: Some(input.page_index_materialization_warning),
        graph_materialization_warning: Some(
            "SearchStrategyFlow graph-neighbor materialization skipped for structured code relation substitute"
                .to_owned(),
        ),
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

fn is_projected_page_not_found_error(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("projected page") && normalized.contains("not found")
}

fn should_skip_page_index_for_structured_code_relation(
    route: &Value,
    repo_search_batches: &[RecordBatch],
    graph_substitute_status: &str,
) -> bool {
    graph_substitute_status == "structured-code-relation-substitute"
        && !repo_search_batches.is_empty()
        && route.get("candidateInputSource").and_then(Value::as_str)
            == Some("rust-code-intelligence-inventory")
        && route.get("evidenceKind").and_then(Value::as_str) == Some("link_graph_dependency_path")
}

fn should_use_structured_source_path_for_repo_search(route: &Value, source_path: &str) -> bool {
    route.get("candidateInputSource").and_then(Value::as_str)
        == Some(CODE_INTELLIGENCE_CANDIDATE_SOURCE)
        && source_path_has_markdown_extension(source_path)
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
        timed_route_receipt(REPO_SEARCH_ROUTE, repo_search_batches, timings.repo_search),
        timed_route_receipt(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            page_index_batches,
            timings.page_index,
        ),
        timed_route_receipt(
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
            retrieval_context_batches,
            timings.retrieval_context,
        ),
        timed_route_receipt(GRAPH_NEIGHBORS_ROUTE, graph_batches, timings.graph),
    ]
}

fn decoded_payload_receipts(input: &DecodedPayloadReceiptInput<'_>) -> Result<Vec<Value>, String> {
    let repo_search_evidence_anchor = format!("path:{}", input.repo_search_path);
    let graph_evidence_anchor = graph_evidence_anchor(
        input.node_id,
        input.graph_node_id,
        input.graph_materialization_status,
    );

    let mut receipts = vec![decoded_payload_receipt(
        REPO_SEARCH_ROUTE,
        input.repo_search_batches,
        &[REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_PATH_COLUMN],
        &repo_search_evidence_anchor,
    )];
    if !input.page_index_batches.is_empty() {
        receipts.push(decoded_payload_receipt(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            input.page_index_batches,
            &["pageId", "rootCount", "rootsJson"],
            &prefixed_evidence_anchor("node", input.node_id),
        ));
    }
    if !input.retrieval_context_batches.is_empty() {
        receipts.push(decoded_payload_receipt(
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
            input.retrieval_context_batches,
            &["pageId", "nodeId", "centerJson", "nodeContextJson"],
            &format!(
                "node-context:{}",
                first_string(&input.retrieval_context_batches[0], "nodeId")?
            ),
        ));
    }
    receipts.push(decoded_payload_receipt(
        GRAPH_NEIGHBORS_ROUTE,
        input.graph_batches,
        &["rowType"],
        &graph_evidence_anchor,
    ));
    Ok(receipts)
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
    if let Some(warning) = receipt.page_index_materialization_warning {
        object.insert("pageIndexMaterializationWarning".to_owned(), json!(warning));
    }
    object.insert(
        "repoSearchResolutionStatus".to_owned(),
        json!(receipt.repo_search_resolution_status),
    );
    if let Some(warning) = receipt.repo_search_resolution_warning {
        object.insert("repoSearchResolutionWarning".to_owned(), json!(warning));
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
