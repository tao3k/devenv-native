//! Refresh request and result contracts for link-graph materialization.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Refresh execution mode selected by `LinkGraph` incremental refresh logic.
pub enum LinkGraphRefreshMode {
    /// No-op (no changed paths provided).
    Noop,
    /// Apply incremental delta updates.
    Delta,
    /// Run full index rebuild.
    Full,
}
