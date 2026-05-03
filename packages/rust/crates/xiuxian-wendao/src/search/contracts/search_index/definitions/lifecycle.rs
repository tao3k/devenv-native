use serde::{Deserialize, Serialize};
use specta::Type;

/// Search-plane lifecycle phase surfaced to Studio clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SearchIndexPhase {
    /// No build has been started for the corpus.
    Idle,
    /// A staging epoch is currently being built.
    Indexing,
    /// A published epoch is ready for reads.
    Ready,
    /// A published epoch is readable but partially stale or inconsistent.
    Degraded,
    /// The latest build attempt failed.
    Failed,
}
