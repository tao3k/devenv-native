//! Gateway command execution.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
#[cfg(feature = "zhenfa-router")]
use arrow_flight::flight_service_server::FlightServiceServer;
use axum::Json;
use axum::error_handling::HandleErrorLayer;
use axum::extract::Request;
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
#[cfg(feature = "zhenfa-router")]
use axum::routing::any_service;
use axum::routing::{Router, get, post};
use log::{error, info, warn};
use tokio::sync::mpsc;
#[cfg(feature = "zhenfa-router")]
use tonic_web::GrpcWebLayer;
#[cfg(feature = "zhenfa-router")]
use tower::Layer;
use tower::{BoxError, ServiceBuilder};

use crate::bin_support::wendao::execute::gateway::{
    config::{
        GatewayRuntimeTomlConfig, get_gateway_runtime_from_config, resolve_bind_addr,
        resolve_config_path, resolve_port, resolve_webhook_config,
    },
    health::health,
    query::{GATEWAY_QUERY_AXUM_PATH, GATEWAY_RESPONSES_AXUM_PATH, query, responses},
    registry::build_plugin_registry,
    state::AppState,
    status::{notify_status, stats},
};
use crate::bin_support::wendao::types::{Cli, GatewayArgs, GatewayCommand, GatewayStartArgs};
use crate::contracts::routes as openapi_paths;
#[cfg(feature = "zhenfa-router")]
use crate::studio::build_studio_flight_service_with_weights;
use crate::studio::{
    GatewayStartupHealthReport, describe_gateway_startup_health, probe_gateway_startup_health,
    studio_routes,
};
#[cfg(feature = "zhenfa-router")]
use crate::transport::{
    EffectiveRerankFlightHostSettings, EffectiveRerankFlightHostSettingsInput, RerankScoreWeights,
    rerank_score_weights_from_env,
    resolve_effective_rerank_flight_host_settings as resolve_runtime_effective_rerank_flight_host_settings,
};
use xiuxian_config_core::{lookup_bool_flag, lookup_positive_parsed};
use xiuxian_wendao::LinkGraphIndex;
#[cfg(feature = "zhenfa-router")]
use xiuxian_wendao::link_graph::resolve_link_graph_rerank_flight_runtime_settings;
use xiuxian_zhenfa::{NotificationService, ZhenfaSignal, notification_worker};

const GATEWAY_LISTEN_BACKLOG_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_LISTEN_BACKLOG";
const GATEWAY_STUDIO_CONCURRENCY_LIMIT_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_STUDIO_CONCURRENCY_LIMIT";
const GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS";
const GATEWAY_FLIGHT_CONCURRENCY_LIMIT_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_FLIGHT_CONCURRENCY_LIMIT";
const GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS";
const GATEWAY_FLIGHT_GRPC_WEB_ENABLED_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_FLIGHT_GRPC_WEB_ENABLED";
const GATEWAY_BEARER_TOKEN_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_BEARER_TOKEN";
const DEFAULT_GATEWAY_LISTEN_BACKLOG: u32 = 2048;
const MIN_GATEWAY_LISTEN_BACKLOG: u32 = 128;
const MAX_GATEWAY_LISTEN_BACKLOG: u32 = 8192;
const DEFAULT_GATEWAY_STUDIO_CONCURRENCY_FALLBACK: usize = 8;
const MIN_GATEWAY_STUDIO_CONCURRENCY_LIMIT: usize = 32;
const MAX_GATEWAY_STUDIO_CONCURRENCY_LIMIT: usize = 128;
const DEFAULT_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS: u64 = 15;
const MIN_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS: u64 = 5;
const MAX_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS: u64 = 60;
const MIN_GATEWAY_FLIGHT_CONCURRENCY_LIMIT: usize = 4;
const MAX_GATEWAY_FLIGHT_CONCURRENCY_LIMIT: usize = 128;
const MIN_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS: u64 = 5;
const MAX_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS: u64 = 900;
const DEFAULT_GATEWAY_FLIGHT_GRPC_WEB_ENABLED: bool = false;
pub(crate) const GATEWAY_FLIGHT_SERVICE_AXUM_PATH: &str =
    "/arrow.flight.protocol.FlightService/{*grpc_method}";
#[cfg(feature = "zhenfa-router")]
const DEFAULT_GATEWAY_SEARCH_FLIGHT_RERANK_DIMENSION: usize = 3;

