use crate::OutputFormat;
use std::path::{Path, PathBuf};

/// Execution context shared by standalone and embedded client commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientContext {
    root: PathBuf,
    output: OutputFormat,
}

impl ClientContext {
    /// Construct one client execution context.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, output: OutputFormat) -> Self {
        let root = root.into();
        Self {
            root: absolutize(&root),
            output,
        }
    }

    /// Root directory used to resolve relative input paths.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Output mode for rendered command results.
    #[must_use]
    pub fn output(&self) -> OutputFormat {
        self.output
    }
}

fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir().map_or_else(|_| path.to_path_buf(), |cwd| cwd.join(path))
}
