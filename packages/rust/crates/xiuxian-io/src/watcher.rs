//! Rust-Native File Watcher with Event Bus Integration
//!
//! Uses `notify` crate for cross-platform file system monitoring.
//! Publishes file events to the global `xiuxian-event` `EventBus` for reactive architecture.

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use globset::{Glob, GlobSetBuilder};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::mpsc;

#[cfg(feature = "notify")]
use xiuxian_event::{GLOBAL_BUS, OmniEvent, topics};

use crate::error::Result;

/// Configuration for file watcher.
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Paths to watch
    pub paths: Vec<String>,
    /// File patterns to include (glob patterns)
    pub patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude: Vec<String>,
    /// Debounce duration for rapid changes
    pub debounce_ms: u64,
    /// Whether to watch recursively
    pub recursive: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            paths: vec![],
            patterns: vec!["**/*".to_string()],
            exclude: vec![
                "**/*.pyc".to_string(),
                "**/__pycache__/**".to_string(),
                "**/.git/**".to_string(),
                "**/*.tmp".to_string(),
            ],
            debounce_ms: 100,
            recursive: true,
        }
    }
}

/// Path evidence shared by watcher events.
#[derive(Debug, Clone)]
pub struct FileEventPath {
    /// Path to the file or directory.
    pub path: String,
}

/// Path and directory evidence shared by create/delete events.
#[derive(Debug, Clone)]
pub struct FileEventPathKind {
    /// Path to the file or directory.
    pub path: String,
    /// Whether the path is a directory.
    pub is_dir: bool,
}

/// Watcher backend error event.
#[derive(Debug, Clone)]
pub struct FileEventError {
    /// Path associated with the error when available.
    pub path: String,
    /// Error message.
    pub error: String,
}

/// File system event types matching `notify` crate.
#[derive(Debug, Clone)]
pub enum FileEvent {
    /// File was created.
    Created(FileEventPathKind),
    /// File was modified.
    Modified(FileEventPath),
    /// File was deleted.
    Deleted(FileEventPathKind),
    /// Error occurred while processing watcher events.
    Error(FileEventError),
}

/// Result from file watcher
pub type WatcherResult = (FileEvent, Option<OmniEvent>);

/// Handle to control the file watcher.
#[derive(Clone)]
pub struct FileWatcherHandle {
    tx: mpsc::Sender<()>,
}

impl FileWatcherHandle {
    /// Stop the watcher.
    pub async fn stop(&self) {
        let _ = self.tx.send(()).await;
    }
}

/// Convert `notify` event kind to topic and handle macOS edge cases.
#[cfg(feature = "notify")]
fn event_to_topic_and_path(kind: notify::EventKind, path: &Path) -> (&'static str, String) {
    let path_str = path.to_string_lossy().to_string();

    // Handle macOS edge case: some editors may send Create/Modify instead of Remove
    // when a file is deleted. We check if the file actually exists.
    if (matches!(kind, notify::EventKind::Create(_))
        || matches!(kind, notify::EventKind::Modify(_)))
        && !path.exists()
    {
        // File was reported as created/modified but doesn't exist -> it was deleted
        return (topics::FILE_DELETED, path_str);
    }

    (
        match kind {
            notify::EventKind::Create(_) => topics::FILE_CREATED,
            notify::EventKind::Remove(_) => topics::FILE_DELETED,
            _ => topics::FILE_CHANGED,
        },
        path_str,
    )
}

/// Check if path matches any pattern using high-performance `GlobSet`.
fn matches_patterns(path: &Path, patterns: &[String], exclude: &[String]) -> bool {
    let path_str = path.to_string_lossy();

    // Build exclude set
    let exclude_set = if exclude.is_empty() {
        None
    } else {
        let mut builder = GlobSetBuilder::new();
        for ex in exclude {
            if let Ok(glob) = Glob::new(ex) {
                builder.add(glob);
            }
        }
        Some(builder.build())
    };

    // Check exclude patterns first
    if let Some(Ok(set)) = exclude_set {
        if set.matches(&*path_str).is_empty() {
            // Continue include matching.
        } else {
            return false;
        }
    }

    // Build include set
    if !patterns.is_empty() {
        let mut builder = GlobSetBuilder::new();
        for pat in patterns {
            if let Ok(glob) = Glob::new(pat) {
                builder.add(glob);
            }
        }
        if let Ok(set) = builder.build() {
            return !set.matches(&*path_str).is_empty();
        }
    }

    patterns.is_empty()
}