/// Handle the gateway command.
pub(crate) async fn handle(
    cli: &Cli,
    args: &GatewayArgs,
    index: Option<&LinkGraphIndex>,
) -> Result<()> {
    match &args.command {
        GatewayCommand::Start(start_args) => handle_start(cli, start_args, index).await,
    }
}

/// Handle the `gateway start` subcommand.
async fn handle_start(
    cli: &Cli,
    args: &GatewayStartArgs,
    index: Option<&LinkGraphIndex>,
) -> Result<()> {
    let config_path = resolve_config_path(cli.config_file.as_deref());
    let plugin_registry = build_plugin_registry()?;
    let startup_health = probe_gateway_startup_health(plugin_registry.as_ref());
    ensure_gateway_startup_health(&startup_health)?;

    // Resolve port: CLI arg > config file > default
    let port = resolve_port(args.port, config_path.as_deref());

    // 1. Start Webhook notification sidecar
    let (signal_tx, signal_rx) = mpsc::unbounded_channel::<ZhenfaSignal>();

    // Configure webhook: TOML > env var > defaults
    let webhook_config = resolve_webhook_config(config_path.as_deref());
    let effective_webhook_url =
        (!webhook_config.url.is_empty()).then(|| webhook_config.url.clone());

    let notification_service = Arc::new(NotificationService::new(webhook_config));

    // Spawn the notification worker as a background task
    tokio::spawn(notification_worker(
        signal_rx,
        Arc::clone(&notification_service),
    ));
    info!(
        "Gateway: Notification worker started (id={})",
        notification_service.id()
    );

    // 2. Create app state with index and signal channel
    // Note: Julia/Modelica plugins should be registered here if this crate
    // depended on them. Since it doesn't (to avoid circular dependency),
    // they are currently empty. A separate aggregator crate would be needed
    // to provide a pre-populated registry.
    let app_state = Arc::new(AppState::new_for_gateway_start(
        index.map(|i| Arc::new(i.clone())),
        Some(signal_tx),
        effective_webhook_url,
        config_path.as_deref(),
        plugin_registry,
    ));

    let gateway_runtime = get_gateway_runtime_from_config(config_path.as_deref());
    let listen_backlog = gateway_listen_backlog(gateway_runtime);
    let studio_concurrency_limit = gateway_studio_concurrency_limit(gateway_runtime);
    let studio_request_timeout = gateway_studio_request_timeout(gateway_runtime);
    let flight_concurrency_limit = gateway_flight_concurrency_limit(studio_concurrency_limit);
    let flight_request_timeout = gateway_flight_request_timeout(studio_request_timeout);
    let flight_grpc_web_enabled = gateway_flight_grpc_web_enabled();
    let bearer_token = gateway_bearer_token();
    let bearer_auth_required = bearer_token.is_some();

    // 3. Build the Axum router
    let app = build_gateway_router(
        app_state.clone(),
        studio_concurrency_limit,
        studio_request_timeout,
        flight_concurrency_limit,
        flight_request_timeout,
        flight_grpc_web_enabled,
        bearer_token,
    )?;

    // 4. Start the server
    let bind_addr = resolve_bind_addr(config_path.as_deref());
    let addr = SocketAddr::from((bind_addr, port));
    info!("Starting Wendao Gateway on {addr}");
    info!(
        "Gateway listener backlog={listen_backlog}, studio concurrency limit={studio_concurrency_limit}, studio request timeout={}s",
        studio_request_timeout.as_secs()
    );
    #[cfg(feature = "zhenfa-router")]
    info!(
        "Gateway Flight concurrency limit={flight_concurrency_limit}, Flight request timeout={}s, gRPC-Web enabled={flight_grpc_web_enabled}",
        flight_request_timeout.as_secs(),
    );
    info!(
        "Gateway bearer auth required={}",
        if bearer_auth_required {
            "true"
        } else {
            "false"
        }
    );
    info!("Endpoints:");
    info!(
        "  - GET {}  - Health check",
        openapi_paths::API_HEALTH_AXUM_PATH
    );
    info!(
        "  - GET {}   - Graph statistics",
        openapi_paths::API_STATS_AXUM_PATH
    );
    info!(
        "  - GET {}  - Notification service status",
        openapi_paths::API_NOTIFY_AXUM_PATH
    );
    info!("  - POST {GATEWAY_RESPONSES_AXUM_PATH}  - Public JSON/SSE query response");
    #[cfg(feature = "zhenfa-router")]
    info!("  - POST {GATEWAY_FLIGHT_SERVICE_AXUM_PATH}  - Arrow Flight business plane");

    let socket = tokio::net::TcpSocket::new_v4()?;
    socket.set_reuseaddr(true)?;
    socket.bind(addr)?;
    let listener = socket.listen(listen_backlog)?;
    Ok(axum::serve(listener, app)
        .with_graceful_shutdown(gateway_shutdown_signal())
        .await?)
}

