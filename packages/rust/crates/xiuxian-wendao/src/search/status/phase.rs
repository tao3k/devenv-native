//! `search::status::phase` owns Wendao search status phase behavior.

use serde::{Deserialize, Serialize};

/// Runtime phase for a search-plane corpus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchPlanePhase {
    /// No build has been started for the corpus.
    #[default]
    Idle,
    /// A staging epoch is being built in the background.
    Indexing,
    /// A published epoch is available for reads.
    Ready,
    /// A published epoch is still readable, but the corpus is partially stale or inconsistent.
    Degraded,
    /// The latest attempted build failed.
    Failed,
}
