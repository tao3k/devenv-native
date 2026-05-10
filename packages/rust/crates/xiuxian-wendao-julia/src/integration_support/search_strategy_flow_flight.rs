//! Arrow Flight materialization for Rust-owned `SearchStrategyFlow` routes.

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use super::search_strategy_flow_candidates::{
    FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE, SearchStrategyFlowCandidateInput,
    SearchStrategyFlowCandidateInputBatch, SearchStrategyFlowRepoSearchHit,
    search_strategy_flow_candidate_input_batch,
    search_strategy_flow_candidate_input_from_repo_search_hit,
};
use arrow::array::{Array, Float64Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch;
use arrow_flight::decode::FlightRecordBatchStream;
use arrow_flight::flight_service_client::FlightServiceClient;
use arrow_flight::{FlightDescriptor, Ticket};
use futures::TryStreamExt;
use serde_json::{Value, json};
use tonic::Request;
use tonic::metadata::MetadataMap;
use tonic::transport::Endpoint;
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_BEST_SECTION_COLUMN, REPO_SEARCH_DOC_ID_COLUMN,
    REPO_SEARCH_NAVIGATION_LINE_COLUMN, REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
    REPO_SEARCH_NAVIGATION_PATH_COLUMN, REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_ROUTE,
    REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TITLE_COLUMN, WENDAO_GRAPH_DIRECTION_HEADER,
    WENDAO_GRAPH_HOPS_HEADER, WENDAO_GRAPH_LIMIT_HEADER, WENDAO_GRAPH_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
    WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER, WENDAO_REPO_SEARCH_LIMIT_HEADER,
    WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER, WENDAO_REPO_SEARCH_QUERY_HEADER,
    WENDAO_REPO_SEARCH_REPO_HEADER, WENDAO_SCHEMA_VERSION_HEADER, flight_descriptor_path,
};

const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const REPO_SEARCH_LIMIT: usize = 10;
const MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS: usize = 32;
const MAX_FLIGHT_DISCOVERY_CANDIDATES: usize = 12;
const RELATED_CONTEXT_LIMIT: usize = 5;
const GRAPH_HOPS: usize = 2;
const GRAPH_LIMIT: usize = 50;
const MARKDOWN_LANGUAGE_FILTER: &str = "markdown";

/// Network endpoint settings for Rust-owned `SearchStrategyFlow` Flight
/// materialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchStrategyFlowFlightMaterializationConfig {
    /// Base URL of the Studio Arrow Flight endpoint.
    pub base_url: String,
    /// Repo id used by native Wendao Flight query contracts.
    pub repo_id: String,
    /// Per-route request timeout in seconds.
    pub timeout_seconds: u64,
}

impl SearchStrategyFlowFlightMaterializationConfig {
    /// Creates a Flight materialization config.
    ///
    /// # Errors
    ///
    /// Returns an error when the endpoint or repo id is blank.
    pub fn new(base_url: impl Into<String>, repo_id: impl Into<String>) -> Result<Self, String> {
        let base_url = base_url.into();
        if base_url.trim().is_empty() {
            return Err("SearchStrategyFlow Flight base URL must not be blank".to_owned());
        }
        let repo_id = repo_id.into();
        if repo_id.trim().is_empty() {
            return Err("SearchStrategyFlow Flight repo id must not be blank".to_owned());
        }
        Ok(Self {
            base_url,
            repo_id,
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
        })
    }

    /// Sets the per-route request timeout.
    #[must_use]
    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds.max(1);
        self
    }
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

