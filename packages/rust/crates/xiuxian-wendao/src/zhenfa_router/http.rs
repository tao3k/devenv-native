//! `zhenfa_router::http` owns Wendao zhenfa router http behavior.

use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;
use xiuxian_zhenfa::{MethodRegistry, ZhenfaRouter};

use super::models::{WendaoSearchHttpResponse, WendaoSearchRequest};
use super::rpc::{
    execute_search_async, export_plugin_artifact_from_rpc_params, search_from_rpc_params,
};
use crate::link_graph::{
    LinkGraphDirection, LinkGraphDocument, LinkGraphIndex, LinkGraphMetadata, LinkGraphNeighbor,
    LinkGraphPlannedSearchPayload, LinkGraphSearchOptions, LinkGraphStats,
};

const WENDAO_PREFIX: &str = "/v1/wendao";
const WENDAO_SEARCH_ROUTE: &str = "/v1/wendao/search";
const WENDAO_SEARCH_PLANNED_ROUTE: &str = "/v1/wendao/search/planned";
const WENDAO_GRAPH_NEIGHBORS_ROUTE: &str = "/v1/wendao/graph/neighbors/{*id}";
const WENDAO_GRAPH_RELATED_ROUTE: &str = "/v1/wendao/graph/related/{*id}";
const WENDAO_GRAPH_METADATA_ROUTE: &str = "/v1/wendao/graph/metadata/{*id}";
const WENDAO_GRAPH_TOC_ROUTE: &str = "/v1/wendao/graph/toc";
const WENDAO_GRAPH_STATS_ROUTE: &str = "/v1/wendao/graph/stats";

const DEFAULT_SEARCH_LIMIT: usize = 20;
const MAX_SEARCH_LIMIT: usize = 200;
const DEFAULT_NEIGHBORS_HOPS: usize = 1;
const MAX_NEIGHBORS_HOPS: usize = 5;
const DEFAULT_NEIGHBORS_LIMIT: usize = 50;
const MAX_NEIGHBORS_LIMIT: usize = 200;
const DEFAULT_RELATED_DISTANCE: usize = 2;
const MAX_RELATED_DISTANCE: usize = 5;
const DEFAULT_RELATED_LIMIT: usize = 20;
const MAX_RELATED_LIMIT: usize = 200;
const DEFAULT_TOC_LIMIT: usize = 1000;
const MAX_TOC_LIMIT: usize = 5000;

/// `xiuxian-wendao` adapter mounted into `xiuxian-zhenfa` gateway.
#[derive(Clone, Default)]
pub struct WendaoZhenfaRouter;

impl WendaoZhenfaRouter {
    /// Create a new Wendao router adapter.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ZhenfaRouter for WendaoZhenfaRouter {
    fn prefix(&self) -> &'static str {
        WENDAO_PREFIX
    }

    fn mount(&self, router: Router) -> Router {
        router.merge(
            Router::new()
                .route(WENDAO_SEARCH_ROUTE, post(search_http))
                .route(WENDAO_SEARCH_PLANNED_ROUTE, post(search_planned_http))
                .route(WENDAO_GRAPH_NEIGHBORS_ROUTE, get(graph_neighbors_http))
                .route(WENDAO_GRAPH_RELATED_ROUTE, get(graph_related_http))
                .route(WENDAO_GRAPH_METADATA_ROUTE, get(graph_metadata_http))
                .route(WENDAO_GRAPH_TOC_ROUTE, get(graph_toc_http))
                .route(WENDAO_GRAPH_STATS_ROUTE, get(graph_stats_http)),
        )
    }

    fn register_methods(&self, registry: &mut MethodRegistry) {
        registry.register_fn("wendao.search", move |params, _meta| async move {
            search_from_rpc_params(params)
        });
        registry.register_fn("wendao.plugin_artifact", move |params, _meta| async move {
            export_plugin_artifact_from_rpc_params(params)
        });
    }
}

