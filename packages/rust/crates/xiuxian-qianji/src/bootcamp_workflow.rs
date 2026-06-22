//! Bootcamp workflow surface for `xiuxian-qianji`.

pub(super) use super::manifest::{parse_manifest, parsed_manifest_requires_link_graph};
use super::manifest::{parsed_manifest_requires_llm, resolve_flow_manifest_toml};
use super::runtime::{
    build_link_graph_index, build_placeholder_link_graph_index, unix_timestamp_millis,
};
use super::{BootcampRunOptions, BootcampVfsMount, WorkflowReport};
use crate::error::QianjiError;
use crate::scheduler_preflight::{RuntimeWendaoMount, with_runtime_wendao_mounts};
use crate::{QianjiApp, QianjiManifestPipelineRequest, QianjiPipelineDependencies};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

/// Runs one workflow manifest resolved from a canonical `wendao://` URI.
///
/// This is the high-level "laboratory" entrypoint:
/// 1. resolve manifest URI from embedded Wendao resources,
/// 2. hydrate compiler dependencies,
/// 3. compile and execute through `QianjiScheduler`,
/// 4. return execution metadata plus final context.
///
/// # Errors
///
/// Returns [`QianjiError`] when URI resolution, manifest parsing, dependency
/// bootstrap, workflow compilation, or runtime execution fails.
pub async fn run_workflow(
    flow_uri: &str,
    initial_context: Value,
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    run_workflow_with_mounts(flow_uri, initial_context, &[], options).await
}

/// Runs one workflow manifest with optional extra embedded VFS mounts.
///
/// Mounts are used during initial flow TOML loading. When the same URI exists
/// in both extra mounts and Wendao built-in embedded registry, extra mounts
/// take precedence.
///
/// # Errors
///
/// Returns [`QianjiError`] when URI resolution, manifest parsing, dependency
/// bootstrap, workflow compilation, or runtime execution fails.
pub async fn run_workflow_with_mounts(
    flow_uri: &str,
    initial_context: Value,
    vfs_mounts: &[BootcampVfsMount],
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    let trimmed_flow_uri = flow_uri.trim();
    if trimmed_flow_uri.is_empty() {
        return Err(QianjiError::Topology(
            "bootcamp flow URI must be non-empty".to_string(),
        ));
    }

    let manifest_toml = resolve_flow_manifest_toml(trimmed_flow_uri, vfs_mounts)?;
    run_workflow_from_manifest_payload(
        trimmed_flow_uri,
        manifest_toml.as_str(),
        initial_context,
        vfs_mounts,
        options,
    )
    .await
}

/// Runs one workflow from raw manifest TOML without `wendao://` URI
/// resolution.
///
/// This helper is intended for bounded host-owned workflows that ship their
/// manifest as a built-in string constant and still want the standard bootcamp
/// runtime assembly.
///
/// # Errors
///
/// Returns [`QianjiError`] when manifest parsing, dependency bootstrap,
/// workflow compilation, or runtime execution fails.
pub async fn run_workflow_from_manifest_toml(
    manifest_toml: &str,
    initial_context: Value,
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    let trimmed_manifest_toml = manifest_toml.trim();
    if trimmed_manifest_toml.is_empty() {
        return Err(QianjiError::Topology(
            "bootcamp manifest TOML must be non-empty".to_string(),
        ));
    }

    run_workflow_from_manifest_payload(
        "inline://qianji/manifest",
        trimmed_manifest_toml,
        initial_context,
        &[],
        options,
    )
    .await
}

/// Compatibility alias of [`run_workflow`] for scenario-style callers.
///
/// This API accepts extra `include_dir` mounts so domain crates can provide
/// embedded resources directly without requiring hardcoded path wiring.
///
/// # Errors
///
/// Returns the same errors as [`run_workflow_with_mounts`].
pub async fn run_scenario(
    flow_uri: &str,
    initial_context: Value,
    vfs_mounts: &[BootcampVfsMount],
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    run_workflow_with_mounts(flow_uri, initial_context, vfs_mounts, options).await
}

async fn run_workflow_from_manifest_payload(
    flow_uri: &str,
    manifest_toml: &str,
    initial_context: Value,
    vfs_mounts: &[BootcampVfsMount],
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    let manifest = parse_manifest(manifest_toml)?;
    let requires_llm = parsed_manifest_requires_llm(&manifest);

    let BootcampRunOptions {
        repo_path,
        session_id,
        redis_url,
        index,
        consensus_manager,
    } = options;

    if requires_llm {
        return Err(QianjiError::Topology(
            "bootcamp workflow manifest contains llm nodes; local Qianji LLM execution is retired, use marlin-agent-core or an external service adapter"
                .to_string(),
        ));
    }

    let index = match index {
        Some(index) => index,
        None if parsed_manifest_requires_link_graph(&manifest) => {
            Arc::new(build_link_graph_index(repo_path.as_deref())?)
        }
        None => Arc::new(build_placeholder_link_graph_index()?),
    };
    let dependencies =
        QianjiPipelineDependencies::new(index).with_consensus_manager(consensus_manager);
    let scheduler = QianjiApp::create_pipeline_from_manifest(QianjiManifestPipelineRequest {
        manifest_toml,
        dependencies,
    })?;
    let runtime_mounts = vfs_mounts
        .iter()
        .copied()
        .map(RuntimeWendaoMount::from)
        .collect::<Vec<_>>();
    let started_at_unix_ms = unix_timestamp_millis()?;
    let started_at = Instant::now();
    let final_context = with_runtime_wendao_mounts(
        runtime_mounts,
        scheduler.run_with_checkpoint(initial_context, session_id, redis_url),
    )
    .await?;
    let finished_at_unix_ms = unix_timestamp_millis()?;
    let duration_ms = started_at.elapsed().as_millis();

    Ok(WorkflowReport {
        flow_uri: flow_uri.to_string(),
        manifest_name: manifest.name,
        node_count: manifest.nodes.len(),
        edge_count: manifest.edges.len(),
        requires_llm,
        started_at_unix_ms,
        finished_at_unix_ms,
        duration_ms,
        final_context,
    })
}

#[cfg(test)]
#[path = "../tests/unit/bootcamp/workflow.rs"]
mod tests;