async fn gateway_shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            warn!("Gateway failed to install ctrl-c shutdown handler: {error}");
        }
    };

    #[cfg(unix)]
    {
        let terminate = async {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(mut stream) => {
                    stream.recv().await;
                }
                Err(error) => {
                    warn!("Gateway failed to install SIGTERM shutdown handler: {error}");
                    std::future::pending::<()>().await;
                }
            }
        };

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await;
    }

    info!("Gateway shutdown signal received");
}

fn log_gateway_startup_health(report: &GatewayStartupHealthReport) {
    info!("Gateway startup dependency health checks:");
    for line in describe_gateway_startup_health(report) {
        if line.contains("=failed ") {
            error!("  - {line}");
        } else {
            info!("  - {line}");
        }
    }
}

pub(crate) fn ensure_gateway_startup_health(report: &GatewayStartupHealthReport) -> Result<()> {
    log_gateway_startup_health(report);
    if let Some(summary) = report.failure_summary() {
        return Err(anyhow!("gateway startup health checks failed: {summary}"));
    }
    Ok(())
}

pub(crate) fn build_gateway_router(
    app_state: Arc<AppState>,
    studio_concurrency_limit: usize,
    studio_request_timeout: Duration,
    flight_concurrency_limit: usize,
    flight_request_timeout: Duration,
    flight_grpc_web_enabled: bool,
    bearer_token: Option<Arc<str>>,
) -> Result<Router> {
    let studio_app = studio_routes().layer(
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_gateway_service_error))
            .load_shed()
            .timeout(studio_request_timeout)
            .concurrency_limit(studio_concurrency_limit),
    );
    let protected_app = Router::new()
        .route(openapi_paths::API_STATS_AXUM_PATH, get(stats))
        .route(openapi_paths::API_NOTIFY_AXUM_PATH, get(notify_status))
        .route(GATEWAY_QUERY_AXUM_PATH, post(query))
        .route(GATEWAY_RESPONSES_AXUM_PATH, post(responses))
        .merge(studio_app);
    let protected_app = with_gateway_bearer_auth(protected_app, bearer_token.clone());
    let app = Router::new()
        .route(openapi_paths::API_HEALTH_AXUM_PATH, get(health))
        .merge(protected_app)
        .with_state(app_state.clone());

    #[cfg(feature = "zhenfa-router")]
    let app = mount_gateway_flight_service(
        app,
        app_state,
        flight_concurrency_limit,
        flight_request_timeout,
        flight_grpc_web_enabled,
        bearer_token,
    )?;
    #[cfg(not(feature = "zhenfa-router"))]
    let _ = (
        flight_concurrency_limit,
        flight_request_timeout,
        flight_grpc_web_enabled,
    );

    Ok(app)
}

#[derive(Clone)]
struct GatewayBearerAuth {
    token: Arc<str>,
}

fn with_gateway_bearer_auth<S>(router: Router<S>, bearer_token: Option<Arc<str>>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    if let Some(token) = bearer_token {
        router.route_layer(middleware::from_fn_with_state(
            GatewayBearerAuth { token },
            require_gateway_bearer_auth,
        ))
    } else {
        router
    }
}

async fn require_gateway_bearer_auth(
    axum::extract::State(auth): axum::extract::State<GatewayBearerAuth>,
    request: Request,
    next: Next,
) -> Response {
    let expected = format!("Bearer {}", auth.token);
    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == expected);
    if authorized {
        return next.run(request).await;
    }
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": "missing or invalid bearer token",
            "code": "UNAUTHORIZED",
        })),
    )
        .into_response()
}

