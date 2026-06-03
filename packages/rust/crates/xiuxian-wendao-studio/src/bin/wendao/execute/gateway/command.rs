//! Gateway command execution.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
#[cfg(feature = "zhenfa-router")]
use arrow_flight::flight_service_server::FlightServiceServer;
use axum::Json;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
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
use tower_http::limit::RequestBodyLimitLayer;

#[cfg(feature = "postgres-auth")]
use crate::bin_support::wendao::execute::gateway::postgres_auth::gateway_postgres_api_token_repository_with_lookup;
use crate::bin_support::wendao::execute::gateway::{
    auth::{gateway_auth_issuer, gateway_auth_router},
    config::{
        get_gateway_runtime_from_config, resolve_bind_addr, resolve_config_path, resolve_port,
        resolve_webhook_config,
    },
    health::health,
    policy::{
        gateway_flight_concurrency_limit, gateway_flight_grpc_web_enabled,
        gateway_flight_rate_limit_per_second, gateway_flight_request_timeout,
        gateway_flight_stream_budget_bytes, gateway_https_rate_limit_per_second,
        gateway_https_stream_budget_bytes, gateway_listen_backlog,
        gateway_studio_concurrency_limit, gateway_studio_request_timeout,
    },
    query::{GATEWAY_QUERY_AXUM_PATH, GATEWAY_RESPONSES_AXUM_PATH, query, responses},
    registry::build_plugin_registry,
    security::{
        GATEWAY_BEARER_TOKEN_ENV, GatewayApiTokenAdmission, GatewayPublicProtocolSurface,
        GatewaySurfacePolicy, GatewaySurfaceSecurity, gateway_api_token_admission,
        gateway_bearer_token, gateway_internal_principal_secret, with_gateway_surface_security,
    },
    state::AppState,
    status::{notify_status, stats},
};

