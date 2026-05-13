//! Sentinel command implementation - starts the Project Sentinel file observer.
//!
//! This command starts the Sentinel daemon that watches specified paths
//! for file changes, emits semantic drift signals through the `ObservationBus`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use log::info;
use tokio::sync::mpsc;

use crate::bin_support::wendao::types::{Cli, SentinelArgs, SentinelCommand, SentinelWatchArgs};
use xiuxian_wendao::LinkGraphIndex;
use xiuxian_wendao::zhenfa_router::native::sentinel::{Sentinel, SentinelConfig};
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaSessionId, ZhenfaSignal};

/// Handle the sentinel command.
pub(crate) async fn handle(
    _cli: &Cli,
    args: &SentinelArgs,
    index: Option<&LinkGraphIndex>,
) -> Result<()> {
    match &args.command {
        SentinelCommand::Watch(watch_args) => handle_watch(watch_args, index).await,
    }
}

/// Handle the `sentinel watch` subcommand.
async fn handle_watch(args: &SentinelWatchArgs, index: Option<&LinkGraphIndex>) -> Result<()> {
    // 1. First, try CLI-provided paths
    let mut watch_paths: Vec<PathBuf> = args
        .paths
        .iter()
        .flat_map(|p: &String| p.split(','))
        .map(PathBuf::from)
        .collect();

    // 2. If CLI didn't specify paths, try configuration (wendao.toml via LinkGraphIndex)
    if watch_paths.is_empty()
        && let Some(idx) = index
    {
        let config_paths: Vec<PathBuf> = idx.include_dirs().iter().map(PathBuf::from).collect();
        if !config_paths.is_empty() {
            watch_paths = config_paths;
            info!("Sentinel: Using include_dirs from wendao.toml: {watch_paths:?}");
        }
    }

    // 3. Final fallback to defaults (only if no config available)
    if watch_paths.is_empty() {
        watch_paths = vec![PathBuf::from("docs"), PathBuf::from("src")];
        info!("Sentinel: No configuration found, using default paths: {watch_paths:?}");
    }

    info!("Starting Project Sentinel watching paths: {watch_paths:?}");

    // Create signal channel for semantic drift events
    let (signal_tx, mut signal_rx) = mpsc::unbounded_channel::<ZhenfaSignal>();

    // Build ZhenfaContext with optional LinkGraphIndex
    let mut ctx = ZhenfaContext::new(
        Some(ZhenfaSessionId::from("sentinel")),
        None,
        HashMap::default(),
    );

    // Attach signal sender so Sentinel can emit signals
    ctx.attach_signal_sender(signal_tx);

    // Inject LinkGraphIndex if available (for semantic drift detection)
    if let Some(index) = index {
        ctx.insert_shared_extension(Arc::new(index.clone()));
        info!("Sentinel: LinkGraphIndex injected for semantic drift detection");
    }

    // Configure Sentinel with debounce duration from args
    let config = SentinelConfig {
        watch_paths,
        debounce_duration: Duration::from_millis(args.debounce_ms),
    };

    // Start the real Sentinel engine
    let _sentinel = Sentinel::start(Arc::new(ctx), config)?;

    info!("Sentinel is active and scanning. Press Ctrl+C to stop.");

    // Spawn signal handler task
    tokio::spawn(async move {
        while let Some(signal) = signal_rx.recv().await {
            match &signal {
                ZhenfaSignal::SemanticDrift {
                    source_path,
                    file_stem,
                    affected_count,
                    confidence,
                    summary,
                } => {
                    info!(
                        "📡 SemanticDrift: {affected_count} docs affected by change in '{file_stem}' ({confidence})"
                    );
                    info!("   Source: {source_path}");
                    info!("   Summary: {summary}");
                }
                other => {
                    info!("📡 Signal received: {other:?}");
                }
            }
        }
    });

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Sentinel shutting down...");

    Ok(())
}

/// Parse comma-separated paths into a vector of `PathBuf`.
/// This is exposed for testing.
#[cfg(test)]
fn parse_paths(paths: &[String]) -> Vec<PathBuf> {
    paths
        .iter()
        .flat_map(|p: &String| p.split(','))
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
#[path = "../../../../tests/unit/bin/wendao/execute/sentinel.rs"]
mod tests;
