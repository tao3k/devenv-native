use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use redis::Client as ValkeyClient;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, RwLock};

use super::types::{ToolRuntimeCallResult, ToolRuntimeListRequestParams, ToolRuntimeListResult};

const DISCOVER_TOOL_NAME: &str = "skill.discover";
const JSON_RPC_VERSION: &str = "2.0";
const DEFAULT_PROTOCOL_VERSION: &str = "2025-06-18";
const HEADER_SESSION_ID: &str = "Mcp-Session-Id";
const JSON_MIME_TYPE: &str = "application/json";
const EVENT_STREAM_MIME_TYPE: &str = "text/event-stream";

/// Connection and retry policy for the remote tool-runtime client pool.
#[derive(Clone, Debug, Serialize)]
pub struct ToolPoolConnectConfig {
    /// Desired logical pool width used by higher-level configuration surfaces.
    pub pool_size: usize,
    /// Timeout budget for the initial JSON-RPC handshake.
    pub handshake_timeout_secs: u64,
    /// Maximum number of reconnect attempts for a failing operation.
    pub connect_retries: u32,
    /// Backoff between reconnect attempts in milliseconds.
    pub connect_retry_backoff_ms: u64,
    /// Timeout budget for each `tools/list` or `tools/call` request.
    pub tool_timeout_secs: u64,
    /// TTL for the in-process `tools/list` cache in milliseconds.
    pub list_tools_cache_ttl_ms: u64,
}

/// Snapshot of the in-process `tools/list` cache counters.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ToolListCacheStatsSnapshot {
    /// Total `tools/list` requests seen by the pool.
    pub requests_total: u64,
    /// Number of requests served from the in-process cache.
    pub cache_hits: u64,
    /// Number of requests that missed the in-process cache.
    pub cache_misses: u64,
    /// Number of successful cache refreshes.
    pub cache_refreshes: u64,
    /// Number of refresh attempts that failed.
    pub refresh_errors: u64,
    /// Unix timestamp in milliseconds of the last successful refresh.
    pub last_refresh_unix_ms: Option<u64>,
}

/// Snapshot of the read-through discover-cache counters.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ToolDiscoverCacheStatsSnapshot {
    /// Total discover-cache lookups attempted.
    pub requests_total: u64,
    /// Number of discover-cache hits.
    pub cache_hits: u64,
    /// Number of discover-cache misses.
    pub cache_misses: u64,
    /// Number of successful cache writes.
    pub cache_writes: u64,
    /// Cache hit rate expressed as a percentage in the `0..=100` range.
    pub hit_rate_pct: f64,
}

/// Configuration for the Valkey-backed discover read-through cache.
#[derive(Clone, Debug)]
pub struct ToolDiscoverCacheConfig {
    /// Valkey connection string.
    pub valkey_url: String,
    /// Prefix used for discover-cache keys.
    pub key_prefix: String,
    /// Cache TTL in seconds.
    pub ttl_secs: u64,
}

/// Runtime metadata for the discover-cache backend currently in use.
#[derive(Clone, Debug)]
pub struct ToolDiscoverCacheRuntimeInfo {
    /// Backend name, for example `valkey`.
    pub backend: &'static str,
    /// Effective TTL in seconds.
    pub ttl_secs: u64,
}

#[derive(Debug)]
struct ToolListCacheStats {
    requests_total: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_refreshes: AtomicU64,
    refresh_errors: AtomicU64,
    last_refresh_unix_ms: AtomicU64,
}

impl Default for ToolListCacheStats {
    fn default() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_refreshes: AtomicU64::new(0),
            refresh_errors: AtomicU64::new(0),
            last_refresh_unix_ms: AtomicU64::new(0),
        }
    }
}

#[derive(Debug)]
struct ToolDiscoverCacheStats {
    requests_total: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_writes: AtomicU64,
}

impl Default for ToolDiscoverCacheStats {
    fn default() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_writes: AtomicU64::new(0),
        }
    }
}

