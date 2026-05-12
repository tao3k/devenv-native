//! `flight_host::server` owns Wendao flight host server behavior.

use std::io::{self, Write};
use std::sync::Arc;

use anyhow::Result;
use arrow_flight::flight_service_server::FlightServiceServer;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::link_graph::LinkGraphIndex;
use crate::search::SearchPlaneService;

use super::config::RepoSearchFlightHostConfig;
use super::providers::{
    SearchStrategyFlowFlightHostParts, build_search_strategy_flow_flight_service,
};
use super::repo_content::{configured_repo_root, maybe_bootstrap_configured_repo_content};

/// Result surface for the live repo-search Flight host application boundary.
pub type FlightHostResult<T> = Result<T>;

/// Run the live repo-search Flight server from binary argument values.
///
/// # Errors
///
/// Returns an error when argument parsing, repository bootstrap, link-graph
/// indexing, Flight service construction, socket binding, or server execution
/// fails.
pub async fn run_repo_search_flight_server_from_args(
    args: impl IntoIterator<Item = String>,
) -> FlightHostResult<()> {
    let config = RepoSearchFlightHostConfig::from_args(args)?;
    let search_plane = Arc::new(SearchPlaneService::new(config.project_root.clone()));
    let bootstrap_analysis = maybe_bootstrap_configured_repo_content(
        search_plane.as_ref(),
        config.repo_id.as_str(),
        config.project_root.as_path(),
        config.config_path.as_deref(),
    )
    .await?;
    let repo_root = configured_repo_root(
        config.repo_id.as_str(),
        config.project_root.as_path(),
        config.config_path.as_deref(),
    )?;
    let link_graph_index = Arc::new(
        LinkGraphIndex::build_with_local_cache(repo_root.as_path(), &[], &[])
            .map_err(anyhow::Error::msg)?,
    );
    let flight_service = build_search_strategy_flow_flight_service(
        SearchStrategyFlowFlightHostParts {
            repo_id: config.repo_id,
            project_root: config.project_root,
            config_path: config.config_path,
            search_plane,
            link_graph_index,
            bootstrap_analysis: bootstrap_analysis.map(Arc::new),
        },
        config.effective_settings.expected_schema_version,
        config.effective_settings.rerank_dimension,
        config.effective_settings.rerank_weights,
    )
    .map_err(anyhow::Error::msg)?;
    serve_search_flight(config.bind_addr, flight_service).await
}

async fn serve_search_flight(
    bind_addr: std::net::SocketAddr,
    flight_service: xiuxian_wendao_runtime::transport::WendaoFlightService,
) -> FlightHostResult<()> {
    let listener = TcpListener::bind(bind_addr).await?;
    let local_addr = listener.local_addr()?;
    writeln!(io::stdout(), "READY http://{local_addr}")?;
    io::stdout().flush()?;

    Server::builder()
        .add_service(FlightServiceServer::new(flight_service))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await?;
    Ok(())
}