#[cfg(feature = "zhenfa-router")]
fn mount_gateway_flight_service(
    app: Router,
    app_state: Arc<AppState>,
    flight_concurrency_limit: usize,
    flight_request_timeout: Duration,
    flight_grpc_web_enabled: bool,
    bearer_token: Option<Arc<str>>,
) -> Result<Router> {
    let effective_settings = resolve_gateway_effective_search_host_settings()?;
    let flight_service = build_studio_flight_service_with_weights(
        Arc::new(app_state.studio.search_plane_service()),
        app_state,
        effective_settings.expected_schema_version,
        effective_settings.rerank_dimension,
        effective_settings.rerank_weights,
    )
    .map_err(anyhow::Error::msg)?;
    let flight_service = FlightServiceServer::new(flight_service);
    if flight_grpc_web_enabled {
        let flight_service = GrpcWebLayer::new().layer(flight_service);
        let flight_service = ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_gateway_service_error))
            .load_shed()
            .timeout(flight_request_timeout)
            .concurrency_limit(flight_concurrency_limit)
            .service(flight_service);
        let flight_router = Router::new().route(
            GATEWAY_FLIGHT_SERVICE_AXUM_PATH,
            any_service(flight_service),
        );
        return Ok(app.merge(with_gateway_bearer_auth(flight_router, bearer_token)));
    }

    let flight_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_gateway_service_error))
        .load_shed()
        .timeout(flight_request_timeout)
        .concurrency_limit(flight_concurrency_limit)
        .service(flight_service);
    let flight_router = Router::new().route(
        GATEWAY_FLIGHT_SERVICE_AXUM_PATH,
        any_service(flight_service),
    );
    Ok(app.merge(with_gateway_bearer_auth(flight_router, bearer_token)))
}

#[cfg(feature = "zhenfa-router")]
fn resolve_gateway_effective_search_host_settings() -> Result<EffectiveRerankFlightHostSettings> {
    let file_backed_settings = resolve_link_graph_rerank_flight_runtime_settings();
    let file_backed_weights = file_backed_settings
        .score_weights
        .map(|weights| RerankScoreWeights::new(weights.vector_weight, weights.semantic_weight))
        .transpose()
        .map_err(anyhow::Error::msg)?;
    Ok(resolve_runtime_effective_rerank_flight_host_settings(
        EffectiveRerankFlightHostSettingsInput {
            schema_version_override: None,
            rerank_dimension_override: None,
            file_backed_schema_version: file_backed_settings.schema_version,
            file_backed_weights,
            fallback_dimension: DEFAULT_GATEWAY_SEARCH_FLIGHT_RERANK_DIMENSION,
            fallback_weights: rerank_score_weights_from_env().map_err(anyhow::Error::msg)?,
        },
    ))
}

async fn handle_gateway_service_error(error: BoxError) -> (StatusCode, Json<serde_json::Value>) {
    if error.is::<tower::timeout::error::Elapsed>() {
        log::warn!("Gateway studio router timed out: {error}");
        return (
            StatusCode::GATEWAY_TIMEOUT,
            Json(serde_json::json!({
                "error": "gateway request timed out",
                "code": "GATEWAY_TIMEOUT",
            })),
        );
    }
    log::warn!("Gateway studio router overloaded: {error}");
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "gateway is overloaded",
            "code": "GATEWAY_OVERLOADED",
        })),
    )
}

pub(crate) fn gateway_listen_backlog(runtime_config: Option<GatewayRuntimeTomlConfig>) -> u32 {
    gateway_listen_backlog_with_lookup(runtime_config, &|key| std::env::var(key).ok())
}

