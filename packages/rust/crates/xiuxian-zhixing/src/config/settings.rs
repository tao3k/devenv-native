//! Runtime settings model for Zhixing-Heyi template and carryover policy.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the Zhixing-Heyi plugin.
///
/// This is intended to be embedded within the main Wendao TOML configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZhixingConfig {
    /// A list of paths to search for templates.
    /// Order defines priority (first path found wins).
    #[serde(default = "default_template_paths")]
    pub template_paths: Vec<PathBuf>,

    /// Threshold for mandatory review (Strict Mode).
    #[serde(default = "default_carryover_threshold")]
    pub carryover_threshold: i32,
}

fn default_template_paths() -> Vec<PathBuf> {
    Vec::new()
}

fn default_carryover_threshold() -> i32 {
    3
}

impl ZhixingConfig {
    /// Returns a glob pattern for Tera based on all configured paths.
    #[must_use]
    pub fn get_tera_globs(&self) -> Vec<String> {
        self.template_paths
            .iter()
            .map(|p| format!("{}/*.md", p.to_string_lossy()))
            .collect()
    }
}