pub(crate) async fn search_strategy_flow_candidate_input_batch_from_repo_search(
    intent: &str,
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> Result<SearchStrategyFlowCandidateInputBatch, String> {
    let mut client = SearchStrategyFlowFlightClient::connect(config).await?;
    let attempts = candidate_discovery_queries(intent);
    let mut attempted = Vec::new();
    let mut seen = HashSet::<(String, String)>::new();
    let mut merged_candidates = Vec::new();
    for attempt in attempts
        .iter()
        .take(MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS)
    {
        attempted.push(format!(
            "query=`{}` prefix=`{}`",
            attempt.query, attempt.path_prefix
        ));
        let batches = client
            .collect_route_batches_allow_empty(
                REPO_SEARCH_ROUTE,
                "SearchStrategyFlow repo-search candidate discovery",
                |metadata| {
                    populate_repo_search_headers(
                        metadata,
                        &config.repo_id,
                        attempt.query.as_str(),
                        REPO_SEARCH_LIMIT,
                        attempt.path_prefix.as_str(),
                    )
                },
            )
            .await?;
        for candidate in repo_relative_candidate_inputs(
            config.repo_id.as_str(),
            repo_search_batches_to_candidate_inputs(&batches),
        ) {
            let key = (
                candidate.relative_path.clone(),
                candidate.heading_anchor.clone(),
            );
            if seen.insert(key) {
                merged_candidates.push(candidate);
            }
            if merged_candidates.len() >= MAX_FLIGHT_DISCOVERY_CANDIDATES {
                return Ok(search_strategy_flow_candidate_input_batch(
                    FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE,
                    &merged_candidates,
                ));
            }
        }
    }
    if !merged_candidates.is_empty() {
        return Ok(search_strategy_flow_candidate_input_batch(
            FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE,
            &merged_candidates,
        ));
    }

    Err(format!(
        "SearchStrategyFlow repo-search candidate discovery returned zero page-index-ready candidate rows after {} attempts: {}",
        attempted.len(),
        attempted.join("; ")
    ))
}

#[derive(Debug)]
struct SearchStrategyFlowRouteReceipt {
    materialized_rows: usize,
    resolved_page_id: String,
    resolved_node_id: String,
    resolved_graph_node_id: String,
    route_receipts: Vec<Value>,
    decoded_payload_receipts: Vec<Value>,
}

struct SearchStrategyFlowFlightClient {
    client: FlightServiceClient<tonic::transport::Channel>,
}

impl SearchStrategyFlowFlightClient {
    async fn connect(
        config: &SearchStrategyFlowFlightMaterializationConfig,
    ) -> Result<Self, String> {
        let endpoint = Endpoint::from_shared(config.base_url.clone())
            .map_err(|error| format!("create SearchStrategyFlow Flight endpoint: {error}"))?
            .timeout(Duration::from_secs(config.timeout_seconds));
        let channel = endpoint
            .connect()
            .await
            .map_err(|error| format!("connect SearchStrategyFlow Flight endpoint: {error}"))?;
        Ok(Self {
            client: FlightServiceClient::new(channel),
        })
    }

    async fn collect_route_batches<F>(
        &mut self,
        route: &str,
        context: &str,
        populate: F,
    ) -> Result<Vec<RecordBatch>, String>
    where
        F: Fn(&mut MetadataMap) -> Result<(), String>,
    {
        self.collect_route_batches_with_row_policy(route, context, true, populate)
            .await
    }

    async fn collect_route_batches_allow_empty<F>(
        &mut self,
        route: &str,
        context: &str,
        populate: F,
    ) -> Result<Vec<RecordBatch>, String>
    where
        F: Fn(&mut MetadataMap) -> Result<(), String>,
    {
        self.collect_route_batches_with_row_policy(route, context, false, populate)
            .await
    }

    async fn collect_route_batches_with_row_policy<F>(
        &mut self,
        route: &str,
        context: &str,
        require_rows: bool,
        populate: F,
    ) -> Result<Vec<RecordBatch>, String>
    where
        F: Fn(&mut MetadataMap) -> Result<(), String>,
    {
        let descriptor_path = flight_descriptor_path(route)
            .map_err(|error| format!("{context} descriptor path: {error}"))?;
        let mut info_request = Request::new(FlightDescriptor::new_path(descriptor_path));
        populate(info_request.metadata_mut())?;
        let flight_info = self
            .client
            .get_flight_info(info_request)
            .await
            .map_err(|error| format!("{context} get_flight_info failed: {error}"))?
            .into_inner();
        let ticket = flight_info
            .endpoint
            .first()
            .and_then(|endpoint| endpoint.ticket.clone())
            .ok_or_else(|| format!("{context} did not return a Flight ticket"))?;
        let mut get_request = Request::new(Ticket {
            ticket: ticket.ticket,
        });
        populate(get_request.metadata_mut())?;
        let response = self
            .client
            .do_get(get_request)
            .await
            .map_err(|error| format!("{context} do_get failed: {error}"))?
            .into_inner()
            .map_err(arrow_flight::error::FlightError::from);
        let mut batch_stream = FlightRecordBatchStream::new_from_flight_data(response);
        let mut batches = Vec::new();
        while let Some(batch) = batch_stream
            .try_next()
            .await
            .map_err(|error| format!("{context} Arrow decode failed: {error}"))?
        {
            batches.push(batch);
        }
        if require_rows && row_count(&batches) == 0 {
            return Err(format!("{context} returned zero decoded rows"));
        }
        Ok(batches)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RepoSearchAttempt {
    query: String,
    path_prefix: String,
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

fn populate_repo_search_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    query_text: &str,
    limit: usize,
    path_prefix: &str,
) -> Result<(), String> {
    populate_schema_headers(metadata)?;
    insert_header(metadata, WENDAO_REPO_SEARCH_REPO_HEADER, repo_id)?;
    insert_header(metadata, WENDAO_REPO_SEARCH_QUERY_HEADER, query_text)?;
    insert_header(
        metadata,
        WENDAO_REPO_SEARCH_LIMIT_HEADER,
        &limit.to_string(),
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER,
        MARKDOWN_LANGUAGE_FILTER,
    )?;
    if path_prefix.trim().is_empty() {
        return Ok(());
    }
    insert_header(
        metadata,
        WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER,
        path_prefix,
    )
}

fn populate_page_index_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    page_id: &str,
) -> Result<(), String> {
    populate_schema_headers(metadata)?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
        repo_id,
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
        page_id,
    )
}

fn populate_retrieval_context_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    page_id: &str,
    node_id: &str,
    related_limit: usize,
) -> Result<(), String> {
    populate_schema_headers(metadata)?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
        repo_id,
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
        page_id,
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
        node_id,
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
        &related_limit.to_string(),
    )
}

fn populate_graph_neighbors_headers(
    metadata: &mut MetadataMap,
    node_id: &str,
    direction: &str,
    hops: usize,
    limit: usize,
) -> Result<(), String> {
    populate_schema_headers(metadata)?;
    insert_header(metadata, WENDAO_GRAPH_NODE_ID_HEADER, node_id)?;
    insert_header(metadata, WENDAO_GRAPH_DIRECTION_HEADER, direction)?;
    insert_header(metadata, WENDAO_GRAPH_HOPS_HEADER, &hops.to_string())?;
    insert_header(metadata, WENDAO_GRAPH_LIMIT_HEADER, &limit.to_string())
}

fn populate_schema_headers(metadata: &mut MetadataMap) -> Result<(), String> {
    insert_header(metadata, WENDAO_SCHEMA_VERSION_HEADER, "v2")
}

fn insert_header(
    metadata: &mut MetadataMap,
    header: &'static str,
    value: &str,
) -> Result<(), String> {
    metadata.insert(
        header,
        value
            .parse()
            .map_err(|error| format!("invalid metadata value for `{header}`: {error}"))?,
    );
    Ok(())
}

fn route_receipt(route: &str, batches: &[RecordBatch]) -> Value {
    json!({
        "route": route,
        "rowCount": row_count(batches),
    })
}

fn decoded_payload_receipt(
    route: &str,
    batches: &[RecordBatch],
    decoded_columns: &[&str],
    evidence_anchor: &str,
) -> Value {
    json!({
        "route": route,
        "rowCount": row_count(batches),
        "decodedColumns": decoded_columns,
        "evidenceAnchor": evidence_anchor,
    })
}

fn row_count(batches: &[RecordBatch]) -> usize {
    batches.iter().map(RecordBatch::num_rows).sum()
}

