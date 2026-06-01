use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Request, StatusCode};
use axum::routing::{Router, post};
use tower::ServiceExt;

#[cfg(feature = "zhenfa-router")]
use crate::bin_support::wendao::execute::gateway::command::GATEWAY_FLIGHT_SERVICE_AXUM_PATH;
use crate::bin_support::wendao::execute::gateway::command::build_gateway_router as build_gateway_router_with_policy;
use crate::bin_support::wendao::execute::gateway::security::{
    GatewayPublicProtocolSurface, GatewaySurfaceSecurity, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER, WENDAO_PUBLIC_PROTOCOL_HEADER,
    WENDAO_SIGNED_PRINCIPAL_HEADER, with_gateway_surface_security,
};

use super::support::app_state;

mod auth;
mod lifecycle;
mod responses;

fn build_gateway_router(
    app_state: Arc<crate::bin_support::wendao::execute::gateway::state::AppState>,
    studio_concurrency_limit: usize,
    studio_request_timeout: std::time::Duration,
    flight_concurrency_limit: usize,
    flight_request_timeout: std::time::Duration,
    flight_grpc_web_enabled: bool,
    bearer_token: Option<Arc<str>>,
) -> anyhow::Result<Router> {
    build_gateway_router_with_surface_policy(
        app_state,
        studio_concurrency_limit,
        studio_request_timeout,
        flight_concurrency_limit,
        flight_request_timeout,
        512,
        128,
        64 * 1024 * 1024,
        1024 * 1024 * 1024,
        flight_grpc_web_enabled,
        bearer_token,
    )
}

fn build_gateway_router_with_surface_policy(
    app_state: Arc<crate::bin_support::wendao::execute::gateway::state::AppState>,
    studio_concurrency_limit: usize,
    studio_request_timeout: std::time::Duration,
    flight_concurrency_limit: usize,
    flight_request_timeout: std::time::Duration,
    https_rate_limit_per_second: u64,
    flight_rate_limit_per_second: u64,
    https_stream_budget_bytes: usize,
    flight_stream_budget_bytes: usize,
    flight_grpc_web_enabled: bool,
    bearer_token: Option<Arc<str>>,
) -> anyhow::Result<Router> {
    build_gateway_router_with_policy(
        app_state,
        studio_concurrency_limit,
        studio_request_timeout,
        flight_concurrency_limit,
        flight_request_timeout,
        https_rate_limit_per_second,
        flight_rate_limit_per_second,
        https_stream_budget_bytes,
        flight_stream_budget_bytes,
        flight_grpc_web_enabled,
        bearer_token,
    )
}
