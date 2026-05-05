//! Runtime context passed from CLI parsing into command executors.

use crate::OutputFormat;
use std::path::{Path, PathBuf};

/// Execution context shared by standalone and embedded client commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientContext {
    root: PathBuf,
    config_file: Option<PathBuf>,
    output: OutputFormat,
}

impl ClientContext {
    /// Construct one client execution context.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, output: OutputFormat) -> Self {
        let root = root.into();
        Self {
            root: absolutize(&root),
            config_file: None,
            output,
        }
    }

    /// Root directory used to resolve relative input paths.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Optional config path used by config-aware commands.
    #[must_use]
    pub fn config_file(&self) -> Option<&Path> {
        self.config_file.as_deref()
    }

    /// Output mode for rendered command results.
    #[must_use]
    pub fn output(&self) -> OutputFormat {
        self.output
    }

    /// Attach an optional config path to the client execution context.
    #[must_use]
    pub fn with_config_file(mut self, config_file: Option<PathBuf>) -> Self {
        self.config_file = config_file.map(|path| absolutize(path.as_path()));
        self
    }
}

fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir().map_or_else(|_| path.to_path_buf(), |cwd| cwd.join(path))
}