pub(crate) fn gateway_listen_backlog_with_lookup(
    runtime_config: Option<GatewayRuntimeTomlConfig>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> u32 {
    runtime_config
        .and_then(|config| config.listen_backlog)
        .or_else(|| lookup_positive_parsed::<u32>(GATEWAY_LISTEN_BACKLOG_ENV, lookup))
        .unwrap_or(DEFAULT_GATEWAY_LISTEN_BACKLOG)
        .clamp(MIN_GATEWAY_LISTEN_BACKLOG, MAX_GATEWAY_LISTEN_BACKLOG)
}

pub(crate) fn gateway_studio_concurrency_limit(
    runtime_config: Option<GatewayRuntimeTomlConfig>,
) -> usize {
    gateway_studio_concurrency_limit_with_lookup(
        runtime_config,
        &|key| std::env::var(key).ok(),
        std::thread::available_parallelism()
            .ok()
            .map(std::num::NonZeroUsize::get),
    )
}

pub(crate) fn gateway_studio_concurrency_limit_with_lookup(
    runtime_config: Option<GatewayRuntimeTomlConfig>,
    lookup: &dyn Fn(&str) -> Option<String>,
    available_parallelism: Option<usize>,
) -> usize {
    runtime_config
        .and_then(|config| config.studio_concurrency_limit)
        .or_else(|| lookup_positive_parsed::<usize>(GATEWAY_STUDIO_CONCURRENCY_LIMIT_ENV, lookup))
        .unwrap_or_else(|| default_gateway_studio_concurrency_limit(available_parallelism))
        .clamp(
            MIN_GATEWAY_STUDIO_CONCURRENCY_LIMIT,
            MAX_GATEWAY_STUDIO_CONCURRENCY_LIMIT,
        )
}

fn default_gateway_studio_concurrency_limit(available_parallelism: Option<usize>) -> usize {
    available_parallelism
        .unwrap_or(DEFAULT_GATEWAY_STUDIO_CONCURRENCY_FALLBACK)
        .saturating_mul(4)
        .clamp(
            MIN_GATEWAY_STUDIO_CONCURRENCY_LIMIT,
            MAX_GATEWAY_STUDIO_CONCURRENCY_LIMIT,
        )
}

pub(crate) fn gateway_studio_request_timeout(
    runtime_config: Option<GatewayRuntimeTomlConfig>,
) -> Duration {
    Duration::from_secs(gateway_studio_request_timeout_secs_with_lookup(
        runtime_config,
        &|key| std::env::var(key).ok(),
    ))
}

pub(crate) fn gateway_flight_concurrency_limit(studio_concurrency_limit: usize) -> usize {
    gateway_flight_concurrency_limit_with_lookup(studio_concurrency_limit, &|key| {
        std::env::var(key).ok()
    })
}

pub(crate) fn gateway_flight_concurrency_limit_with_lookup(
    studio_concurrency_limit: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> usize {
    lookup_positive_parsed::<usize>(GATEWAY_FLIGHT_CONCURRENCY_LIMIT_ENV, lookup)
        .unwrap_or(studio_concurrency_limit)
        .clamp(
            MIN_GATEWAY_FLIGHT_CONCURRENCY_LIMIT,
            MAX_GATEWAY_FLIGHT_CONCURRENCY_LIMIT,
        )
}

pub(crate) fn gateway_flight_request_timeout(studio_request_timeout: Duration) -> Duration {
    Duration::from_secs(gateway_flight_request_timeout_secs_with_lookup(
        studio_request_timeout.as_secs(),
        &|key| std::env::var(key).ok(),
    ))
}

pub(crate) fn gateway_flight_request_timeout_secs_with_lookup(
    studio_request_timeout_secs: u64,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> u64 {
    lookup_positive_parsed::<u64>(GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS_ENV, lookup)
        .unwrap_or(studio_request_timeout_secs)
        .clamp(
            MIN_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS,
            MAX_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS,
        )
}

pub(crate) fn gateway_flight_grpc_web_enabled() -> bool {
    gateway_flight_grpc_web_enabled_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn gateway_flight_grpc_web_enabled_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    lookup_bool_flag(GATEWAY_FLIGHT_GRPC_WEB_ENABLED_ENV, lookup)
        .unwrap_or(DEFAULT_GATEWAY_FLIGHT_GRPC_WEB_ENABLED)
}

pub(crate) fn gateway_bearer_token() -> Option<Arc<str>> {
    gateway_bearer_token_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn gateway_bearer_token_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<Arc<str>> {
    lookup(GATEWAY_BEARER_TOKEN_ENV)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(Arc::<str>::from)
}

pub(crate) fn gateway_studio_request_timeout_secs_with_lookup(
    runtime_config: Option<GatewayRuntimeTomlConfig>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> u64 {
    runtime_config
        .and_then(|config| config.studio_request_timeout_secs)
        .or_else(|| lookup_positive_parsed::<u64>(GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS_ENV, lookup))
        .unwrap_or(DEFAULT_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS)
        .clamp(
            MIN_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS,
            MAX_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS,
        )
}

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/execute/gateway/command.rs"]
mod tests;