/// Start a file watcher that publishes events to the global `EventBus`.
///
/// # Errors
///
/// Returns an error if watcher initialization fails or any configured path cannot be watched.
#[cfg(feature = "notify")]
pub async fn start_file_watcher<F>(
    config: WatcherConfig,
    callback: Option<F>,
) -> Result<FileWatcherHandle>
where
    F: Fn(WatcherResult) + Send + 'static,
{
    let (tx, mut rx) = mpsc::channel(1);
    let debounce_map = std::sync::Arc::new(TokioMutex::new(HashMap::new()));
    let debounce_duration = Duration::from_millis(config.debounce_ms);
    let (watcher_tx, mut watcher_rx) = mpsc::channel(100);
    let mut watcher = build_notify_watcher(watcher_tx)?;
    watch_configured_paths(&mut watcher, &config)?;

    let patterns = config.patterns.clone();
    let exclude = config.exclude.clone();
    let cb = callback;

    let _task = tokio::spawn(async move {
        let _watcher = watcher;
        loop {
            tokio::select! {
                _ = rx.recv() => {
                    break;
                }
                result = watcher_rx.recv() => {
                    match result {
                        Some(Ok(event)) => {
                            if let Some(result) = process_notify_event(
                                event,
                                &patterns,
                                &exclude,
                                &debounce_map,
                                debounce_duration,
                            ).await {
                                if let Some(ref cb) = cb {
                                    cb(result);
                                }
                            }
                        }
                        Some(Err(e)) => {
                            if let Some(ref cb) = cb {
                                cb((FileEvent::Error(FileEventError {
                                    path: String::new(),
                                    error: e.to_string(),
                                }), None));
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
        }
    });

    Ok(FileWatcherHandle { tx })
}

/// Start watching with default config.
///
/// # Errors
///
/// Returns an error if the watcher cannot be created for `path`.
#[cfg(feature = "notify")]
pub async fn watch_path<P: AsRef<Path>>(path: P) -> Result<FileWatcherHandle> {
    let config = WatcherConfig {
        paths: vec![path.as_ref().to_string_lossy().to_string()],
        ..WatcherConfig::default()
    };
    start_file_watcher::<fn(WatcherResult)>(config, None).await
}

#[cfg(feature = "notify")]
fn build_notify_watcher(
    watcher_tx: mpsc::Sender<std::result::Result<Event, notify::Error>>,
) -> Result<RecommendedWatcher> {
    RecommendedWatcher::new(
        move |result| {
            let _ = watcher_tx.blocking_send(result);
        },
        Config::default().with_poll_interval(Duration::from_millis(50)),
    )
    .map_err(Into::into)
}

#[cfg(feature = "notify")]
fn watch_configured_paths(watcher: &mut RecommendedWatcher, config: &WatcherConfig) -> Result<()> {
    let mode = watch_recursive_mode(config.recursive);
    for path in &config.paths {
        watcher.watch(Path::new(path), mode)?;
    }
    Ok(())
}

#[cfg(feature = "notify")]
fn watch_recursive_mode(recursive: bool) -> RecursiveMode {
    if recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    }
}

#[cfg(feature = "notify")]
async fn process_notify_event(
    event: Event,
    patterns: &[String],
    exclude: &[String],
    debounce_map: &TokioMutex<HashMap<String, Instant>>,
    debounce_duration: Duration,
) -> Option<WatcherResult> {
    let path = event.paths.first()?;
    if !matches_patterns(path, patterns, exclude) {
        return None;
    }
    if debounce_rejects_event(&event, path, debounce_map, debounce_duration).await {
        return None;
    }

    let (topic, final_path_str) = event_to_topic_and_path(event.kind.clone(), path);
    let file_event = file_event_from_topic(topic, &final_path_str, path);
    let bus_event = OmniEvent::new(
        "watcher",
        topic,
        watcher_payload(&final_path_str, path.is_dir(), topic, &event),
    );
    let _ = GLOBAL_BUS.publish(bus_event.clone());
    Some((file_event, Some(bus_event)))
}

#[cfg(feature = "notify")]
async fn debounce_rejects_event(
    event: &Event,
    path: &Path,
    debounce_map: &TokioMutex<HashMap<String, Instant>>,
    debounce_duration: Duration,
) -> bool {
    if !matches!(event.kind, notify::EventKind::Modify(_)) {
        return false;
    }

    let path_str = path.to_string_lossy().to_string();
    let mut debounce = debounce_map.lock().await;
    let now = Instant::now();
    if let Some(last) = debounce.get(&path_str)
        && now.duration_since(*last) < debounce_duration
    {
        return true;
    }
    debounce.insert(path_str, now);
    false
}

#[cfg(feature = "notify")]
fn file_event_from_topic(topic: &str, path: &str, source_path: &Path) -> FileEvent {
    match topic {
        topics::FILE_DELETED => FileEvent::Deleted(FileEventPathKind {
            path: path.to_string(),
            is_dir: source_path.is_dir(),
        }),
        topics::FILE_CREATED => FileEvent::Created(FileEventPathKind {
            path: path.to_string(),
            is_dir: source_path.is_dir(),
        }),
        _ => FileEvent::Modified(FileEventPath {
            path: path.to_string(),
        }),
    }
}

#[cfg(feature = "notify")]
fn watcher_payload(path: &str, is_dir: bool, topic: &str, event: &Event) -> serde_json::Value {
    serde_json::json!({
        "path": path,
        "is_dir": is_dir,
        "event_type": format!("{:?}", event.kind),
        "resolved_type": topic,
    })
}

#[cfg(all(test, feature = "notify"))]
#[path = "../tests/unit/watcher.rs"]
mod tests;
