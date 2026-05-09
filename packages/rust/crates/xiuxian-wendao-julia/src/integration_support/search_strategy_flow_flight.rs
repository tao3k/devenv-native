//! Arrow Flight materialization for Rust-owned SearchStrategyFlow routes.

use std::time::Duration;

use arrow::array::{Array, StringArray};
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
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_ROUTE,
    WENDAO_GRAPH_DIRECTION_HEADER, WENDAO_GRAPH_HOPS_HEADER, WENDAO_GRAPH_LIMIT_HEADER,
    WENDAO_GRAPH_NODE_ID_HEADER, WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER, WENDAO_REPO_SEARCH_LIMIT_HEADER,
    WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER, WENDAO_REPO_SEARCH_QUERY_HEADER,
    WENDAO_REPO_SEARCH_REPO_HEADER, WENDAO_SCHEMA_VERSION_HEADER, flight_descriptor_path,
};

const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const REPO_SEARCH_LIMIT: usize = 5;
const RELATED_CONTEXT_LIMIT: usize = 5;
const GRAPH_HOPS: usize = 2;
const GRAPH_LIMIT: usize = 50;

/// Network endpoint settings for Rust-owned SearchStrategyFlow Flight
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

/// Executes all SearchStrategyFlow retrieval routes in a JSON trace through a
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
        if row_count(&batches) == 0 {
            return Err(format!("{context} returned zero decoded rows"));
        }
        Ok(batches)
    }
}

async fn materialize_route(
    client: &mut SearchStrategyFlowFlightClient,
    config: &SearchStrategyFlowFlightMaterializationConfig,
    route: &Value,
) -> Result<SearchStrategyFlowRouteReceipt, String> {
    let source_path = route_string(route, "sourcePath")?;
    let heading_anchor = route.get("headingAnchor").and_then(Value::as_str);
    let query = repo_search_query(source_path, heading_anchor);

    let repo_search_batches = client
        .collect_route_batches(
            REPO_SEARCH_ROUTE,
            "SearchStrategyFlow repo search materialization",
            |metadata| {
                populate_repo_search_headers(
                    metadata,
                    &config.repo_id,
                    query.as_str(),
                    REPO_SEARCH_LIMIT,
                    source_path,
                )
            },
        )
        .await?;
    let repo_search_path = first_string(&repo_search_batches[0], REPO_SEARCH_PATH_COLUMN)
        .unwrap_or_else(|_| source_path.to_owned());
    let doc_id = first_string(&repo_search_batches[0], REPO_SEARCH_DOC_ID_COLUMN)
        .unwrap_or_else(|_| format!("repo:{}:doc:{repo_search_path}", config.repo_id));
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

    let route_receipts = vec![
        route_receipt(REPO_SEARCH_ROUTE, &repo_search_batches),
        route_receipt(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            &page_index_batches,
        ),
        route_receipt(
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
            &retrieval_context_batches,
        ),
        route_receipt(GRAPH_NEIGHBORS_ROUTE, &graph_batches),
    ];
    let decoded_payload_receipts = vec![
        decoded_payload_receipt(
            REPO_SEARCH_ROUTE,
            &repo_search_batches,
            vec![REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_PATH_COLUMN],
            format!(
                "path:{}",
                first_string(&repo_search_batches[0], REPO_SEARCH_PATH_COLUMN)?
            ),
        ),
        decoded_payload_receipt(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            &page_index_batches,
            vec!["pageId", "rootCount", "rootsJson"],
            format!("node:{node_id}"),
        ),
        decoded_payload_receipt(
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
            &retrieval_context_batches,
            vec!["pageId", "nodeId", "centerJson", "nodeContextJson"],
            format!(
                "node-context:{}",
                first_string(&retrieval_context_batches[0], "nodeId")?
            ),
        ),
        decoded_payload_receipt(
            GRAPH_NEIGHBORS_ROUTE,
            &graph_batches,
            vec!["rowType"],
            format!("graph-node:{graph_node_id}"),
        ),
    ];
    let materialized_rows = route_receipts
        .iter()
        .map(|receipt| {
            receipt
                .get("rowCount")
                .and_then(Value::as_u64)
                .unwrap_or_default() as usize
        })
        .sum();

    Ok(SearchStrategyFlowRouteReceipt {
        materialized_rows,
        resolved_page_id: page_id,
        resolved_node_id: node_id,
        resolved_graph_node_id: graph_node_id,
        route_receipts,
        decoded_payload_receipts,
    })
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
    decoded_columns: Vec<&str>,
    evidence_anchor: String,
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
    let effective_doc_id = if doc_id.trim().is_empty() {
        format!("repo:{repo_id}:doc:{source_path}")
    } else if doc_id.starts_with("repo:") {
        doc_id.to_owned()
    } else if !source_path.trim().is_empty() {
        format!("repo:{repo_id}:doc:{source_path}")
    } else {
        format!("repo:{repo_id}:doc:{doc_id}")
    };
    format!("repo:{repo_id}:projection:reference:doc:{effective_doc_id}")
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

fn repo_search_query(source_path: &str, heading_anchor: Option<&str>) -> String {
    let Some(anchor) = heading_anchor else {
        return source_path.to_owned();
    };
    let terms = anchor
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    let terms = match terms.as_slice() {
        ["stage", number, tail @ ..]
            if number.chars().all(|character| character.is_ascii_digit()) =>
        {
            tail
        }
        _ => terms.as_slice(),
    };
    if terms.is_empty() {
        source_path.to_owned()
    } else {
        terms.join(" ")
    }
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
    use super::{graph_node_display_id, projected_page_id};

    #[test]
    fn projected_page_id_normalizes_short_repo_search_doc_id_with_path() {
        let page_id = projected_page_id(
            "wendaograph_search_strategy",
            "30.01_search_strategy_flow.md",
            "docs/30_search_strategy/30.01_search_strategy_flow.md",
        );

        assert_eq!(
            page_id,
            "repo:wendaograph_search_strategy:projection:reference:doc:repo:wendaograph_search_strategy:doc:docs/30_search_strategy/30.01_search_strategy_flow.md"
        );
    }

    #[test]
    fn projected_page_id_preserves_full_doc_id() {
        let page_id = projected_page_id(
            "wendaograph_search_strategy",
            "repo:wendaograph_search_strategy:doc:docs/30_search_strategy/30.01_search_strategy_flow.md",
            "docs/30_search_strategy/30.01_search_strategy_flow.md",
        );

        assert_eq!(
            page_id,
            "repo:wendaograph_search_strategy:projection:reference:doc:repo:wendaograph_search_strategy:doc:docs/30_search_strategy/30.01_search_strategy_flow.md"
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
    use super::repo_search_query;

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
}