/// Valkey-backed read-through cache for `skill.discover` tool calls.
#[derive(Debug)]
pub struct ToolDiscoverReadThroughCache {
    client: ValkeyClient,
    key_prefix: String,
    ttl_secs: u64,
    stats: Arc<ToolDiscoverCacheStats>,
}

impl ToolDiscoverReadThroughCache {
    /// Builds a discover-cache client from runtime configuration.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey client cannot be constructed from the
    /// provided URL.
    pub fn from_config(config: ToolDiscoverCacheConfig) -> Result<Self> {
        let client = ValkeyClient::open(config.valkey_url.clone())
            .with_context(|| format!("open discover cache valkey client: {}", config.valkey_url))?;
        Ok(Self {
            client,
            key_prefix: config.key_prefix,
            ttl_secs: config.ttl_secs,
            stats: Arc::new(ToolDiscoverCacheStats::default()),
        })
    }

    #[must_use]
    /// Returns static runtime metadata about the discover-cache backend.
    pub fn runtime_info(&self) -> ToolDiscoverCacheRuntimeInfo {
        ToolDiscoverCacheRuntimeInfo {
            backend: "valkey",
            ttl_secs: self.ttl_secs,
        }
    }

    #[must_use]
    /// Returns a point-in-time snapshot of discover-cache counters.
    pub fn stats_snapshot(&self) -> ToolDiscoverCacheStatsSnapshot {
        let requests_total = self.stats.requests_total.load(Ordering::Relaxed);
        let cache_hits = self.stats.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.stats.cache_misses.load(Ordering::Relaxed);
        let cache_writes = self.stats.cache_writes.load(Ordering::Relaxed);
        let hit_rate_pct = if requests_total == 0 {
            0.0
        } else {
            ratio_pct(cache_hits, requests_total)
        };
        ToolDiscoverCacheStatsSnapshot {
            requests_total,
            cache_hits,
            cache_misses,
            cache_writes,
            hit_rate_pct,
        }
    }

