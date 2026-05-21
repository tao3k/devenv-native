//! Launch metadata for artifact-producing Wendao commands.

use serde::{Deserialize, Serialize};

/// Generic launch specification for a managed plugin process.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginLaunchSpec {
    /// Launcher path relative to the repository root.
    pub launcher_path: String,
    /// Ordered launcher arguments.
    pub args: Vec<String>,
}
