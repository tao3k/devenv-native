//! Transport-kind catalog for Wendao plugin runtime data exchange.

use serde::{Deserialize, Serialize};

/// Supported transport kinds for Wendao plugin-runtime data exchange.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginTransportKind {
    /// Apache Arrow Flight RPC.
    #[default]
    ArrowFlight,
}
