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
use walkdir::WalkDir;
use xiuxian_wendao::analyzers::load_repo_intelligence_config;
use xiuxian_wendao::link_graph::resolve_link_graph_rerank_flight_runtime_settings;
use xiuxian_wendao::repo_index::RepoCodeDocument;
use xiuxian_wendao::search::SearchPlaneService;
use xiuxian_wendao::set_link_graph_wendao_config_override;

const SEARCH_FLIGHT_GRPC_WEB_ENABLED_ENV: &str = "XIUXIAN_WENDAO_SEARCH_FLIGHT_GRPC_WEB_ENABLED";
const DEFAULT_SEARCH_FLIGHT_GRPC_WEB_ENABLED: bool = false;
const BOOTSTRAP_CONFIGURED_REPO_CONTENT_ENV: &str = "WENDAO_BOOTSTRAP_CONFIGURED_REPO_CONTENT";

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
    maybe_bootstrap_configured_repo_content(search_plane.as_ref(), repo_id.as_str(), &project_root)
        .await?;
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

async fn maybe_bootstrap_configured_repo_content(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    project_root: &Path,
) -> Result<()> {
    if env::var_os(BOOTSTRAP_CONFIGURED_REPO_CONTENT_ENV).is_none() {
        return Ok(());
    }

    let config_path = resolve_runtime_config_path(project_root)
        .ok_or_else(|| anyhow!("configured repo-content bootstrap requires wendao.toml"))?;
    let repo_config = load_repo_intelligence_config(Some(config_path.as_path()), project_root)
        .map_err(|error| anyhow!("load configured repo-content bootstrap config: {error}"))?;
    let repository = repo_config
        .repos
        .iter()
        .find(|repository| repository.id == repo_id)
        .ok_or_else(|| anyhow!("configured repo-content bootstrap repo `{repo_id}` not found"))?;
    let checkout_root = repository
        .path
        .as_deref()
        .ok_or_else(|| anyhow!("configured repo-content bootstrap repo `{repo_id}` has no path"))?;
    let documents = collect_configured_repo_content_documents(checkout_root)?;
    if documents.is_empty() {
        return Err(anyhow!(
            "configured repo-content bootstrap found no supported documents in `{}`",
            checkout_root.display()
        ));
    }

    search_plane
        .publish_repo_content_chunks_with_revision(
            repo_id,
            &documents,
            Some("configured-repo-content-bootstrap"),
        )
        .await
        .map_err(|error| anyhow!("publish configured repo-content bootstrap: {error}"))?;
    println!(
        "BOOTSTRAPPED_REPO_CONTENT {repo_id} documents={}",
        documents.len()
    );
    Ok(())
}

fn collect_configured_repo_content_documents(repo_root: &Path) -> Result<Vec<RepoCodeDocument>> {
    let mut documents = Vec::new();
    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|entry| !is_ignored_repo_content_path(entry.path()))
    {
        let entry = entry.map_err(|error| anyhow!("walk configured repo content: {error}"))?;
        let path = entry.path();
        if !path.is_file() || !is_supported_repo_content_path(path) {
            continue;
        }
        let relative_path = path
            .strip_prefix(repo_root)
            .map_err(|error| anyhow!("strip configured repo content path: {error}"))?
            .to_string_lossy()
            .replace('\\', "/");
        let contents = std::fs::read_to_string(path).map_err(|error| {
            anyhow!("read configured repo content `{}`: {error}", path.display())
        })?;
        let metadata = entry
            .metadata()
            .map_err(|error| anyhow!("read configured repo content metadata: {error}"))?;
        documents.push(RepoCodeDocument {
            path: relative_path,
            language: language_for_repo_content_path(path).map(str::to_owned),
            contents: Arc::<str>::from(contents),
            size_bytes: metadata.len(),
            modified_unix_ms: metadata
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |duration| {
                    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
                }),
        });
    }
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}

fn is_ignored_repo_content_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, ".git" | "target" | "node_modules"))
}

fn is_supported_repo_content_path(path: &Path) -> bool {
    language_for_repo_content_path(path).is_some()
}

fn language_for_repo_content_path(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("jl") => Some("julia"),
        Some("md") => Some("markdown"),
        Some("toml") => Some("toml"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/bin_support/flight_server.rs"]
mod tests;