/// JSON body for planned search endpoint.
#[derive(Debug, Deserialize)]
struct WendaoSearchPlannedRequest {
    query: String,
    #[serde(default)]
    query_vector: Option<Vec<f32>>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    root_dir: Option<String>,
    #[serde(default)]
    options: Option<Value>,
    #[serde(default)]
    include_provisional: Option<bool>,
    #[serde(default)]
    provisional_limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct GraphNeighborsQuery {
    #[serde(default)]
    root_dir: Option<String>,
    #[serde(default)]
    direction: Option<String>,
    #[serde(default = "default_neighbors_hops")]
    hops: usize,
    #[serde(default = "default_neighbors_limit")]
    limit: usize,
}

#[derive(Debug, Deserialize)]
struct GraphRelatedQuery {
    #[serde(default)]
    root_dir: Option<String>,
    #[serde(default = "default_related_distance")]
    max_distance: usize,
    #[serde(default = "default_related_limit")]
    limit: usize,
}

#[derive(Debug, Deserialize)]
struct GraphMetadataQuery {
    #[serde(default)]
    root_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GraphTocQuery {
    #[serde(default)]
    root_dir: Option<String>,
    #[serde(default = "default_toc_limit")]
    limit: usize,
}

#[derive(Debug, Deserialize)]
struct GraphStatsQuery {
    #[serde(default)]
    root_dir: Option<String>,
}

async fn search_http(
    Json(body): Json<WendaoSearchRequest>,
) -> Result<Json<WendaoSearchHttpResponse>, (StatusCode, Json<Value>)> {
    execute_search_async(&body)
        .await
        .map(|result| Json(WendaoSearchHttpResponse { result }))
        .map_err(|error: String| internal_http_error(error.as_str()))
}

async fn search_planned_http(
    Json(body): Json<WendaoSearchPlannedRequest>,
) -> Result<Json<LinkGraphPlannedSearchPayload>, (StatusCode, Json<Value>)> {
    let query = body.query.trim();
    if query.is_empty() {
        return Err(bad_request("`query` must be a non-empty string"));
    }

    let index = build_index(body.root_dir.as_deref())?;
    let limit = normalize_limit(body.limit, DEFAULT_SEARCH_LIMIT, MAX_SEARCH_LIMIT);
    let base_options =
        parse_search_options(body.options).map_err(|error| bad_request(error.as_str()))?;

    let payload = index
        .search_planned_payload_with_agentic_async_with_query_vector(
            query,
            body.query_vector.as_deref().unwrap_or(&[]),
            limit,
            base_options,
            body.include_provisional,
            body.provisional_limit,
        )
        .await;
    Ok(Json(payload))
}

async fn graph_neighbors_http(
    Path(id): Path<String>,
    Query(query): Query<GraphNeighborsQuery>,
) -> Result<Json<Vec<LinkGraphNeighbor>>, (StatusCode, Json<Value>)> {
    let index = build_index(query.root_dir.as_deref())?;
    let direction = parse_direction(query.direction.as_deref());
    let hops = query.hops.clamp(1, MAX_NEIGHBORS_HOPS);
    let limit = query.limit.clamp(1, MAX_NEIGHBORS_LIMIT);
    let neighbors = index.neighbors(&id, direction, hops, limit);
    Ok(Json(neighbors))
}

async fn graph_related_http(
    Path(id): Path<String>,
    Query(query): Query<GraphRelatedQuery>,
) -> Result<Json<Vec<LinkGraphNeighbor>>, (StatusCode, Json<Value>)> {
    let index = build_index(query.root_dir.as_deref())?;
    let max_distance = query.max_distance.clamp(1, MAX_RELATED_DISTANCE);
    let limit = query.limit.clamp(1, MAX_RELATED_LIMIT);
    let related = index.related(&id, max_distance, limit);
    Ok(Json(related))
}

async fn graph_metadata_http(
    Path(id): Path<String>,
    Query(query): Query<GraphMetadataQuery>,
) -> Result<Json<Option<LinkGraphMetadata>>, (StatusCode, Json<Value>)> {
    let index = build_index(query.root_dir.as_deref())?;
    Ok(Json(index.metadata(&id)))
}

async fn graph_toc_http(
    Query(query): Query<GraphTocQuery>,
) -> Result<Json<Vec<LinkGraphDocument>>, (StatusCode, Json<Value>)> {
    let index = build_index(query.root_dir.as_deref())?;
    let limit = query.limit.clamp(1, MAX_TOC_LIMIT);
    Ok(Json(index.toc(limit)))
}

async fn graph_stats_http(
    Query(query): Query<GraphStatsQuery>,
) -> Result<Json<LinkGraphStats>, (StatusCode, Json<Value>)> {
    let index = build_index(query.root_dir.as_deref())?;
    Ok(Json(index.stats()))
}

fn build_index(root_dir: Option<&str>) -> Result<LinkGraphIndex, (StatusCode, Json<Value>)> {
    let root_dir = root_dir.unwrap_or(".").trim();
    if root_dir.is_empty() {
        return Err(bad_request("`root_dir` must be non-empty when provided"));
    }
    let root = PathBuf::from(root_dir);
    LinkGraphIndex::build(&root).map_err(|error| {
        internal_http_error(
            format!("failed to build LinkGraph index at `{root_dir}`: {error}").as_str(),
        )
    })
}

fn parse_search_options(raw: Option<Value>) -> Result<LinkGraphSearchOptions, String> {
    let Some(mut value) = raw else {
        return Ok(LinkGraphSearchOptions::default());
    };
    if let Value::Object(ref mut map) = value {
        map.remove("schema");
    }
    let options: LinkGraphSearchOptions =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    options.validate()?;
    Ok(options)
}

fn normalize_limit(raw: Option<usize>, default_limit: usize, max_limit: usize) -> usize {
    raw.unwrap_or(default_limit).clamp(1, max_limit)
}

fn parse_direction(raw: Option<&str>) -> LinkGraphDirection {
    match raw.unwrap_or("both").trim().to_lowercase().as_str() {
        "incoming" | "to" => LinkGraphDirection::Incoming,
        "outgoing" | "from" => LinkGraphDirection::Outgoing,
        _ => LinkGraphDirection::Both,
    }
}

fn default_neighbors_hops() -> usize {
    DEFAULT_NEIGHBORS_HOPS
}

fn default_neighbors_limit() -> usize {
    DEFAULT_NEIGHBORS_LIMIT
}

fn default_related_distance() -> usize {
    DEFAULT_RELATED_DISTANCE
}

fn default_related_limit() -> usize {
    DEFAULT_RELATED_LIMIT
}

fn default_toc_limit() -> usize {
    DEFAULT_TOC_LIMIT
}

fn bad_request(details: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": "invalid wendao request",
            "details": details,
        })),
    )
}

fn internal_http_error(error: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "wendao search failed",
            "details": error,
        })),
    )
}
