use std::fs;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::contracts::FlowhubModuleManifest;
use crate::error::QianjiError;

/// High-level Flowhub directory kind used by the unified CLI bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowhubDirKind {
    /// A Flowhub library root containing one or more module manifests below it.
    Root,
    /// A single Flowhub module directory with a root `qianji.toml`.
    Module,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FlowhubModuleCandidate {
    pub module_ref: String,
    pub module_dir: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FlowhubDiscoveredModule {
    pub module_ref: String,
    pub module_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: FlowhubModuleManifest,
}

/// Returns the Flowhub kind of a directory when it looks like a Flowhub root
/// or module.
///
/// Detection is intentionally shallow so invalid module manifests still route
/// into Flowhub `check` diagnostics instead of falling through as unknown
/// paths.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the directory contains a candidate
/// Flowhub manifest that cannot be read.
pub fn classify_flowhub_dir(dir: impl AsRef<Path>) -> Result<Option<FlowhubDirKind>, QianjiError> {
    let dir = dir.as_ref();
    if manifest_declares_module_contract(dir.join("qianji.toml"))? {
        return Ok(Some(FlowhubDirKind::Module));
    }

    if manifest_declares_flowhub_root_contract(dir.join("qianji.toml"))? {
        return Ok(Some(FlowhubDirKind::Root));
    }

    if !dir.is_dir() {
        return Ok(None);
    }

    if discover_flowhub_top_level_candidates(dir)?.is_empty() {
        Ok(None)
    } else {
        Ok(Some(FlowhubDirKind::Root))
    }
}

pub(super) fn module_candidate_from_dir(
    module_dir: &Path,
) -> Result<FlowhubModuleCandidate, QianjiError> {
    let root = find_flowhub_root_for_module_dir(module_dir)?;
    let module_dir = module_dir.to_path_buf();
    let module_ref = module_dir
        .strip_prefix(&root)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to resolve Flowhub module reference for `{}` against `{}`: {error}",
                module_dir.display(),
                root.display()
            ))
        })?
        .to_string_lossy()
        .replace('\\', "/");

    Ok(FlowhubModuleCandidate {
        module_ref,
        manifest_path: module_dir.join("qianji.toml"),
        module_dir,
    })
}

pub(super) fn discover_flowhub_top_level_candidates(
    root: &Path,
) -> Result<Vec<FlowhubModuleCandidate>, QianjiError> {
    let mut candidates = Vec::new();
    for candidate in discover_flowhub_module_candidates(root)? {
        let Some(parent_dir) = candidate.module_dir.parent() else {
            candidates.push(candidate);
            continue;
        };
        if parent_dir == root {
            candidates.push(candidate);
            continue;
        }
        if !manifest_declares_module_contract(parent_dir.join("qianji.toml"))? {
            candidates.push(candidate);
        }
    }
    Ok(candidates)
}

pub(super) fn discover_all_flowhub_module_refs(root: &Path) -> Result<Vec<String>, QianjiError> {
    Ok(discover_flowhub_module_candidates(root)?
        .into_iter()
        .map(|candidate| candidate.module_ref)
        .collect())
}

pub(super) fn load_flowhub_module_candidate(
    candidate: &FlowhubModuleCandidate,
) -> Result<FlowhubDiscoveredModule, QianjiError> {
    let manifest = super::load::load_flowhub_module_manifest(&candidate.manifest_path)?;
    Ok(FlowhubDiscoveredModule {
        module_ref: candidate.module_ref.clone(),
        module_dir: candidate.module_dir.clone(),
        manifest_path: candidate.manifest_path.clone(),
        manifest,
    })
}

pub(super) fn module_candidate_from_ref(root: &Path, module_ref: &str) -> FlowhubModuleCandidate {
    let module_dir = root.join(module_ref);
    FlowhubModuleCandidate {
        module_ref: module_ref.to_string(),
        manifest_path: module_dir.join("qianji.toml"),
        module_dir,
    }
}

fn discover_flowhub_module_candidates(
    root: &Path,
) -> Result<Vec<FlowhubModuleCandidate>, QianjiError> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    let walker = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(should_descend_into);

    for entry in walker {
        let entry = entry.map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to walk Flowhub directory `{}`: {error}",
                root.display()
            ))
        })?;
        if !entry.file_type().is_file() || entry.file_name() != "qianji.toml" {
            continue;
        }

        let Some(module_dir) = entry.path().parent() else {
            continue;
        };
        let relative = module_dir.strip_prefix(root).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to relativize Flowhub module directory `{}` against `{}`: {error}",
                module_dir.display(),
                root.display()
            ))
        })?;
        if relative.as_os_str().is_empty() {
            continue;
        }

        candidates.push(FlowhubModuleCandidate {
            module_ref: relative.to_string_lossy().replace('\\', "/"),
            module_dir: module_dir.to_path_buf(),
            manifest_path: entry.path().to_path_buf(),
        });
    }

    candidates.sort_by(|left, right| left.module_ref.cmp(&right.module_ref));
    Ok(candidates)
}

pub(super) fn find_flowhub_root_for_module_dir(module_dir: &Path) -> Result<PathBuf, QianjiError> {
    let mut current_root = module_dir
        .parent()
        .map_or_else(|| module_dir.to_path_buf(), Path::to_path_buf);
    let mut current = module_dir.parent();

    while let Some(parent) = current {
        if manifest_declares_module_contract(parent.join("qianji.toml"))? {
            current = parent.parent();
            continue;
        }
        current_root = parent.to_path_buf();
        break;
    }

    Ok(current_root)
}

fn manifest_declares_module_contract(manifest_path: impl AsRef<Path>) -> Result<bool, QianjiError> {
    let manifest_path = manifest_path.as_ref();
    if !manifest_path.is_file() {
        return Ok(false);
    }

    let manifest_toml = fs::read_to_string(manifest_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub module manifest `{}`: {error}",
            manifest_path.display()
        ))
    })?;

    if let Ok(value) = toml::from_str::<toml::Value>(&manifest_toml) {
        let Some(table) = value.as_table() else {
            return Ok(false);
        };
        return Ok(table.contains_key("module") && table.contains_key("exports"));
    }

    Ok(manifest_toml.contains("[module]") && manifest_toml.contains("[exports]"))
}

fn manifest_declares_flowhub_root_contract(
    manifest_path: impl AsRef<Path>,
) -> Result<bool, QianjiError> {
    let manifest_path = manifest_path.as_ref();
    if !manifest_path.is_file() {
        return Ok(false);
    }

    let manifest_toml = fs::read_to_string(manifest_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub root manifest `{}`: {error}",
            manifest_path.display()
        ))
    })?;

    if let Ok(value) = toml::from_str::<toml::Value>(&manifest_toml) {
        let Some(table) = value.as_table() else {
            return Ok(false);
        };
        return Ok(table.contains_key("flowhub") && table.contains_key("contract"));
    }

    Ok(manifest_toml.contains("[flowhub]") && manifest_toml.contains("[contract]"))
}

fn should_descend_into(entry: &DirEntry) -> bool {
    if entry.depth() == 0 {
        return true;
    }

    let name = entry.file_name().to_string_lossy();
    if name.starts_with('.') {
        return false;
    }

    !matches!(name.as_ref(), "template" | "validation" | "fixtures")
}