fn repo_search_batches_to_candidate_inputs(
    batches: &[RecordBatch],
) -> Vec<SearchStrategyFlowCandidateInput> {
    let mut candidates = Vec::new();
    for batch in batches {
        for row_index in 0..batch.num_rows() {
            let Some(relative_path) = string_at(batch, REPO_SEARCH_PATH_COLUMN, row_index)
                .or_else(|| string_at(batch, REPO_SEARCH_NAVIGATION_PATH_COLUMN, row_index))
            else {
                continue;
            };
            if !is_page_index_candidate_path(relative_path.as_str()) {
                continue;
            }
            let title = string_at(batch, REPO_SEARCH_TITLE_COLUMN, row_index);
            let best_section = string_at(batch, REPO_SEARCH_BEST_SECTION_COLUMN, row_index);
            let hit = SearchStrategyFlowRepoSearchHit {
                relative_path: relative_path.as_str(),
                title: title.as_deref(),
                best_section: best_section.as_deref(),
                line_start: usize_at(batch, REPO_SEARCH_NAVIGATION_LINE_COLUMN, row_index),
                line_end: usize_at(batch, REPO_SEARCH_NAVIGATION_LINE_END_COLUMN, row_index),
                score: f64_at(batch, REPO_SEARCH_SCORE_COLUMN, row_index),
            };
            candidates.push(search_strategy_flow_candidate_input_from_repo_search_hit(
                &hit,
            ));
        }
    }
    candidates
}

fn repo_relative_candidate_inputs(
    repo_id: &str,
    candidates: Vec<SearchStrategyFlowCandidateInput>,
) -> Vec<SearchStrategyFlowCandidateInput> {
    candidates
        .into_iter()
        .map(|mut candidate| {
            candidate.relative_path = repo_relative_source_path(repo_id, &candidate.relative_path);
            candidate
        })
        .collect()
}

fn is_page_index_candidate_path(path: &str) -> bool {
    has_markdown_extension(path)
}

fn first_page_index_repo_search_row(
    batches: &[RecordBatch],
    repo_id: &str,
    preferred_source_path: Option<&str>,
) -> Option<(String, Option<String>)> {
    if let Some(preferred_source_path) = preferred_source_path {
        let preferred_source_path = repo_relative_source_path(repo_id, preferred_source_path);
        return first_page_index_repo_search_row_matching(batches, repo_id, |path| {
            path == preferred_source_path
        });
    }
    first_page_index_repo_search_row_matching(batches, repo_id, |_| true)
}

fn first_page_index_repo_search_row_matching(
    batches: &[RecordBatch],
    repo_id: &str,
    matches_path: impl Fn(&str) -> bool,
) -> Option<(String, Option<String>)> {
    for batch in batches {
        for row_index in 0..batch.num_rows() {
            let Some(path) = string_at(batch, REPO_SEARCH_PATH_COLUMN, row_index)
                .or_else(|| string_at(batch, REPO_SEARCH_NAVIGATION_PATH_COLUMN, row_index))
            else {
                continue;
            };
            let repo_relative_path = repo_relative_source_path(repo_id, path.as_str());
            if is_page_index_candidate_path(repo_relative_path.as_str())
                && matches_path(repo_relative_path.as_str())
            {
                return Some((
                    repo_relative_path,
                    string_at(batch, REPO_SEARCH_DOC_ID_COLUMN, row_index),
                ));
            }
        }
    }
    None
}

fn first_string(batch: &RecordBatch, column: &str) -> Result<String, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("missing column `{column}`"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("column `{column}` should be utf8"))?;
    (0..array.len())
        .find(|index| !array.is_null(*index))
        .map(|index| array.value(index).to_owned())
        .ok_or_else(|| format!("column `{column}` should contain a non-null value"))
}

fn string_at(batch: &RecordBatch, column: &str, row_index: usize) -> Option<String> {
    let array = batch
        .column_by_name(column)?
        .as_any()
        .downcast_ref::<StringArray>()?;
    if row_index >= array.len() || array.is_null(row_index) {
        None
    } else {
        Some(array.value(row_index).to_owned())
    }
}

fn usize_at(batch: &RecordBatch, column: &str, row_index: usize) -> Option<usize> {
    let array = batch
        .column_by_name(column)?
        .as_any()
        .downcast_ref::<Int32Array>()?;
    if row_index >= array.len() || array.is_null(row_index) {
        return None;
    }
    usize::try_from(array.value(row_index)).ok()
}

fn f64_at(batch: &RecordBatch, column: &str, row_index: usize) -> Option<f64> {
    let array = batch
        .column_by_name(column)?
        .as_any()
        .downcast_ref::<Float64Array>()?;
    if row_index >= array.len() || array.is_null(row_index) {
        None
    } else {
        Some(array.value(row_index))
    }
}

fn route_string<'a>(route: &'a Value, key: &str) -> Result<&'a str, String> {
    route
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("SearchStrategyFlow retrieval route missing `{key}`"))
}

fn projected_page_id(repo_id: &str, doc_id: &str, source_path: &str) -> String {
    if doc_id.contains(":projection:") {
        return doc_id.to_owned();
    }
    let projection_kind = projection_kind_token_for_source_path(source_path);
    let effective_doc_id = if doc_id.trim().is_empty() {
        format!("repo:{repo_id}:doc:{source_path}")
    } else if doc_id.starts_with("repo:") {
        doc_id.to_owned()
    } else if !source_path.trim().is_empty() {
        format!("repo:{repo_id}:doc:{source_path}")
    } else {
        format!("repo:{repo_id}:doc:{doc_id}")
    };
    format!("repo:{repo_id}:projection:{projection_kind}:doc:{effective_doc_id}")
}

fn graph_node_display_id(repo_id: &str, source_path: &str) -> String {
    let normalized = source_path.trim().trim_matches('/');
    if normalized.starts_with(format!("{repo_id}/").as_str()) {
        normalized.to_owned()
    } else if normalized.is_empty() {
        repo_id.to_owned()
    } else {
        format!("{repo_id}/{normalized}")
    }
}

fn projection_kind_token_for_source_path(source_path: &str) -> &'static str {
    if has_markdown_extension(source_path) {
        "explanation"
    } else {
        "reference"
    }
}

fn repo_search_query(source_path: &str, heading_anchor: Option<&str>) -> String {
    let Some(anchor) = heading_anchor else {
        return source_path.to_owned();
    };
    let terms = search_terms(anchor);
    let effective_terms = stage_stripped_terms(terms.as_slice());
    if effective_terms.is_empty() {
        source_path.to_owned()
    } else {
        effective_terms.join(" ")
    }
}

