use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Request, StatusCode};
use axum::routing::Router;
use tower::ServiceExt;

#[cfg(feature = "julia")]
use crate::bin_support::wendao::execute::gateway::command::GATEWAY_FLIGHT_SERVICE_AXUM_PATH;
use crate::bin_support::wendao::execute::gateway::command::build_gateway_router;

use super::support::app_state;

mod auth;
mod lifecycle;
mod responses;
