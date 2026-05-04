use serde::{Deserialize, Serialize};
use specta::Type;

/// Read-only project configuration required by the Wendao search runtime.
pub trait ProjectConfigView {
    /// Unique project name.
    fn project_name(&self) -> &str;

    /// Configured project root.
    fn project_root(&self) -> &str;

    /// Explicit subdirectories to index.
    fn project_dirs(&self) -> &[String];
}

/// Domain search configuration for a local project root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchProjectConfig {
    /// Unique name.
    pub name: String,
    /// Relative path to project root.
    pub root: String,
    /// Explicit subdirectories to index.
    pub dirs: Vec<String>,
}

impl ProjectConfigView for SearchProjectConfig {
    fn project_name(&self) -> &str {
        self.name.as_str()
    }

    fn project_root(&self) -> &str {
        self.root.as_str()
    }

    fn project_dirs(&self) -> &[String] {
        self.dirs.as_slice()
    }
}

#[cfg(feature = "search-runtime")]
pub(crate) fn materialize_project_configs(
    projects: &[impl ProjectConfigView],
) -> Vec<SearchProjectConfig> {
    projects
        .iter()
        .map(|project| SearchProjectConfig {
            name: project.project_name().to_string(),
            root: project.project_root().to_string(),
            dirs: project.project_dirs().to_vec(),
        })
        .collect()
}