#[cfg(test)]
pub(crate) use crate::bin_support::wendao::execute::gateway::config::GatewayRuntimeTomlConfig;
#[cfg(test)]
pub(crate) use crate::bin_support::wendao::execute::gateway::policy::{
    gateway_flight_concurrency_limit_with_lookup, gateway_flight_grpc_web_enabled_with_lookup,
    gateway_flight_rate_limit_per_second_with_lookup,
    gateway_flight_request_timeout_secs_with_lookup,
    gateway_flight_stream_budget_bytes_with_lookup,
    gateway_https_rate_limit_per_second_with_lookup, gateway_https_stream_budget_bytes_with_lookup,
    gateway_listen_backlog_with_lookup, gateway_studio_concurrency_limit_with_lookup,
    gateway_studio_request_timeout_secs_with_lookup,
};
#[cfg(test)]
pub(crate) use crate::bin_support::wendao::execute::gateway::security::{
    gateway_bearer_token_with_lookup, gateway_internal_principal_secret_with_lookup,
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
use xiuxian_wendao::LinkGraphIndex;
#[cfg(feature = "zhenfa-router")]
use xiuxian_wendao::link_graph::resolve_link_graph_rerank_flight_runtime_settings;
#[cfg(feature = "zhenfa-router")]
use xiuxian_wendao_server::transport::WendaoFlightInternalSecurity;
use xiuxian_zhenfa::{NotificationService, ZhenfaSignal, notification_worker};

#[cfg(feature = "zhenfa-router")]
const WENDAO_FLIGHT_INTERNAL_PRINCIPAL_REQUIRED_CODE: &str =
    "WENDAO_FLIGHT_INTERNAL_PRINCIPAL_REQUIRED";
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
    let https_rate_limit_per_second = gateway_https_rate_limit_per_second(studio_concurrency_limit);
    let flight_rate_limit_per_second =
        gateway_flight_rate_limit_per_second(flight_concurrency_limit);
    let https_stream_budget_bytes = gateway_https_stream_budget_bytes();
    let flight_stream_budget_bytes = gateway_flight_stream_budget_bytes();
    let flight_grpc_web_enabled = gateway_flight_grpc_web_enabled();
    let bearer_token = require_gateway_bearer_token()?;
    let api_token_admission = gateway_api_token_admission_for_startup().await?;

    // 3. Build the Axum router
    let app = build_gateway_router(
        app_state.clone(),
        studio_concurrency_limit,
        studio_request_timeout,
        flight_concurrency_limit,
        flight_request_timeout,
        https_rate_limit_per_second,
        flight_rate_limit_per_second,
        https_stream_budget_bytes,
        flight_stream_budget_bytes,
        flight_grpc_web_enabled,
        Some(bearer_token),
        api_token_admission,
    )?;

    // 4. Start the server
    let bind_addr = resolve_bind_addr(config_path.as_deref());
    let addr = SocketAddr::from((bind_addr, port));
    info!("Starting Wendao Gateway on {addr}");
    info!(
        "Gateway listener backlog={listen_backlog}, HTTPS concurrency limit={studio_concurrency_limit}, HTTPS rate limit={https_rate_limit_per_second}/s, HTTPS request timeout={}s, HTTPS stream budget={} bytes",
        studio_request_timeout.as_secs(),
        https_stream_budget_bytes
    );
    #[cfg(feature = "zhenfa-router")]
    info!(
        "Gateway Flight concurrency limit={flight_concurrency_limit}, Flight rate limit={flight_rate_limit_per_second}/s, Flight request timeout={}s, Flight stream budget={} bytes, gRPC-Web enabled={flight_grpc_web_enabled}",
        flight_request_timeout.as_secs(),
        flight_stream_budget_bytes,
    );
    info!("Gateway bearer auth required=true");
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
    info!(
        "  - POST {}  - Public API token login/issuance",
        openapi_paths::API_AUTH_TOKENS_AXUM_PATH
    );
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

fn require_gateway_bearer_token() -> Result<Arc<str>> {
    gateway_bearer_token().ok_or_else(|| {
        anyhow!(
            "Wendao Gateway is the only public boundary and requires `{GATEWAY_BEARER_TOKEN_ENV}`"
        )
    })
}

#[cfg(test)]
pub(crate) fn require_gateway_bearer_token_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Arc<str>> {
    gateway_bearer_token_with_lookup(lookup).ok_or_else(|| {
        anyhow!(
            "Wendao Gateway is the only public boundary and requires `{GATEWAY_BEARER_TOKEN_ENV}`"
        )
    })
}

pub(crate) fn build_gateway_router(
    app_state: Arc<AppState>,
    studio_concurrency_limit: usize,
    studio_request_timeout: Duration,
    flight_concurrency_limit: usize,
    flight_request_timeout: Duration,
    https_rate_limit_per_second: u64,
    flight_rate_limit_per_second: u64,
    https_stream_budget_bytes: usize,
    flight_stream_budget_bytes: usize,
    flight_grpc_web_enabled: bool,
    bearer_token: Option<Arc<str>>,
    api_token_admission: Option<GatewayApiTokenAdmission>,
) -> Result<Router> {
    let auth_issuer = gateway_auth_issuer(api_token_admission.clone(), https_rate_limit_per_second);
    let protected_app = Router::new()
        .route(openapi_paths::API_STATS_AXUM_PATH, get(stats))
        .route(openapi_paths::API_NOTIFY_AXUM_PATH, get(notify_status))
        .route(GATEWAY_QUERY_AXUM_PATH, post(query))
        .route(GATEWAY_RESPONSES_AXUM_PATH, post(responses))
        .merge(studio_routes())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_gateway_service_error))
                .load_shed()
                .timeout(studio_request_timeout)
                .concurrency_limit(studio_concurrency_limit)
                .layer(RequestBodyLimitLayer::new(https_stream_budget_bytes)),
        );
    let protected_app = with_gateway_surface_security(
        protected_app,
        gateway_surface_security(
            GatewayPublicProtocolSurface::HttpsJsonSse,
            bearer_token.clone(),
            api_token_admission.clone(),
            GatewaySurfacePolicy::new(https_rate_limit_per_second, https_stream_budget_bytes),
        ),
    );
    let app = Router::new()
        .route(openapi_paths::API_HEALTH_AXUM_PATH, get(health))
        .merge(protected_app);
    let app = if let Some(auth_issuer) = auth_issuer {
        app.merge(gateway_auth_router(auth_issuer))
    } else {
        app
    };
    let app = app.with_state(app_state.clone());

    #[cfg(feature = "zhenfa-router")]
    let app = mount_gateway_flight_service(
        app,
        app_state,
        flight_concurrency_limit,
        flight_request_timeout,
        flight_rate_limit_per_second,
        flight_stream_budget_bytes,
        flight_grpc_web_enabled,
        bearer_token,
        api_token_admission,
    )?;
    #[cfg(not(feature = "zhenfa-router"))]
    let _ = (
        flight_concurrency_limit,
        flight_request_timeout,
        https_rate_limit_per_second,
        flight_rate_limit_per_second,
        https_stream_budget_bytes,
        flight_stream_budget_bytes,
        flight_grpc_web_enabled,
        api_token_admission,
    );

    Ok(app)
}

