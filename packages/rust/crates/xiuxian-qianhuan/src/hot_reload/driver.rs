//! File-watcher driver that triggers Qianhuan hot-reload runtime updates.

use super::HotReloadRuntime;
use anyhow::{Result, anyhow};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use xiuxian_io::{FileEvent, FileWatcherHandle, WatcherConfig, WatcherResult, start_file_watcher};

/// Background driver that wires local file watcher events and remote version
/// polling into a [`HotReloadRuntime`].
pub struct HotReloadDriver {
    watcher: FileWatcherHandle,
    local_task: JoinHandle<()>,
    remote_task: JoinHandle<()>,
}

impl HotReloadDriver {
    /// Starts the hot-reload background driver.
    ///
    /// Returns `Ok(None)` when no targets are registered.
    ///
    /// # Errors
    ///
    /// Returns an error when the watcher cannot be started.
    pub async fn start(
        runtime: Arc<HotReloadRuntime>,
        debounce_ms: u64,
        sync_interval: Duration,
    ) -> Result<Option<Self>> {
        let (roots, patterns) = runtime.watcher_roots_and_patterns()?;
        if roots.is_empty() {
            return Ok(None);
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<PathBuf>();
        let callback_tx = tx.clone();
        let watcher_config = WatcherConfig {
            paths: roots
                .iter()
                .map(|root| root.display().to_string())
                .collect::<Vec<_>>(),
            patterns: if patterns.is_empty() {
                vec!["**/*".to_string()]
            } else {
                patterns
            },
            exclude: vec![],
            debounce_ms,
            recursive: true,
        };

        let watcher = start_file_watcher(
            watcher_config,
            Some(move |(event, _omni_event): WatcherResult| {
                if let Some(path) = event_path(&event) {
                    let _ = callback_tx.send(path);
                }
            }),
        )
        .await
        .map_err(|error| anyhow!("failed to start hot reload watcher: {error}"))?;

        let runtime_for_local = Arc::clone(&runtime);
        let local_task = tokio::spawn(async move {
            while let Some(path) = rx.recv().await {
                if let Err(error) = runtime_for_local.on_local_path_change(&path) {
                    log::warn!("hot reload local path handler failed: {error}");
                }
            }
        });

        let runtime_for_remote = Arc::clone(&runtime);
        let remote_task = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(sync_interval);
            loop {
                ticker.tick().await;
                if let Err(error) = runtime_for_remote.sync_remote_versions() {
                    log::warn!("hot reload remote sync failed: {error}");
                }
            }
        });

        Ok(Some(Self {
            watcher,
            local_task,
            remote_task,
        }))
    }

    /// Stops the driver and aborts background tasks.
    pub async fn stop(self) {
        self.watcher.stop().await;
        self.local_task.abort();
        self.remote_task.abort();
    }
}

impl Drop for HotReloadDriver {
    fn drop(&mut self) {
        self.local_task.abort();
        self.remote_task.abort();
    }
}

fn event_path(event: &FileEvent) -> Option<PathBuf> {
    let raw = match event {
        FileEvent::Created { path, .. }
        | FileEvent::Modified { path }
        | FileEvent::Deleted { path, .. }
        | FileEvent::Error { path, .. } => path.as_str(),
    };
    if raw.trim().is_empty() {
        None
    } else {
        Some(PathBuf::from(raw))
    }
}
