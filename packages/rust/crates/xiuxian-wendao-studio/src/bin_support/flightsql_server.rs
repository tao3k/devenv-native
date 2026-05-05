//! FlightSQL server runtime used by the thin binary entrypoint.

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use arrow_flight::flight_service_server::FlightServiceServer;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use xiuxian_config_core::lookup_bool_flag;

use crate::studio::bootstrap_sample_repo_search_content;
use xiuxian_wendao::search::SearchPlaneService;
use xiuxian_wendao::search::queries::flightsql::build_studio_flightsql_service;

const SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED_ENV: &str =
    "XIUXIAN_WENDAO_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED";
const DEFAULT_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED: bool = false;

/// Starts the Wendao `FlightSQL` server from process arguments.
///
/// # Errors
///
/// Returns an error when command-line arguments are invalid, sample content
/// bootstrap fails, or the TCP/gRPC server cannot bind and serve.
pub async fn run_search_flightsql_server() -> Result<()> {
    let mut args = env::args().skip(1);
    let bind_addr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:0".to_string())
        .parse::<SocketAddr>()
        .map_err(|error| anyhow!("invalid bind address: {error}"))?;
    let repo_id = args.next().unwrap_or_else(|| "alpha/repo".to_string());
    let project_root = match args.next() {
        Some(path) => PathBuf::from(path),
        None => {
            env::current_dir().map_err(|error| anyhow!("failed to resolve current dir: {error}"))?
        }
    };

    let search_plane = SearchPlaneService::new(project_root);
    if env::var_os("WENDAO_BOOTSTRAP_SAMPLE_REPO").is_some() {
        bootstrap_sample_repo_search_content(&search_plane, repo_id.as_str())
            .await
            .map_err(|error| anyhow!(error))?;
    }
    let flightsql_service = build_studio_flightsql_service(search_plane);

    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|error| anyhow!("failed to bind Wendao FlightSQL server: {error}"))?;
    let local_addr = listener
        .local_addr()
        .map_err(|error| anyhow!("failed to read Wendao FlightSQL server address: {error}"))?;
    let grpc_web_enabled = search_flightsql_grpc_web_enabled();
    println!("READY http://{local_addr}");

    if grpc_web_enabled {
        Server::builder()
            .accept_http1(true)
            .layer(GrpcWebLayer::new())
            .add_service(FlightServiceServer::new(flightsql_service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .map_err(|error| anyhow!("Wendao FlightSQL server failed: {error}"))?;
    } else {
        Server::builder()
            .add_service(FlightServiceServer::new(flightsql_service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .map_err(|error| anyhow!("Wendao FlightSQL server failed: {error}"))?;
    }

    Ok(())
}

fn search_flightsql_grpc_web_enabled() -> bool {
    search_flightsql_grpc_web_enabled_with_lookup(&|key| std::env::var(key).ok())
}

fn search_flightsql_grpc_web_enabled_with_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> bool {
    lookup_bool_flag(SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED_ENV, lookup)
        .unwrap_or(DEFAULT_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED)
}

#[cfg(test)]
#[path = "../../tests/unit/bin_support/flightsql_server.rs"]
mod tests;
