//! Modelica repository file snapshot loading.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::surface::{RepositorySurface, repository_surface};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepositorySnapshot {
    entries: Vec<RepositoryFileEntry>,
    package_orders: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepositoryFileEntry {
    pub(crate) absolute_path: PathBuf,
    pub(crate) relative_path: String,
    pub(crate) surface: RepositorySurface,
    pub(crate) modelica_contents: Option<String>,
}

impl RepositorySnapshot {
    pub(crate) fn load(repository_root: &Path) -> Result<Self, RepoIntelligenceError> {
        let mut entries = Vec::new();
        let mut package_orders = BTreeMap::new();

        for absolute_path in repository_file_paths(repository_root) {
            let Some(relative_file_path) = relative_path(repository_root, absolute_path.as_path())
            else {
                continue;
            };
            let file_name = absolute_path.file_name().and_then(std::ffi::OsStr::to_str);
            let extension = absolute_path.extension().and_then(std::ffi::OsStr::to_str);
            let modelica_contents = if extension == Some("mo") {
                Some(read_repository_text_file(absolute_path.as_path())?)
            } else {
                None
            };

            if file_name == Some("package.order") {
                let parent_relative = absolute_path
                    .parent()
                    .and_then(|parent| relative_path(repository_root, parent))
                    .unwrap_or_default();
                let order_entries = read_repository_text_file(absolute_path.as_path())?
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .filter(|line| !line.starts_with("//"))
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                if !order_entries.is_empty() {
                    package_orders.insert(parent_relative, order_entries);
                }
            }

            entries.push(RepositoryFileEntry {
                surface: repository_surface(relative_file_path.as_str()),
                absolute_path,
                relative_path: relative_file_path,
                modelica_contents,
            });
        }

        Ok(Self {
            entries,
            package_orders,
        })
    }

    pub(crate) fn entries(&self) -> &[RepositoryFileEntry] {
        &self.entries
    }

    pub(crate) fn package_orders(&self) -> &BTreeMap<String, Vec<String>> {
        &self.package_orders
    }

    pub(crate) fn package_files(&self) -> Result<Vec<&RepositoryFileEntry>, RepoIntelligenceError> {
        let package_files = self
            .entries
            .iter()
            .filter(|entry| {
                entry
                    .absolute_path
                    .file_name()
                    .and_then(std::ffi::OsStr::to_str)
                    == Some("package.mo")
            })
            .collect::<Vec<_>>();
        if package_files.is_empty() {
            return Err(RepoIntelligenceError::AnalysisFailed {
                message: "no package.mo files discovered during Modelica analysis".to_string(),
            });
        }
        Ok(package_files)
    }
}

fn read_repository_text_file(path: &Path) -> Result<String, RepoIntelligenceError> {
    fs::read_to_string(path).map_err(|error| RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "failed to read repository file `{}`: {error}",
            path.display()
        ),
    })
}

pub(crate) fn relative_path(repository_root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(repository_root)
        .ok()
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
}

fn repository_file_paths(repository_root: &Path) -> Vec<PathBuf> {
    let mut files = WalkDir::new(repository_root)
        .into_iter()
        .filter_entry(|entry| !should_skip_walk_entry(entry))
        .filter_map(Result::ok)
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn should_skip_walk_entry(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 {
        return false;
    }
    entry
        .file_name()
        .to_str()
        .is_some_and(|name| name.starts_with('.'))
}
