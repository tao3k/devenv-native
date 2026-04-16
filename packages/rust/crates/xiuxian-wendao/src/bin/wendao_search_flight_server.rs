//! Runtime-backed Wendao Flight server binary that reads repo-search data from
//! the active search-plane store.

#[cfg(not(feature = "zhenfa-router"))]
fn main() {
    eprintln!("wendao_search_flight_server requires the `zhenfa-router` feature");
    std::process::exit(1);
}

#[cfg(feature = "zhenfa-router")]
use std::env;
#[cfg(feature = "zhenfa-router")]
use std::net::SocketAddr;
#[cfg(feature = "zhenfa-router")]
use std::path::PathBuf;
#[cfg(feature = "zhenfa-router")]
use std::sync::Arc;

#[cfg(feature = "zhenfa-router")]
use anyhow::{Result, anyhow};
#[cfg(feature = "zhenfa-router")]
use arrow_flight::flight_service_server::FlightServiceServer;
#[cfg(feature = "zhenfa-router")]
use tokio::net::TcpListener;
#[cfg(feature = "zhenfa-router")]
use tokio_stream::wrappers::TcpListenerStream;
#[cfg(feature = "zhenfa-router")]
use tonic::transport::Server;
#[cfg(feature = "zhenfa-router")]
use tonic_web::GrpcWebLayer;
#[cfg(feature = "zhenfa-router")]
use xiuxian_wendao::gateway::studio::{
    bootstrap_sample_repo_search_content, build_studio_flight_service_for_roots_with_weights,
    resolve_studio_config_root,
};
#[cfg(feature = "zhenfa-router")]
use xiuxian_wendao::link_graph::resolve_link_graph_rerank_flight_runtime_settings;
#[cfg(feature = "zhenfa-router")]
use xiuxian_wendao::search::SearchPlaneService;
#[cfg(feature = "zhenfa-router")]
use xiuxian_wendao::set_link_graph_wendao_config_override;
#[cfg(feature = "zhenfa-router")]
use xiuxian_wendao_runtime::transport::{
    EffectiveRerankFlightHostSettings, rerank_score_weights_from_env,
    resolve_effective_rerank_flight_host_settings as resolve_runtime_effective_rerank_flight_host_settings,
    split_rerank_flight_host_overrides,
};

#[cfg(feature = "zhenfa-router")]
#[tokio::main]
async fn main() -> Result<()> {
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
    println!("READY http://{local_addr}");

    Server::builder()
        .accept_http1(true)
        .layer(GrpcWebLayer::new())
        .add_service(FlightServiceServer::new(flight_service))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await
        .map_err(|error| anyhow!("Wendao search Flight server failed: {error}"))?;

    Ok(())
}

#[cfg(feature = "zhenfa-router")]
fn resolve_runtime_config_path(project_root: &std::path::Path) -> Option<PathBuf> {
    let local_config = std::path::Path::new("wendao.toml");
    if local_config.exists() {
        return std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(local_config));
    }

    let project_config = project_root.join("wendao.toml");
    project_config.exists().then_some(project_config)
}

#[cfg(feature = "zhenfa-router")]
fn resolve_effective_search_host_settings(
    schema_version_override: Option<String>,
    rerank_dimension_override: Option<usize>,
    fallback_rerank_dimension: usize,
) -> Result<EffectiveRerankFlightHostSettings> {
    let file_backed_settings = resolve_link_graph_rerank_flight_runtime_settings();
    Ok(resolve_runtime_effective_rerank_flight_host_settings(
        schema_version_override,
        rerank_dimension_override,
        file_backed_settings.schema_version,
        file_backed_settings.score_weights,
        fallback_rerank_dimension,
        rerank_score_weights_from_env().map_err(anyhow::Error::msg)?,
    ))
}

#[cfg(feature = "zhenfa-router")]
fn resolve_search_host_studio_config_root(project_root: &std::path::Path) -> PathBuf {
    resolve_runtime_config_path(project_root)
        .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
        .unwrap_or_else(|| resolve_studio_config_root(project_root))
}
