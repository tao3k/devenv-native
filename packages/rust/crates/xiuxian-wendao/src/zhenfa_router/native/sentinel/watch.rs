use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use log::{error, info, warn};
use notify::{Event, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use tokio::sync::mpsc;

use xiuxian_zhenfa::ZhenfaContext;
use xiuxian_zhenfa::ZhenfaSignal;

use super::super::forwarder::ForwardNotifier;
use super::analysis::propagate_source_change;
use super::filters::{
    is_high_noise_file, is_ignorable_path, is_source_code, is_supported_doc, verify_file_stable,
};
use super::observations::ObservationBus;
use crate::zhenfa_router::native::WendaoContextExt;

/// Configuration for the Sentinel observer.
#[derive(Debug, Clone)]
pub struct SentinelConfig {
    /// Paths to watch for changes.
    pub watch_paths: Vec<PathBuf>,
    /// Debounce duration (increased for CAS consistency).
    pub debounce_duration: Duration,
}

impl Default for SentinelConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![PathBuf::from("docs"), PathBuf::from("src")],
            // Increased to 1000ms for CAS consistency (audit recommendation)
            debounce_duration: Duration::from_millis(1000),
        }
    }
}

/// The Sentinel observer.
pub struct Sentinel {
    _ctx: Arc<ZhenfaContext>,
    _config: SentinelConfig,
    _debouncer: Debouncer<RecommendedWatcher, RecommendedCache>,
}

impl Sentinel {
    /// Create and start a new Sentinel observer.
    ///
    /// # Errors
    ///
    /// Returns an error when the filesystem debouncer cannot be created or when any configured
    /// watch path cannot be registered with the underlying watcher.
    pub fn start(ctx: Arc<ZhenfaContext>, config: SentinelConfig) -> Result<Self, anyhow::Error> {
        let (tx, mut rx) = mpsc::channel(100);

        // Create the debouncer
        // DebounceEventResult = Result<Vec<DebouncedEvent>, Vec<Error>>
        let mut debouncer = new_debouncer(
            config.debounce_duration,
            None,
            move |result: DebounceEventResult| {
                if let Ok(events) = result {
                    for event in events {
                        let _ = tx.try_send(event.event);
                    }
                }
            },
        )?;

        // Watch the paths - new API uses debouncer.watch() directly
        for path in &config.watch_paths {
            if path.exists() {
                info!("Sentinel watching: {}", path.display());
                debouncer.watch(path, RecursiveMode::Recursive)?;
            }
        }

        // Spawn the event handler
        let handler_ctx = ctx.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Err(e) = handle_sentinel_event(&handler_ctx, event).await {
                    error!("Sentinel event handler error: {e:?}");
                }
            }
        });

        Ok(Self {
            _ctx: ctx,
            _config: config,
            _debouncer: debouncer,
        })
    }
}

/// Internal event handler for Sentinel.
async fn handle_sentinel_event(ctx: &ZhenfaContext, event: Event) -> Result<(), anyhow::Error> {
    for path in event.paths {
        if is_ignorable_path(&path) {
            continue;
        }

        info!("Sentinel detected change in: {}", path.display());

        // PHASE 5: Instant LinkGraph Refresh for documentation files
        if !is_source_code(&path) && is_supported_doc(&path) {
            handle_doc_change(ctx, &path);
            continue;
        }

        // PHASE 6: Semantic Change Propagation for source code
        if is_source_code(&path) {
            // Skip high-noise files that would cause false positives
            if is_high_noise_file(&path) {
                info!("Skipping high-noise file: {}", path.display());
                continue;
            }

            // CAS Consistency: Verify file is stable before analysis
            if !verify_file_stable(&path) {
                info!("File not yet stable, skipping: {}", path.display());
                continue;
            }

            if let Err(e) = handle_source_change(ctx, &path).await {
                warn!(
                    "Phase 6 semantic propagation failed for {}: {e}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

/// Handle documentation file changes (Phase 5: Incremental Refresh).
fn handle_doc_change(ctx: &ZhenfaContext, path: &Path) {
    info!("Phase 5: Incremental refresh for doc: {}", path.display());

    // Get mutable access to the index through context
    // Note: This requires the index to be behind Arc<RwLock> or similar
    // For now, we emit a signal that can be consumed by the index manager

    let doc_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // Emit trace signal for incremental refresh request
    if let Some(sender) = ctx.get_extension::<mpsc::UnboundedSender<ZhenfaSignal>>() {
        let signal = ZhenfaSignal::Trace {
            node_id: format!("sentinel:doc:{doc_id}"),
            event: format!("incremental_refresh_requested:{}", path.display()),
        };
        if sender.send(signal).is_err() {
            warn!("Failed to emit incremental refresh signal");
        }
    }

    // TODO: When LinkGraphIndex is behind Arc<RwLock>:
    // 1. Parse the modified document
    // 2. Call index.refresh_symbol_cache_for_doc(doc_id)
    // 3. Update the page index tree

    info!("Phase 5: Incremental refresh scheduled for: {doc_id}");
}

/// Handle source code changes (Phase 6: Semantic Propagation).
async fn handle_source_change(ctx: &ZhenfaContext, path: &Path) -> Result<(), anyhow::Error> {
    if let Ok(index) = ctx.link_graph_index() {
        let drift_signals = propagate_source_change(&index, path);

        if drift_signals.is_empty() {
            return Ok(());
        }

        info!(
            "Phase 6.2: Generated {} semantic drift signal(s)",
            drift_signals.len()
        );

        // Convert to ZhenfaSignal and emit
        if let Some(sender) = ctx.get_extension::<mpsc::UnboundedSender<ZhenfaSignal>>() {
            for drift in &drift_signals {
                let signal = ZhenfaSignal::SemanticDrift {
                    source_path: drift.source_path.clone(),
                    file_stem: drift.file_stem.clone(),
                    affected_count: drift.affected_docs.len(),
                    confidence: drift.confidence.to_string(),
                    summary: drift.summary(),
                };

                match sender.send(signal) {
                    Ok(()) => info!("Emitted SemanticDrift signal for: {}", drift.source_path),
                    Err(e) => warn!("Failed to emit SemanticDrift signal: {e}"),
                }
            }
        } else {
            warn!("No signal sender attached to context - signals not emitted");
            // Still log the signals for debugging
            for signal in &drift_signals {
                info!("  Signal (not emitted): {}", signal.summary());
            }
        }

        // Also emit through ObservationBus if available
        if let Some(bus) = ctx.get_extension::<ObservationBus>() {
            for drift in &drift_signals {
                let signal_ids = bus.emit_drift_signals(drift);
                if !signal_ids.is_empty() {
                    info!("Emitted {} ObservationSignals via bus", signal_ids.len());
                }
            }
        }

        // Phase 7: Process drifts through ForwardNotifier for proactive notifications
        if let Some(forwarder) = ctx.get_extension::<Arc<ForwardNotifier>>() {
            for drift in &drift_signals {
                if forwarder.process_drift(drift).await {
                    info!(
                        "ForwardNotifier queued notification for: {}",
                        drift.source_path
                    );
                } else {
                    // Not queued - could be rate limited, debounced, or below threshold
                    log::debug!(
                        "ForwardNotifier skipped notification for: {} (rate limit/debounce/threshold)",
                        drift.source_path
                    );
                }
            }
        }
    }

    Ok(())
}