fn stage_stripped_terms(terms: &[String]) -> &[String] {
    if terms.first().is_none_or(|term| term != "stage") {
        return terms;
    }
    match terms.get(1) {
        Some(term) if term.chars().all(|character| character.is_ascii_digit()) => &terms[2..],
        Some(_) => &terms[1..],
        None => &[],
    }
}

fn candidate_discovery_queries(intent: &str) -> Vec<RepoSearchAttempt> {
    let mut attempts = Vec::new();
    let trimmed = intent.trim();
    push_repo_search_attempt(&mut attempts, trimmed, "");

    let terms = search_terms(trimmed);
    if terms.is_empty() {
        return attempts;
    }

    push_repo_search_attempt(&mut attempts, terms.join(" ").as_str(), "");
    push_exact_anchor_candidate_attempts(&mut attempts, terms.as_slice());
    push_route_hint_candidate_attempts(&mut attempts, terms.as_slice());
    for window_size in [4, 3, 2] {
        if terms.len() < window_size {
            continue;
        }
        for window in terms.windows(window_size) {
            push_repo_search_attempt(&mut attempts, window.join(" ").as_str(), "");
        }
    }
    for term in terms.iter().filter(|term| term.len() >= 4) {
        push_repo_search_attempt(&mut attempts, term.as_str(), "");
    }
    attempts.truncate(32);
    attempts
}

fn push_exact_anchor_candidate_attempts(attempts: &mut Vec<RepoSearchAttempt>, terms: &[String]) {
    if has_all_terms(terms, &["search", "strategy", "flow"]) {
        push_repo_search_attempt(attempts, "SearchStrategyFlow", "docs/30_search_strategy");
        push_repo_search_attempt(attempts, "SearchStrategyFlow", "");
    }
    if has_all_terms(terms, &["page", "index"]) {
        push_repo_search_attempt(attempts, "PageIndex", "docs/20_page_index");
        push_repo_search_attempt(attempts, "PageIndex", "");
    }
    if has_all_terms(terms, &["link", "graph"]) {
        push_repo_search_attempt(attempts, "LinkGraph", "docs/10_graph_compute");
        push_repo_search_attempt(attempts, "LinkGraph", "");
    }
}

fn push_route_hint_candidate_attempts(attempts: &mut Vec<RepoSearchAttempt>, terms: &[String]) {
    if has_all_terms(terms, &["search", "strategy"]) {
        push_repo_search_attempt(attempts, "search strategy flow", "docs/30_search_strategy");
    }
    if has_all_terms(terms, &["page", "index"]) || has_all_terms(terms, &["reasoning", "tree"]) {
        push_repo_search_attempt(attempts, "page index reasoning tree", "docs/20_page_index");
    }
    if has_all_terms(terms, &["link", "graph"])
        || has_all_terms(terms, &["graph"])
        || has_all_terms(terms, &["relation"])
    {
        push_repo_search_attempt(attempts, "link graph compute", "docs/10_graph_compute");
    }
}

fn has_all_terms(terms: &[String], needles: &[&str]) -> bool {
    needles
        .iter()
        .all(|needle| terms.iter().any(|term| term == needle))
}

fn repo_search_attempts_for_route(
    repo_id: &str,
    source_path: &str,
    heading_anchor: Option<&str>,
) -> Vec<RepoSearchAttempt> {
    let mut attempts = Vec::new();
    let mut relaxed_attempts = Vec::new();
    let repo_relative_source_path = repo_relative_source_path(repo_id, source_path);
    let anchor_query = repo_search_query(repo_relative_source_path.as_str(), heading_anchor);
    push_repo_search_attempt(
        &mut attempts,
        anchor_query.as_str(),
        repo_relative_source_path.as_str(),
    );
    push_repo_search_attempt(&mut relaxed_attempts, anchor_query.as_str(), "");

    for file_query in source_path_queries(repo_relative_source_path.as_str()) {
        push_repo_search_attempt(
            &mut attempts,
            file_query.as_str(),
            repo_relative_source_path.as_str(),
        );
        push_repo_search_attempt(&mut relaxed_attempts, file_query.as_str(), "");
    }

    push_repo_search_attempt(
        &mut relaxed_attempts,
        repo_relative_source_path.as_str(),
        "",
    );
    if repo_relative_source_path != source_path.trim().trim_matches('/') {
        push_repo_search_attempt(&mut relaxed_attempts, source_path, "");
    }
    attempts.extend(relaxed_attempts);
    attempts
}

fn push_repo_search_attempt(attempts: &mut Vec<RepoSearchAttempt>, query: &str, path_prefix: &str) {
    let query = query.trim();
    if query.is_empty() {
        return;
    }
    let attempt = RepoSearchAttempt {
        query: query.to_owned(),
        path_prefix: path_prefix.trim().to_owned(),
    };
    if !attempts.contains(&attempt) {
        attempts.push(attempt);
    }
}

#[cfg(test)]
fn source_path_query(source_path: &str) -> String {
    source_path_queries(source_path)
        .into_iter()
        .next()
        .unwrap_or_else(|| source_path.trim().to_owned())
}

fn source_path_queries(source_path: &str) -> Vec<String> {
    let file_name = source_path
        .trim()
        .trim_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(source_path)
        .rsplit_once('.')
        .map_or_else(|| source_path.trim(), |(stem, _)| stem);
    let terms = search_terms(file_name);
    if terms.is_empty() {
        return vec![source_path.trim().to_owned()];
    }

    let semantic_terms = terms
        .iter()
        .filter(|term| !term.chars().all(|character| character.is_ascii_digit()))
        .cloned()
        .collect::<Vec<_>>();
    let mut queries = Vec::new();
    let joined_terms = terms.join(" ");
    push_unique_query(&mut queries, &joined_terms);
    if !semantic_terms.is_empty() {
        let joined_semantic_terms = semantic_terms.join(" ");
        push_unique_query(&mut queries, &joined_semantic_terms);
        let compact = semantic_terms.join("");
        if compact.len() >= 4 {
            push_unique_query(&mut queries, &compact);
        }
    }
    queries
}

fn push_unique_query(queries: &mut Vec<String>, query: &str) {
    let query = query.trim();
    if !query.is_empty() && !queries.iter().any(|existing| existing == query) {
        queries.push(query.to_owned());
    }
}

