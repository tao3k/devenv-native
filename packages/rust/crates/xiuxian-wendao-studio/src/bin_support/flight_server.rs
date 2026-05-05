//! Arrow Flight server runtime used by the thin binary entrypoint.

use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::transport::{
    EffectiveRerankFlightHostSettings, RerankScoreWeights, rerank_score_weights_from_env,
    resolve_effective_rerank_flight_host_settings as resolve_runtime_effective_rerank_flight_host_settings,
    split_rerank_flight_host_overrides,
};
use anyhow::{Result, anyhow};
use arrow_flight::flight_service_server::FlightServiceServer;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use xiuxian_config_core::lookup_bool_flag;

use crate::studio::{
    bootstrap_sample_repo_search_content, build_studio_flight_service_for_roots_with_weights,
    resolve_studio_config_root,
};
use xiuxian_wendao::link_graph::resolve_link_graph_rerank_flight_runtime_settings;
use xiuxian_wendao::search::SearchPlaneService;
use xiuxian_wendao::set_link_graph_wendao_config_override;

const SEARCH_FLIGHT_GRPC_WEB_ENABLED_ENV: &str = "XIUXIAN_WENDAO_SEARCH_FLIGHT_GRPC_WEB_ENABLED";
const DEFAULT_SEARCH_FLIGHT_GRPC_WEB_ENABLED: bool = false;

/// Starts the Wendao repo-search Arrow Flight server from process arguments.
///
/// # Errors
///
/// Returns an error when command-line arguments are invalid, runtime settings
/// cannot be resolved, the backing search service cannot bootstrap sample
/// content, or the TCP/gRPC server cannot bind and serve.
pub async fn run_search_flight_server() -> Result<()> {
    let mut args = env::args().skip(1);
    let bind_addr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:0".to_string())
        .parse::<SocketAddr>()
        .map_err(|error| anyhow!("invalid bind address: {error}"))?;
    let parsed_overrides = split_rerank_flight_host_overrides(args).map_err(anyhow::Error::msg)?;
    let mut positional_args = parsed_overrides.positional_args.into_iter();
    let repo_id = positional_args
        .next()
        .unwrap_or_else(|| "alpha/repo".to_string());
    let project_root = match positional_args.next() {
        Some(path) => PathBuf::from(path),
        None => {
            env::current_dir().map_err(|error| anyhow!("failed to resolve current dir: {error}"))?
        }
    };
    let positional_rerank_dimension = positional_args
        .next()
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|error| anyhow!("invalid rerank dimension: {error}"))
        })
        .transpose()?
        .unwrap_or(3);

    if let Some(config_path) = resolve_runtime_config_path(project_root.as_path())
        && let Some(path_str) = config_path.to_str()
    {
        set_link_graph_wendao_config_override(path_str);
    }
    let effective_settings = resolve_effective_search_host_settings(
        parsed_overrides.schema_version_override,
        parsed_overrides.rerank_dimension_override,
        positional_rerank_dimension,
    )?;

    let search_plane = Arc::new(SearchPlaneService::new(project_root.clone()));
    if env::var_os("WENDAO_BOOTSTRAP_SAMPLE_REPO").is_some() {
        bootstrap_sample_repo_search_content(search_plane.as_ref(), repo_id.as_str())
            .await
            .map_err(|error| anyhow!(error))?;
    }
    let flight_service = build_studio_flight_service_for_roots_with_weights(
        search_plane,
        project_root.clone(),
        resolve_search_host_studio_config_root(project_root.as_path()),
        effective_settings.expected_schema_version,
        effective_settings.rerank_dimension,
        effective_settings.rerank_weights,
    )
    .map_err(|error| anyhow!(error))?;

    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|error| anyhow!("failed to bind Wendao search Flight server: {error}"))?;
    let local_addr = listener
        .local_addr()
        .map_err(|error| anyhow!("failed to read Wendao search Flight server address: {error}"))?;
    let grpc_web_enabled = search_flight_grpc_web_enabled();
    println!("READY http://{local_addr}");

    if grpc_web_enabled {
        Server::builder()
            .accept_http1(true)
            .layer(GrpcWebLayer::new())
            .add_service(FlightServiceServer::new(flight_service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .map_err(|error| anyhow!("Wendao search Flight server failed: {error}"))?;
    } else {
        Server::builder()
            .add_service(FlightServiceServer::new(flight_service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .map_err(|error| anyhow!("Wendao search Flight server failed: {error}"))?;
    }

    Ok(())
}

fn resolve_runtime_config_path(project_root: &Path) -> Option<PathBuf> {
    let local_config = Path::new("wendao.toml");
    if local_config.exists() {
        return std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(local_config));
    }

    let project_config = project_root.join("wendao.toml");
    project_config.exists().then_some(project_config)
}

fn resolve_effective_search_host_settings(
    schema_version_override: Option<String>,
    rerank_dimension_override: Option<usize>,
    fallback_rerank_dimension: usize,
) -> Result<EffectiveRerankFlightHostSettings> {
    let file_backed_settings = resolve_link_graph_rerank_flight_runtime_settings();
    let file_backed_weights = file_backed_settings
        .score_weights
        .map(|weights| RerankScoreWeights::new(weights.vector_weight, weights.semantic_weight))
        .transpose()
        .map_err(anyhow::Error::msg)?;
    Ok(resolve_runtime_effective_rerank_flight_host_settings(
        schema_version_override,
        rerank_dimension_override,
        file_backed_settings.schema_version,
        file_backed_weights,
        fallback_rerank_dimension,
        rerank_score_weights_from_env().map_err(anyhow::Error::msg)?,
    ))
}

fn resolve_search_host_studio_config_root(project_root: &Path) -> PathBuf {
    resolve_runtime_config_path(project_root)
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| resolve_studio_config_root(project_root))
}

fn search_flight_grpc_web_enabled() -> bool {
    search_flight_grpc_web_enabled_with_lookup(&|key| std::env::var(key).ok())
}

fn search_flight_grpc_web_enabled_with_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> bool {
    lookup_bool_flag(SEARCH_FLIGHT_GRPC_WEB_ENABLED_ENV, lookup)
        .unwrap_or(DEFAULT_SEARCH_FLIGHT_GRPC_WEB_ENABLED)
}

#[cfg(test)]
#[path = "../../tests/unit/bin_support/flight_server.rs"]
mod tests;
