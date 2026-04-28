//! Shared-query `FlightSQL` server binary over the Wendao search-plane surface.

#[cfg(not(feature = "julia"))]
fn main() {
    eprintln!("wendao_search_flightsql_server requires the `julia` feature");
    std::process::exit(1);
}

#[cfg(feature = "julia")]
use std::env;
#[cfg(feature = "julia")]
use std::net::SocketAddr;
#[cfg(feature = "julia")]
use std::path::PathBuf;

#[cfg(feature = "julia")]
use anyhow::{Result, anyhow};
#[cfg(feature = "julia")]
use arrow_flight::flight_service_server::FlightServiceServer;
#[cfg(feature = "julia")]
use tokio::net::TcpListener;
#[cfg(feature = "julia")]
use tokio_stream::wrappers::TcpListenerStream;
#[cfg(feature = "julia")]
use tonic::transport::Server;
#[cfg(feature = "julia")]
use tonic_web::GrpcWebLayer;
#[cfg(feature = "julia")]
use xiuxian_config_core::lookup_bool_flag;
#[cfg(feature = "julia")]
use xiuxian_wendao::gateway::studio::bootstrap_sample_repo_search_content;
#[cfg(feature = "julia")]
use xiuxian_wendao::search::SearchPlaneService;
#[cfg(feature = "julia")]
use xiuxian_wendao::search::queries::flightsql::build_studio_flightsql_service;

#[cfg(feature = "julia")]
const SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED_ENV: &str =
    "XIUXIAN_WENDAO_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED";
#[cfg(feature = "julia")]
const DEFAULT_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED: bool = false;

#[cfg(feature = "julia")]
#[tokio::main]
async fn main() -> Result<()> {
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

#[cfg(feature = "julia")]
fn search_flightsql_grpc_web_enabled() -> bool {
    search_flightsql_grpc_web_enabled_with_lookup(&|key| std::env::var(key).ok())
}

#[cfg(feature = "julia")]
fn search_flightsql_grpc_web_enabled_with_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> bool {
    lookup_bool_flag(SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED_ENV, lookup)
        .unwrap_or(DEFAULT_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED)
}

#[cfg(all(test, feature = "julia"))]
mod tests {
    use super::search_flightsql_grpc_web_enabled_with_lookup;

    #[test]
    fn search_flightsql_grpc_web_defaults_to_disabled() {
        assert!(!search_flightsql_grpc_web_enabled_with_lookup(&|_| None));
    }

    #[test]
    fn search_flightsql_grpc_web_accepts_explicit_override() {
        assert!(search_flightsql_grpc_web_enabled_with_lookup(
            &|key| match key {
                "XIUXIAN_WENDAO_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED" => Some("yes".to_string()),
                _ => None,
            }
        ));
        assert!(!search_flightsql_grpc_web_enabled_with_lookup(
            &|key| match key {
                "XIUXIAN_WENDAO_SEARCH_FLIGHTSQL_GRPC_WEB_ENABLED" => Some("off".to_string()),
                _ => None,
            }
        ));
    }
}