fn has_markdown_extension(path: &str) -> bool {
    Path::new(path.trim()).extension().is_some_and(|extension| {
        extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
    })
}

fn repo_relative_source_path(repo_id: &str, source_path: &str) -> String {
    let normalized = source_path.trim().trim_matches('/');
    let repo_prefix = format!("{}/", repo_id.trim().trim_matches('/'));
    normalized
        .strip_prefix(repo_prefix.as_str())
        .unwrap_or(normalized)
        .to_owned()
}

fn normalized_repo_search_doc_id(
    repo_id: &str,
    repo_relative_path: &str,
    doc_id: Option<&str>,
) -> String {
    let repo_id = repo_id.trim().trim_matches('/');
    let repo_relative_path = repo_relative_path.trim().trim_matches('/');
    let canonical_doc_id = || format!("repo:{repo_id}:doc:{repo_relative_path}");
    let Some(doc_id) = doc_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return canonical_doc_id();
    };
    let repo_display_doc_prefix = format!("repo:{repo_id}:doc:{repo_id}/");
    if let Some(relative_path) = doc_id.strip_prefix(repo_display_doc_prefix.as_str()) {
        return format!("repo:{repo_id}:doc:{relative_path}");
    }
    if doc_id.starts_with("repo:") {
        doc_id.to_owned()
    } else {
        canonical_doc_id()
    }
}

fn search_terms(text: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();
    let mut previous_was_lowercase = false;

    for character in text.chars() {
        if character.is_ascii_alphanumeric() {
            if previous_was_lowercase && character.is_ascii_uppercase() && !current.is_empty() {
                terms.push(std::mem::take(&mut current));
            }
            current.push(character.to_ascii_lowercase());
            previous_was_lowercase = character.is_ascii_lowercase();
        } else {
            if !current.is_empty() {
                terms.push(std::mem::take(&mut current));
            }
            previous_was_lowercase = false;
        }
    }
    if !current.is_empty() {
        terms.push(current);
    }

    terms.into_iter().filter(|term| term.len() >= 2).collect()
}