fn gateway_surface_security(
    surface: GatewayPublicProtocolSurface,
    bearer_token: Option<Arc<str>>,
    api_token_admission: Option<GatewayApiTokenAdmission>,
    policy: GatewaySurfacePolicy,
) -> GatewaySurfaceSecurity {
    let security = GatewaySurfaceSecurity::new(surface, bearer_token).with_policy(policy);
    if let Some((verifier, lookup)) = api_token_admission {
        return security.with_api_token_admission(verifier, lookup);
    }
    security
}

async fn gateway_api_token_admission_for_startup() -> Result<Option<GatewayApiTokenAdmission>> {
    let Some((verifier, local_repository)) = gateway_api_token_admission() else {
        return Ok(None);
    };
    #[cfg(feature = "postgres-auth")]
    {
        if let Some(postgres_repository) =
            gateway_postgres_api_token_repository_with_lookup(&|key| std::env::var(key).ok())
                .await
                .map_err(anyhow::Error::msg)?
        {
            return Ok(Some((verifier, postgres_repository)));
        }
    }
    Ok(Some((verifier, local_repository)))
}

#[cfg(feature = "zhenfa-router")]
fn mount_gateway_flight_service(
    app: Router,
    app_state: Arc<AppState>,
    flight_concurrency_limit: usize,
    flight_request_timeout: Duration,
    flight_rate_limit_per_second: u64,
    flight_stream_budget_bytes: usize,
    flight_grpc_web_enabled: bool,
    bearer_token: Option<Arc<str>>,
    api_token_admission: Option<GatewayApiTokenAdmission>,
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
    let flight_service = match gateway_internal_principal_secret().or_else(|| bearer_token.clone())
    {
        Some(secret) => {
            flight_service.with_internal_security(WendaoFlightInternalSecurity::gateway(
                secret,
                Arc::<str>::from(WENDAO_FLIGHT_INTERNAL_PRINCIPAL_REQUIRED_CODE),
            ))
        }
        None => flight_service,
    };
    let flight_service = FlightServiceServer::new(flight_service);
    if flight_grpc_web_enabled {
        let flight_service = GrpcWebLayer::new().layer(flight_service);
        let flight_service = ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_gateway_service_error))
            .load_shed()
            .timeout(flight_request_timeout)
            .concurrency_limit(flight_concurrency_limit)
            .layer(RequestBodyLimitLayer::new(flight_stream_budget_bytes))
            .service(flight_service);
        let flight_router = Router::new().route(
            GATEWAY_FLIGHT_SERVICE_AXUM_PATH,
            any_service(flight_service),
        );
        return Ok(app.merge(with_gateway_surface_security(
            flight_router,
            gateway_surface_security(
                GatewayPublicProtocolSurface::ArrowFlight,
                bearer_token,
                api_token_admission,
                GatewaySurfacePolicy::new(flight_rate_limit_per_second, flight_stream_budget_bytes),
            ),
        )));
    }

    let flight_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_gateway_service_error))
        .load_shed()
        .timeout(flight_request_timeout)
        .concurrency_limit(flight_concurrency_limit)
        .layer(RequestBodyLimitLayer::new(flight_stream_budget_bytes))
        .service(flight_service);
    let flight_router = Router::new().route(
        GATEWAY_FLIGHT_SERVICE_AXUM_PATH,
        any_service(flight_service),
    );
    Ok(app.merge(with_gateway_surface_security(
        flight_router,
        gateway_surface_security(
            GatewayPublicProtocolSurface::ArrowFlight,
            bearer_token,
            api_token_admission,
            GatewaySurfacePolicy::new(flight_rate_limit_per_second, flight_stream_budget_bytes),
        ),
    )))
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

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/execute/gateway/command/mod.rs"]
mod tests;