    /// Looks up a cached `skill.discover` result by normalized tool arguments.
    ///
    /// # Errors
    ///
    /// Returns an error when the cache key cannot be serialized, Valkey cannot
    /// be reached, or a cached payload cannot be decoded.
    pub async fn lookup(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<Option<ToolRuntimeCallResult>> {
        self.stats.requests_total.fetch_add(1, Ordering::Relaxed);
        let key = self.cache_key(tool_name, arguments)?;
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .context("connect discover cache valkey")?;
        let payload: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut connection)
            .await
            .with_context(|| format!("discover cache GET {key}"))?;
        if let Some(payload) = payload {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            let result = serde_json::from_str(&payload)
                .with_context(|| format!("decode discover cache payload for {key}"))?;
            Ok(Some(result))
        } else {
            self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }

    /// Stores a successful `skill.discover` result in the read-through cache.
    ///
    /// # Errors
    ///
    /// Returns an error when the cache key cannot be serialized, the result
    /// payload cannot be encoded, or Valkey write operations fail.
    pub async fn store(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
        result: &ToolRuntimeCallResult,
    ) -> Result<()> {
        let key = self.cache_key(tool_name, arguments)?;
        let payload = serde_json::to_string(result)
            .with_context(|| format!("encode discover cache payload for {key}"))?;
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .context("connect discover cache valkey")?;
        let _: () = redis::cmd("SETEX")
            .arg(&key)
            .arg(self.ttl_secs)
            .arg(payload)
            .query_async(&mut connection)
            .await
            .with_context(|| format!("discover cache SETEX {key}"))?;
        self.stats.cache_writes.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn cache_key(&self, tool_name: &str, arguments: &serde_json::Value) -> Result<String> {
        let normalized = normalize_json(arguments);
        let canonical =
            serde_json::to_string(&normalized).context("serialize discover cache key")?;
        let mut digest = Sha256::new();
        digest.update(tool_name.as_bytes());
        digest.update([0]);
        digest.update(canonical.as_bytes());
        let digest_hex = hex::encode(digest.finalize());
        Ok(format!("{}:{tool_name}:{digest_hex}", self.key_prefix))
    }
}

type ToolClientService = ToolRuntimeSessionClient;

#[derive(Debug)]
struct ToolRuntimeSessionClient {
    http_client: reqwest::Client,
    url: String,
    session_id: String,
    next_request_id: AtomicU64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolRuntimeInitializeRequestParams {
    protocol_version: &'static str,
    capabilities: Value,
    client_info: ToolRuntimeImplementation,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolRuntimeImplementation {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icons: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    website_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct ToolRuntimeJsonRpcRequest<P> {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<P>,
}

#[derive(Debug, Serialize)]
struct ToolRuntimeJsonRpcNotification<P> {
    jsonrpc: &'static str,
    method: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<P>,
}

#[derive(Debug, Deserialize)]
struct ToolRuntimeJsonRpcResponse<R> {
    jsonrpc: String,
    id: Value,
    result: Option<R>,
    error: Option<ToolRuntimeJsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct ToolRuntimeJsonRpcError {
    code: i64,
    message: String,
    #[serde(default)]
    data: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolRuntimeInitializeResult {
    protocol_version: String,
    #[serde(default)]
    capabilities: Value,
    server_info: ToolRuntimeImplementation,
    #[serde(default)]
    instructions: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolRuntimeCallWireResult {
    #[serde(default)]
    content: Vec<ToolRuntimeContentBlock>,
    #[serde(default)]
    is_error: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ToolRuntimeContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolRuntimeCallRequestParams {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<Map<String, Value>>,
}

#[derive(Debug)]
struct ToolRuntimePostResponse {
    session_id: Option<String>,
    payload: Option<Value>,
    accepted: bool,
}

impl ToolRuntimeSessionClient {
    async fn connect(url: &str, handshake_timeout_secs: u64) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .build()
            .context("build tool runtime HTTP client")?;
        let timeout = Duration::from_secs(handshake_timeout_secs.max(1));
        tokio::time::timeout(timeout, async {
            let initialize_request = ToolRuntimeJsonRpcRequest {
                jsonrpc: JSON_RPC_VERSION,
                id: 0,
                method: "initialize",
                params: Some(ToolRuntimeInitializeRequestParams {
                    protocol_version: DEFAULT_PROTOCOL_VERSION,
                    capabilities: Value::Object(Map::new()),
                    client_info: ToolRuntimeImplementation::from_build_env(),
                }),
            };
            let initialize_response =
                post_runtime_message(&http_client, url, None, &initialize_request, "initialize")
                    .await?;
            let session_id = initialize_response
                .session_id
                .clone()
                .ok_or_else(|| anyhow!("tool runtime initialize did not return a session id"))?;
            let initialize_result: ToolRuntimeInitializeResult =
                decode_rpc_result(initialize_response, 0, "initialize")?;
            if initialize_result.protocol_version.is_empty() {
                return Err(anyhow!(
                    "tool runtime initialize returned an empty protocol version"
                ));
            }
            let _ = (
                initialize_result.capabilities,
                initialize_result.server_info,
                initialize_result.instructions,
            );

            let client = Self {
                http_client,
                url: url.to_string(),
                session_id,
                next_request_id: AtomicU64::new(1),
            };
            client.send_initialized_notification().await?;
            Ok(client)
        })
        .await
        .map_err(|_| {
            anyhow!(
                "tool runtime handshake timed out after {}s",
                timeout.as_secs()
            )
        })?
    }

    async fn list_tools(
        &self,
        params: Option<ToolRuntimeListRequestParams>,
    ) -> Result<ToolRuntimeListResult> {
        self.post_request("tools/list", params).await
    }

    async fn call_tool(
        &self,
        name: String,
        arguments: Option<Value>,
    ) -> Result<ToolRuntimeCallResult> {
        let result: ToolRuntimeCallWireResult = self
            .post_request(
                "tools/call",
                Some(ToolRuntimeCallRequestParams {
                    name,
                    arguments: arguments.and_then(|value| value.as_object().cloned()),
                }),
            )
            .await?;
        Ok(ToolRuntimeCallResult {
            text_segments: result
                .content
                .into_iter()
                .filter(|content| content.kind == "text")
                .filter_map(|content| content.text)
                .collect(),
            is_error: result.is_error.unwrap_or(false),
        })
    }

    async fn post_request<P, R>(&self, method: &'static str, params: Option<P>) -> Result<R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let request = ToolRuntimeJsonRpcRequest {
            jsonrpc: JSON_RPC_VERSION,
            id,
            method,
            params,
        };
        let response = post_runtime_message(
            &self.http_client,
            &self.url,
            Some(self.session_id.as_str()),
            &request,
            method,
        )
        .await?;
        decode_rpc_result(response, id, method)
    }

    async fn send_initialized_notification(&self) -> Result<()> {
        let notification = ToolRuntimeJsonRpcNotification::<Value> {
            jsonrpc: JSON_RPC_VERSION,
            method: "notifications/initialized",
            params: None,
        };
        let response = post_runtime_message(
            &self.http_client,
            &self.url,
            Some(self.session_id.as_str()),
            &notification,
            "notifications/initialized",
        )
        .await?;
        if !response.accepted {
            return Err(anyhow!(
                "notifications/initialized should return an accepted response"
            ));
        }
        Ok(())
    }
}

impl ToolRuntimeImplementation {
    fn from_build_env() -> Self {
        Self {
            name: env!("CARGO_CRATE_NAME").to_owned(),
            title: None,
            version: env!("CARGO_PKG_VERSION").to_owned(),
            description: None,
            icons: None,
            website_url: None,
        }
    }
}

#[derive(Debug)]
struct ToolRuntimeState {
    service: Option<Arc<ToolClientService>>,
}

#[derive(Debug)]
struct CachedToolList {
    value: ToolRuntimeListResult,
    expires_at: Instant,
}

/// Lazy reconnecting client facade for the remote tool runtime.
#[derive(Debug)]
pub struct ToolClientPool {
    url: String,
    config: ToolPoolConnectConfig,
    state: Mutex<ToolRuntimeState>,
    list_cache: RwLock<Option<CachedToolList>>,
    list_stats: Arc<ToolListCacheStats>,
    discover_cache: Option<Arc<ToolDiscoverReadThroughCache>>,
}

impl ToolClientPool {
    /// Lists tools from the remote runtime, using the in-process cache when
    /// pagination parameters are absent.
    ///
    /// # Errors
    ///
    /// Returns an error when the remote runtime cannot be reached, the request
    /// times out, or the remote response is invalid.
    pub async fn list_tools(
        &self,
        params: Option<ToolRuntimeListRequestParams>,
    ) -> Result<ToolRuntimeListResult> {
        self.list_stats
            .requests_total
            .fetch_add(1, Ordering::Relaxed);
        if params.is_none()
            && let Some(cached) = self.read_cached_tools().await
        {
            self.list_stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(cached);
        }
        self.list_stats.cache_misses.fetch_add(1, Ordering::Relaxed);

        let timeout_secs = self.config.tool_timeout_secs.max(1);
        let list = self
            .call_with_retry("tools/list", timeout_secs, |service| {
                let params = params.clone();
                async move {
                    service
                        .list_tools(params)
                        .await
                        .map_err(|error| anyhow!("tools/list failed: {error}"))
                }
            })
            .await?;
        if params.is_none() {
            self.store_cached_tools(list.clone()).await;
        }
        Ok(list)
    }

    /// Calls one tool on the remote runtime and normalizes its textual output.
    ///
    /// # Errors
    ///
    /// Returns an error when the remote runtime cannot be reached, the request
    /// times out, or the remote response is invalid.
    pub async fn call_tool(
        &self,
        name: String,
        arguments: Option<serde_json::Value>,
    ) -> Result<ToolRuntimeCallResult> {
        if let Some(discover_cache) = self.discover_cache.as_ref()
            && name == DISCOVER_TOOL_NAME
            && let Some(arguments) = arguments.as_ref()
            && let Some(result) = discover_cache.lookup(&name, arguments).await?
        {
            return Ok(result);
        }

        let timeout_secs = self.config.tool_timeout_secs.max(1);
        let result = self
            .call_with_retry("tools/call", timeout_secs, |service| {
                let name = name.clone();
                let arguments = arguments.clone();
                async move {
                    service
                        .call_tool(name, arguments)
                        .await
                        .map_err(|error| anyhow!("tools/call failed: {error}"))
                }
            })
            .await?;

        if let Some(discover_cache) = self.discover_cache.as_ref()
            && name == DISCOVER_TOOL_NAME
            && let Some(arguments) = arguments.as_ref()
            && !result.is_error
            && let Err(error) = discover_cache.store(&name, arguments, &result).await
        {
            tracing::warn!(
                event = "tool_runtime.discover_cache.store_failed",
                tool_name = %name,
                error = %error,
                "discover cache store failed; continuing without cached write"
            );
        }

        Ok(result)
    }

    #[must_use]
    /// Returns a point-in-time snapshot of `tools/list` cache counters.
    pub fn tools_list_cache_stats_snapshot(&self) -> ToolListCacheStatsSnapshot {
        let last_refresh_unix_ms =
            match self.list_stats.last_refresh_unix_ms.load(Ordering::Relaxed) {
                0 => None,
                value => Some(value),
            };
        ToolListCacheStatsSnapshot {
            requests_total: self.list_stats.requests_total.load(Ordering::Relaxed),
            cache_hits: self.list_stats.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.list_stats.cache_misses.load(Ordering::Relaxed),
            cache_refreshes: self.list_stats.cache_refreshes.load(Ordering::Relaxed),
            refresh_errors: self.list_stats.refresh_errors.load(Ordering::Relaxed),
            last_refresh_unix_ms,
        }
    }

    #[must_use]
    /// Returns discover-cache counters when the read-through cache is enabled.
    pub fn discover_cache_stats_snapshot(&self) -> Option<ToolDiscoverCacheStatsSnapshot> {
        self.discover_cache
            .as_ref()
            .map(|cache| cache.stats_snapshot())
    }

    async fn call_with_retry<T, F, Fut>(
        &self,
        op_name: &'static str,
        timeout_secs: u64,
        mut op: F,
    ) -> Result<T>
    where
        F: FnMut(Arc<ToolClientService>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let max_attempts = self.config.connect_retries.max(1);
        let timeout = Duration::from_secs(timeout_secs);
        let backoff = Duration::from_millis(self.config.connect_retry_backoff_ms.max(1));

        let mut last_error = None;
        for attempt in 1..=max_attempts {
            let service = match self.connected_service().await {
                Ok(service) => service,
                Err(error) => {
                    last_error = Some(error);
                    if attempt < max_attempts {
                        tokio::time::sleep(backoff).await;
                    }
                    continue;
                }
            };
            let outcome = tokio::time::timeout(timeout, op(service)).await;
            match outcome {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(error)) => {
                    last_error = Some(error);
                    self.invalidate_connection().await;
                }
                Err(_) => {
                    self.invalidate_connection().await;
                    last_error = Some(anyhow!("{op_name} timed out after {timeout_secs}s"));
                }
            }
            if attempt < max_attempts {
                tokio::time::sleep(backoff).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("{op_name} failed without error")))
    }

    async fn connected_service(&self) -> Result<Arc<ToolClientService>> {
        let mut guard = self.state.lock().await;
        if let Some(service) = guard.service.as_ref() {
            return Ok(Arc::clone(service));
        }
        let service =
            Arc::new(connect_service(&self.url, self.config.handshake_timeout_secs).await?);
        guard.service = Some(Arc::clone(&service));
        Ok(service)
    }

    async fn invalidate_connection(&self) {
        let mut guard = self.state.lock().await;
        guard.service = None;
    }

    async fn read_cached_tools(&self) -> Option<ToolRuntimeListResult> {
        let guard = self.list_cache.read().await;
        let cached = guard.as_ref()?;
        (Instant::now() < cached.expires_at).then(|| cached.value.clone())
    }

    async fn store_cached_tools(&self, list: ToolRuntimeListResult) {
        self.list_stats
            .cache_refreshes
            .fetch_add(1, Ordering::Relaxed);
        self.list_stats
            .last_refresh_unix_ms
            .store(now_unix_ms().unwrap_or_default(), Ordering::Relaxed);
        let ttl_ms = self.config.list_tools_cache_ttl_ms.max(1);
        let expires_at = Instant::now() + Duration::from_millis(ttl_ms);
        let mut guard = self.list_cache.write().await;
        *guard = Some(CachedToolList {
            value: list,
            expires_at,
        });
    }
}

pub(crate) async fn connect_tool_pool_backend(
    url: &str,
    config: ToolPoolConnectConfig,
    discover_cache: Option<Arc<ToolDiscoverReadThroughCache>>,
) -> Result<ToolClientPool> {
    let pool = ToolClientPool {
        url: url.to_string(),
        config,
        state: Mutex::new(ToolRuntimeState { service: None }),
        list_cache: RwLock::new(None),
        list_stats: Arc::new(ToolListCacheStats::default()),
        discover_cache,
    };
    let service = connect_service_with_retry(url, &pool.config).await?;
    {
        let mut guard = pool.state.lock().await;
        guard.service = Some(Arc::new(service));
    }
    Ok(pool)
}

async fn connect_service(url: &str, handshake_timeout_secs: u64) -> Result<ToolClientService> {
    ToolRuntimeSessionClient::connect(url, handshake_timeout_secs).await
}

async fn connect_service_with_retry(
    url: &str,
    config: &ToolPoolConnectConfig,
) -> Result<ToolClientService> {
    let max_attempts = config.connect_retries.max(1);
    let backoff = Duration::from_millis(config.connect_retry_backoff_ms.max(1));
    let mut last_error = None;
    for attempt in 1..=max_attempts {
        match connect_service(url, config.handshake_timeout_secs).await {
            Ok(service) => return Ok(service),
            Err(error) => {
                last_error = Some(error);
                if attempt < max_attempts {
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    match last_error {
        Some(error) => Err(anyhow!(
            "connect failed after {max_attempts} attempts: {error}"
        )),
        None => Err(anyhow!(
            "connect failed after {max_attempts} attempts without an error"
        )),
    }
}

fn normalize_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let sorted: BTreeMap<String, serde_json::Value> = map
                .iter()
                .map(|(key, value)| (key.clone(), normalize_json(value)))
                .collect();
            let normalized = sorted.into_iter().collect();
            serde_json::Value::Object(normalized)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(normalize_json).collect())
        }
        _ => value.clone(),
    }
}

fn now_unix_ms() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
}

async fn post_runtime_message<M: Serialize>(
    http_client: &reqwest::Client,
    url: &str,
    session_id: Option<&str>,
    message: &M,
    method: &'static str,
) -> Result<ToolRuntimePostResponse> {
    let mut request = http_client
        .post(url)
        .header(
            ACCEPT,
            format!("{JSON_MIME_TYPE}, {EVENT_STREAM_MIME_TYPE}"),
        )
        .header(CONTENT_TYPE, JSON_MIME_TYPE);
    if let Some(session_id) = session_id {
        request = request.header(HEADER_SESSION_ID, session_id);
    }
    let response = request
        .json(message)
        .send()
        .await
        .with_context(|| format!("send tool runtime {method} request"))?;
    let status = response.status();
    let response_session_id = response
        .headers()
        .get(HEADER_SESSION_ID)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    if matches!(
        status,
        reqwest::StatusCode::ACCEPTED | reqwest::StatusCode::NO_CONTENT
    ) {
        return Ok(ToolRuntimePostResponse {
            session_id: response_session_id,
            payload: None,
            accepted: true,
        });
    }

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!(
            "tool runtime {method} request failed with status {status}: {body}"
        ));
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    let payload = match content_type.as_deref() {
        Some(content_type) if content_type.starts_with(EVENT_STREAM_MIME_TYPE) => {
            Some(parse_first_sse_message(response, method).await?)
        }
        Some(content_type) if content_type.starts_with(JSON_MIME_TYPE) => Some(
            response
                .json()
                .await
                .with_context(|| format!("decode tool runtime {method} JSON response"))?,
        ),
        Some(content_type) => {
            return Err(anyhow!(
                "tool runtime {method} returned unsupported content-type: {content_type}"
            ));
        }
        None => {
            return Err(anyhow!(
                "tool runtime {method} response missing content-type header"
            ));
        }
    };

    Ok(ToolRuntimePostResponse {
        session_id: response_session_id,
        payload,
        accepted: false,
    })
}

fn ratio_pct(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    let basis_points = numerator.saturating_mul(10_000) / denominator;
    f64::from(u32::try_from(basis_points).unwrap_or(10_000)) / 100.0
}

async fn parse_first_sse_message(
    response: reqwest::Response,
    method: &'static str,
) -> Result<Value> {
    let payload = response
        .bytes()
        .await
        .with_context(|| format!("read tool runtime {method} SSE response body"))?;
    let buffer = String::from_utf8_lossy(&payload);
    try_parse_sse_message(&buffer)
        .with_context(|| format!("decode tool runtime {method} SSE response"))?
        .ok_or_else(|| {
            anyhow!("tool runtime {method} SSE response ended before a JSON-RPC message arrived")
        })
}

fn try_parse_sse_message(buffer: &str) -> Result<Option<Value>> {
    let normalized = buffer.replace("\r\n", "\n");
    if !normalized.contains("\n\n") {
        return Ok(None);
    }

    for event in normalized.split("\n\n") {
        let payload = event
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim_start)
            .collect::<Vec<_>>()
            .join("\n");
        if payload.trim().is_empty() {
            continue;
        }
        let message =
            serde_json::from_str(&payload).context("deserialize tool runtime SSE payload")?;
        return Ok(Some(message));
    }

    Ok(None)
}

fn decode_rpc_result<R>(
    response: ToolRuntimePostResponse,
    request_id: u64,
    method: &'static str,
) -> Result<R>
where
    R: DeserializeOwned,
{
    let payload = response
        .payload
        .ok_or_else(|| anyhow!("tool runtime {method} returned an empty response body"))?;
    let response: ToolRuntimeJsonRpcResponse<R> = serde_json::from_value(payload)
        .with_context(|| format!("deserialize tool runtime {method} JSON-RPC response"))?;
    if response.jsonrpc != JSON_RPC_VERSION {
        return Err(anyhow!(
            "tool runtime {method} returned unexpected jsonrpc version: {}",
            response.jsonrpc
        ));
    }
    match &response.id {
        Value::Number(number) if number.as_u64() == Some(request_id) => {}
        other => {
            return Err(anyhow!(
                "tool runtime {method} returned mismatched response id: expected {request_id}, got {other}"
            ));
        }
    }
    if let Some(error) = response.error {
        let error_data = error
            .data
            .map(|data| format!(" data={data}"))
            .unwrap_or_default();
        return Err(anyhow!(
            "tool runtime {method} failed with code {}: {}{}",
            error.code,
            error.message,
            error_data
        ));
    }
    response
        .result
        .ok_or_else(|| anyhow!("tool runtime {method} response missing result"))
}

#[cfg(test)]
#[path = "../../tests/unit/tool_runtime/bridge.rs"]
mod tests;