fn find_node_id_by_anchor_or_title(value: &Value, anchor: &str) -> Option<String> {
    match value {
        Value::Array(nodes) => nodes
            .iter()
            .find_map(|node| find_node_id_by_anchor_or_title(node, anchor)),
        Value::Object(node) => {
            let expected_slug = normalize_anchor(anchor);
            if ["anchor", "headingAnchor", "slug"]
                .iter()
                .filter_map(|key| node.get(*key).and_then(Value::as_str))
                .any(|value| normalize_anchor(value) == expected_slug)
                || node
                    .get("title")
                    .and_then(Value::as_str)
                    .is_some_and(|title| normalize_anchor(title) == expected_slug)
                || node
                    .get("node_id")
                    .and_then(Value::as_str)
                    .is_some_and(|node_id| node_id.ends_with(anchor))
            {
                return node
                    .get("node_id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
            }
            node.get("children")
                .and_then(|children| find_node_id_by_anchor_or_title(children, anchor))
        }
        _ => None,
    }
}

fn first_node_id(value: &Value) -> Option<String> {
    match value {
        Value::Array(nodes) => nodes.iter().find_map(first_node_id),
        Value::Object(node) => node
            .get("node_id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| node.get("children").and_then(first_node_id)),
        _ => None,
    }
}

fn normalize_anchor(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod page_id_tests {
    use super::{graph_node_display_id, projected_page_id, projection_kind_token_for_source_path};

    #[test]
    fn projected_page_id_normalizes_markdown_doc_id_with_path() {
        let page_id = projected_page_id(
            "wendaograph_search_strategy",
            "30.01_search_strategy_flow.md",
            "docs/30_search_strategy/30.01_search_strategy_flow.md",
        );

        assert_eq!(
            page_id,
            "repo:wendaograph_search_strategy:projection:explanation:doc:repo:wendaograph_search_strategy:doc:docs/30_search_strategy/30.01_search_strategy_flow.md"
        );
    }

    #[test]
    fn projected_page_id_preserves_full_markdown_doc_id() {
        let page_id = projected_page_id(
            "wendaograph_search_strategy",
            "repo:wendaograph_search_strategy:doc:docs/30_search_strategy/30.01_search_strategy_flow.md",
            "docs/30_search_strategy/30.01_search_strategy_flow.md",
        );

        assert_eq!(
            page_id,
            "repo:wendaograph_search_strategy:projection:explanation:doc:repo:wendaograph_search_strategy:doc:docs/30_search_strategy/30.01_search_strategy_flow.md"
        );
    }

    #[test]
    fn projection_kind_token_for_source_path_matches_default_markdown_parser() {
        assert_eq!(
            projection_kind_token_for_source_path("docs/search.md"),
            "explanation"
        );
        assert_eq!(
            projection_kind_token_for_source_path("src/SearchStrategyFlow.jl"),
            "reference"
        );
    }

    #[test]
    fn graph_node_display_id_scopes_source_path_to_repo_display_path() {
        let node_id = graph_node_display_id(
            "wendaograph_search_strategy",
            "docs/30_search_strategy/30.01_search_strategy_flow.md",
        );

        assert_eq!(
            node_id,
            "wendaograph_search_strategy/docs/30_search_strategy/30.01_search_strategy_flow.md"
        );
    }

    #[test]
    fn graph_node_display_id_does_not_double_scope_repo_prefixed_path() {
        let node_id = graph_node_display_id(
            "wendaograph_search_strategy",
            "wendaograph_search_strategy/docs/30_search_strategy/30.01_search_strategy_flow.md",
        );

        assert_eq!(
            node_id,
            "wendaograph_search_strategy/docs/30_search_strategy/30.01_search_strategy_flow.md"
        );
    }
}

#[cfg(test)]
mod query_tests {
    use super::{
        candidate_discovery_queries, normalized_repo_search_doc_id, populate_repo_search_headers,
        repo_relative_source_path, repo_search_attempts_for_route, repo_search_query, search_terms,
        source_path_queries, source_path_query,
    };
    use tonic::metadata::MetadataMap;
    use xiuxian_wendao_runtime::transport::{
        WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER, WENDAO_REPO_SEARCH_LIMIT_HEADER,
        WENDAO_REPO_SEARCH_QUERY_HEADER, WENDAO_REPO_SEARCH_REPO_HEADER,
    };

    #[test]
    fn search_strategy_flow_repo_query_uses_anchor_terms_not_path_fragment() {
        let query = repo_search_query(
            "docs/30_search_strategy/30.01_search_strategy_flow.md",
            Some("stage-1-query-understanding"),
        );

        assert_eq!(query, "query understanding");
    }

    #[test]
    fn search_strategy_flow_repo_query_falls_back_to_source_path_without_anchor() {
        let query = repo_search_query(
            "docs/30_search_strategy/30.01_search_strategy_flow.md",
            None,
        );

        assert_eq!(
            query,
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        );
    }

    #[test]
    fn candidate_discovery_queries_decompose_natural_language_intent() {
        let attempts = candidate_discovery_queries(
            "query understanding reasoning tree page index search strategy flow",
        );
        let queries = attempts
            .iter()
            .map(|attempt| attempt.query.as_str())
            .collect::<Vec<_>>();

        assert!(
            queries.contains(&"query understanding reasoning tree page index search strategy flow")
        );
        assert!(queries.contains(&"SearchStrategyFlow"));
        assert!(queries.contains(&"PageIndex"));
        assert!(queries.contains(&"search strategy flow"));
        assert!(queries.contains(&"query understanding"));
        assert!(queries.contains(&"reasoning tree"));
        assert!(attempts.iter().any(|attempt| {
            attempt.query == "SearchStrategyFlow"
                && attempt.path_prefix == "docs/30_search_strategy"
        }));
        assert!(attempts.iter().any(|attempt| {
            attempt.query == "PageIndex" && attempt.path_prefix == "docs/20_page_index"
        }));
        assert!(attempts.iter().any(|attempt| {
            attempt.query == "page index reasoning tree"
                && attempt.path_prefix == "docs/20_page_index"
        }));
        assert!(attempts.iter().any(|attempt| {
            attempt.query == "search strategy flow"
                && attempt.path_prefix == "docs/30_search_strategy"
        }));
        let Some(search_strategy_prefixed_index) = attempts.iter().position(|attempt| {
            attempt.query == "SearchStrategyFlow"
                && attempt.path_prefix == "docs/30_search_strategy"
        }) else {
            panic!("SearchStrategyFlow prefixed anchor should be attempted");
        };
        let Some(search_strategy_broad_index) = attempts.iter().position(|attempt| {
            attempt.query == "SearchStrategyFlow" && attempt.path_prefix.is_empty()
        }) else {
            panic!("SearchStrategyFlow broad anchor should be attempted");
        };
        assert!(
            search_strategy_prefixed_index < search_strategy_broad_index,
            "prefixed exact anchors should run before broad anchors for route diversity"
        );
    }

    #[test]
    fn candidate_discovery_queries_split_camel_case_intent() {
        let attempts = candidate_discovery_queries("SearchStrategyFlow");

        assert!(
            attempts
                .iter()
                .any(|attempt| attempt.query == "search strategy flow")
        );
    }

    #[test]
    fn repo_search_headers_filter_candidate_discovery_to_markdown()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = MetadataMap::new();

        populate_repo_search_headers(
            &mut metadata,
            "wendaograph",
            "search strategy flow",
            10,
            "docs/30_search_strategy",
        )?;

        assert_eq!(
            metadata
                .get(WENDAO_REPO_SEARCH_REPO_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("wendaograph")
        );
        assert_eq!(
            metadata
                .get(WENDAO_REPO_SEARCH_QUERY_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("search strategy flow")
        );
        assert_eq!(
            metadata
                .get(WENDAO_REPO_SEARCH_LIMIT_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("10")
        );
        assert_eq!(
            metadata
                .get(WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("markdown")
        );
        Ok(())
    }

    #[test]
    fn route_repo_search_attempts_relax_exact_path_prefix() {
        let attempts = repo_search_attempts_for_route(
            "wendaograph",
            "docs/30_search_strategy/30.01_search_strategy_flow.md",
            Some("stage-1-query-understanding"),
        );

        assert_eq!(attempts[0].query, "query understanding");
        assert_eq!(
            attempts[0].path_prefix,
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        );
        assert!(attempts.iter().any(
            |attempt| attempt.query == "query understanding" && attempt.path_prefix.is_empty()
        ));
        assert!(
            attempts
                .iter()
                .any(|attempt| attempt.query == "30 01 search strategy flow"
                    && attempt.path_prefix.is_empty())
        );
    }

    #[test]
    fn route_repo_search_attempts_strip_repo_prefix_and_add_compact_file_query() {
        let attempts = repo_search_attempts_for_route(
            "wendaograph",
            "wendaograph/docs/30_search_strategy/30.01_search_strategy_flow.md",
            Some("stage-1-query-understanding"),
        );

        assert!(attempts.iter().any(|attempt| {
            attempt.query == "searchstrategyflow"
                && attempt.path_prefix == "docs/30_search_strategy/30.01_search_strategy_flow.md"
        }));
        assert!(!attempts.iter().any(|attempt| attempt.path_prefix
            == "wendaograph/docs/30_search_strategy/30.01_search_strategy_flow.md"));
    }

    #[test]
    fn search_terms_split_symbols_and_camel_case() {
        assert_eq!(
            search_terms("SearchStrategyFlow stage-1_query"),
            vec!["search", "strategy", "flow", "stage", "query"]
        );
    }

    #[test]
    fn source_path_query_uses_file_stem_terms() {
        assert_eq!(
            source_path_query("docs/30_search_strategy/30.01_search_strategy_flow.md"),
            "30 01 search strategy flow"
        );
    }

    #[test]
    fn source_path_queries_include_compact_semantic_stem() {
        assert_eq!(
            source_path_queries("docs/30_search_strategy/30.01_search_strategy_flow.md"),
            vec![
                "30 01 search strategy flow",
                "search strategy flow",
                "searchstrategyflow"
            ]
        );
    }

    #[test]
    fn repo_relative_source_path_strips_repo_display_prefix() {
        assert_eq!(
            repo_relative_source_path(
                "wendaograph",
                "wendaograph/docs/30_search_strategy/30.01_search_strategy_flow.md",
            ),
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        );
    }

    #[test]
    fn normalized_repo_search_doc_id_strips_repo_display_path_prefix() {
        assert_eq!(
            normalized_repo_search_doc_id(
                "wendaograph",
                "docs/30_search_strategy/30.01_search_strategy_flow.md",
                Some(
                    "repo:wendaograph:doc:wendaograph/docs/30_search_strategy/30.01_search_strategy_flow.md"
                ),
            ),
            "repo:wendaograph:doc:docs/30_search_strategy/30.01_search_strategy_flow.md"
        );
    }
}

#[cfg(test)]
mod candidate_tests {
    use std::sync::Arc;

    use arrow::array::{Float64Array, Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    use super::{
        SearchStrategyFlowRouteReceipt, apply_route_receipt, decoded_payload_receipts,
        first_page_index_repo_search_row, repo_relative_candidate_inputs,
        repo_search_batches_to_candidate_inputs,
    };
    use crate::integration_support::search_strategy_flow_candidates::{
        FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE, SearchStrategyFlowRepoSearchHit,
        search_strategy_flow_candidate_input_batch,
        search_strategy_flow_candidate_input_from_repo_search_hit,
    };
    use xiuxian_wendao_runtime::transport::{
        ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE, GRAPH_NEIGHBORS_ROUTE,
        REPO_SEARCH_BEST_SECTION_COLUMN, REPO_SEARCH_DOC_ID_COLUMN,
        REPO_SEARCH_NAVIGATION_LINE_COLUMN, REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
        REPO_SEARCH_NAVIGATION_PATH_COLUMN, REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_ROUTE,
        REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TITLE_COLUMN,
    };

    #[test]
    fn repo_search_batches_map_to_section_candidates() -> Result<(), Box<dyn std::error::Error>> {
        let schema = Arc::new(Schema::new(vec![
            Field::new(REPO_SEARCH_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_NAVIGATION_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_TITLE_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_BEST_SECTION_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_NAVIGATION_LINE_COLUMN, DataType::Int32, false),
            Field::new(
                REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
                DataType::Int32,
                false,
            ),
            Field::new(REPO_SEARCH_SCORE_COLUMN, DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec![
                    "docs/search.md",
                    "src/search_strategy_flow.jl",
                ])),
                Arc::new(StringArray::from(vec![
                    "docs/search.md",
                    "src/search_strategy_flow.jl",
                ])),
                Arc::new(StringArray::from(vec![
                    "Search Strategy",
                    "SearchStrategyFlow",
                ])),
                Arc::new(StringArray::from(vec![
                    "Query Understanding",
                    "query_understanding",
                ])),
                Arc::new(Int32Array::from(vec![12, 1])),
                Arc::new(Int32Array::from(vec![18, 40])),
                Arc::new(Float64Array::from(vec![0.92, 0.99])),
            ],
        )?;

        let candidates = repo_search_batches_to_candidate_inputs(&[batch]);

        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.relative_path, "docs/search.md");
        assert_eq!(candidate.heading_anchor, "query-understanding");
        assert_eq!(candidate.line_start, 12);
        assert_eq!(candidate.line_end, 18);
        assert!(candidate.edge_kinds.contains(&"arrow-flight".to_owned()));
        assert!(candidate.edge_kinds.contains(&"repo-search".to_owned()));
        let batch = search_strategy_flow_candidate_input_batch(
            FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE,
            &candidates,
        );
        assert_eq!(batch.source, FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE);
        assert_eq!(batch.row_count, 1);
        Ok(())
    }

    #[test]
    fn repo_relative_candidate_inputs_strip_repo_display_prefix() {
        let hit = SearchStrategyFlowRepoSearchHit {
            relative_path: "wendaograph/docs/search.md",
            title: Some("Search Strategy"),
            best_section: Some("Query Understanding"),
            line_start: Some(1),
            line_end: Some(3),
            score: Some(0.8),
        };
        let candidates = repo_relative_candidate_inputs(
            "wendaograph",
            vec![search_strategy_flow_candidate_input_from_repo_search_hit(
                &hit,
            )],
        );

        assert_eq!(candidates[0].relative_path, "docs/search.md");
    }

    #[test]
    fn first_page_index_repo_search_row_prefers_planned_source_path()
    -> Result<(), Box<dyn std::error::Error>> {
        let schema = Arc::new(Schema::new(vec![
            Field::new(REPO_SEARCH_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_NAVIGATION_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_DOC_ID_COLUMN, DataType::Utf8, false),
        ]));
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec![
                    "README.md",
                    "docs/30_search_strategy/30.01_search_strategy_flow.md",
                ])),
                Arc::new(StringArray::from(vec![
                    "README.md",
                    "docs/30_search_strategy/30.01_search_strategy_flow.md",
                ])),
                Arc::new(StringArray::from(vec![
                    "repo:wendaograph:doc:README.md",
                    "repo:wendaograph:doc:docs/30_search_strategy/30.01_search_strategy_flow.md",
                ])),
            ],
        )?;

        let row = first_page_index_repo_search_row(
            &[batch],
            "wendaograph",
            Some("docs/30_search_strategy/30.01_search_strategy_flow.md"),
        )
        .unwrap_or_else(|| panic!("expected preferred row"));

        assert_eq!(
            row.0,
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        );
        assert_eq!(
            row.1.as_deref(),
            Some("repo:wendaograph:doc:docs/30_search_strategy/30.01_search_strategy_flow.md")
        );
        Ok(())
    }

    #[test]
    fn first_page_index_repo_search_row_does_not_drift_from_planned_source_path()
    -> Result<(), Box<dyn std::error::Error>> {
        let schema = Arc::new(Schema::new(vec![
            Field::new(REPO_SEARCH_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_NAVIGATION_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_DOC_ID_COLUMN, DataType::Utf8, false),
        ]));
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["README.md"])),
                Arc::new(StringArray::from(vec!["README.md"])),
                Arc::new(StringArray::from(vec!["repo:wendaograph:doc:README.md"])),
            ],
        )?;

        let row = first_page_index_repo_search_row(
            &[batch],
            "wendaograph",
            Some("docs/10_graph_compute/10.01_reasoning_tree.md"),
        );

        assert_eq!(row, None);
        Ok(())
    }

    #[test]
    fn decoded_payload_receipts_include_route_provenance_anchors() {
        let repo_search_batch = string_batch(
            &[REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_PATH_COLUMN],
            &[
                &["repo:wendaograph:doc:docs/30_search_strategy/30.02_precision_pruning.md"],
                &["docs/30_search_strategy/30.02_precision_pruning.md"],
            ],
        );
        let page_index_batch = string_batch(
            &["pageId", "rootsJson"],
            &[
                &[
                    "repo:wendaograph:projection:explanation:doc:repo:wendaograph:doc:docs/30_search_strategy/30.02_precision_pruning.md",
                ],
                &["{\"roots\":[]}"],
            ],
        );
        let retrieval_context_batch = string_batch(
            &["pageId", "nodeId", "centerJson", "nodeContextJson"],
            &[
                &[
                    "repo:wendaograph:projection:explanation:doc:repo:wendaograph:doc:docs/30_search_strategy/30.02_precision_pruning.md",
                ],
                &["node:precision-pruning"],
                &["{}"],
                &["{}"],
            ],
        );
        let graph_batch = string_batch(&["rowType"], &[&["neighbor"]]);

        let receipts = decoded_payload_receipts(
            "docs/30_search_strategy/30.02_precision_pruning.md",
            &[repo_search_batch],
            &[page_index_batch],
            &[retrieval_context_batch],
            &[graph_batch],
            "node:precision-pruning",
            "wendaograph/docs/30_search_strategy/30.02_precision_pruning.md",
        )
        .unwrap_or_else(|error| panic!("build decoded receipts: {error}"));

        assert_eq!(receipts.len(), 4);
        assert_eq!(
            receipts[0].get("route"),
            Some(&serde_json::json!(REPO_SEARCH_ROUTE))
        );
        assert_eq!(
            receipts[0].get("evidenceAnchor"),
            Some(&serde_json::json!(
                "path:docs/30_search_strategy/30.02_precision_pruning.md"
            ))
        );
        assert_eq!(
            receipts[1].get("evidenceAnchor"),
            Some(&serde_json::json!("node:precision-pruning"))
        );
        assert_eq!(
            receipts[2].get("evidenceAnchor"),
            Some(&serde_json::json!("node-context:node:precision-pruning"))
        );
        assert_eq!(
            receipts[3].get("evidenceAnchor"),
            Some(&serde_json::json!(
                "graph-node:wendaograph/docs/30_search_strategy/30.02_precision_pruning.md"
            ))
        );
    }

    #[test]
    fn executed_route_receipt_preserves_section_source_and_materialization_rows() {
        let mut route = serde_json::json!({
            "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md#precision-score",
            "materializationOwner": "studio-rust",
            "materializationStatus": "planned",
            "receiptSource": "rust-bridge",
            "primaryTransport": "arrow-flight",
            "sourcePath": "docs/30_search_strategy/30.02_precision_pruning.md",
            "headingAnchor": "precision-score",
            "directFileReadAllowed": false,
            "executeBeforeAnswer": true
        });
        let receipt = SearchStrategyFlowRouteReceipt {
            materialized_rows: 4,
            resolved_page_id:
                "repo:wendaograph:projection:explanation:doc:repo:wendaograph:doc:docs/30_search_strategy/30.02_precision_pruning.md"
                    .to_owned(),
            resolved_node_id: "node:precision-score".to_owned(),
            resolved_graph_node_id:
                "wendaograph/docs/30_search_strategy/30.02_precision_pruning.md".to_owned(),
            route_receipts: vec![
                serde_json::json!({"route": REPO_SEARCH_ROUTE, "rowCount": 1}),
                serde_json::json!({"route": ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, "rowCount": 1}),
                serde_json::json!({"route": ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE, "rowCount": 1}),
                serde_json::json!({"route": GRAPH_NEIGHBORS_ROUTE, "rowCount": 1}),
            ],
            decoded_payload_receipts: vec![
                serde_json::json!({
                    "route": ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
                    "rowCount": 1,
                    "decodedColumns": ["pageId", "nodeId", "centerJson", "nodeContextJson"],
                    "evidenceAnchor": "node-context:node:precision-score"
                }),
                serde_json::json!({
                    "route": GRAPH_NEIGHBORS_ROUTE,
                    "rowCount": 1,
                    "decodedColumns": ["rowType"],
                    "evidenceAnchor": "graph-node:wendaograph/docs/30_search_strategy/30.02_precision_pruning.md"
                }),
            ],
        };

        apply_route_receipt(&mut route, receipt)
            .unwrap_or_else(|error| panic!("apply route receipt: {error}"));

        assert_eq!(
            route.get("sourcePath"),
            Some(&serde_json::json!(
                "docs/30_search_strategy/30.02_precision_pruning.md"
            ))
        );
        assert_eq!(
            route.get("headingAnchor"),
            Some(&serde_json::json!("precision-score"))
        );
        assert_eq!(
            route.get("directFileReadAllowed"),
            Some(&serde_json::json!(false))
        );
        assert_eq!(
            route.get("materializationStatus"),
            Some(&serde_json::json!("executed"))
        );
        assert_eq!(route.get("materializedRows"), Some(&serde_json::json!(4)));
        assert_eq!(
            route.get("resolvedNodeId"),
            Some(&serde_json::json!("node:precision-score"))
        );
        assert_eq!(
            route.get("decodedPayloadStatus"),
            Some(&serde_json::json!("decoded"))
        );
        assert_eq!(
            route
                .get("routeReceipts")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len),
            Some(4)
        );
        assert_eq!(
            route
                .get("decodedPayloadReceipts")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len),
            Some(2)
        );
    }

    fn string_batch(columns: &[&str], values: &[&[&str]]) -> RecordBatch {
        let schema = Arc::new(Schema::new(
            columns
                .iter()
                .map(|column| Field::new(*column, DataType::Utf8, false))
                .collect::<Vec<_>>(),
        ));
        let arrays = values
            .iter()
            .map(|column_values| Arc::new(StringArray::from(column_values.to_vec())) as _)
            .collect::<Vec<_>>();
        RecordBatch::try_new(schema, arrays)
            .unwrap_or_else(|error| panic!("build string record batch: {error}"))
    }
}
