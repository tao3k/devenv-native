use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Request, StatusCode};
use axum::routing::Router;
use tower::ServiceExt;

use crate::execute::gateway::command::{GATEWAY_FLIGHT_SERVICE_AXUM_PATH, build_gateway_router};

use super::support::app_state;

mod auth;
mod lifecycle;
mod responses;
