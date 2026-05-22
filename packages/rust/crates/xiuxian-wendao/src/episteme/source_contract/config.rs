//! Episteme-owned runtime defaults.

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::EpistemeError;

const EPISTEME_TOML: &str = "episteme.toml";

/// Runtime defaults loaded from an optional episteme-owned `episteme.toml`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeRuntimeConfig {
    /// Resolved source corpus root.
    pub corpus: Option<PathBuf>,
    /// Resolved structure run root.
    pub structure_runs: Option<PathBuf>,
    /// Resolved evidence selection run root.
    pub evidence_selection_runs: Option<PathBuf>,
    /// Resolved extraction run root.
    pub extraction_runs: Option<PathBuf>,
    /// Resolved ontology candidate generation run root.
    pub ontology_generation_runs: Option<PathBuf>,
    /// Resolved legacy Office converter executable or wrapper path.
    pub legacy_office_converter: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct EpistemeToml {
    #[serde(rename = "schema_version")]
    _schema_version: Option<u32>,
    #[serde(default)]
    runtime: EpistemeRuntimeDefaults,
}

#[derive(Debug, Default, Deserialize)]
struct EpistemeRuntimeDefaults {
    #[serde(rename = "corpus_root")]
    corpus: Option<PathBuf>,
    #[serde(rename = "structure_run_root")]
    structure_runs: Option<PathBuf>,
    #[serde(rename = "evidence_selection_run_root")]
    evidence_selection_runs: Option<PathBuf>,
    #[serde(rename = "extraction_run_root")]
    extraction_runs: Option<PathBuf>,
    #[serde(rename = "ontology_generation_run_root")]
    ontology_generation_runs: Option<PathBuf>,
    #[serde(rename = "legacy_office_converter")]
    legacy_office_converter: Option<PathBuf>,
}

/// Load optional episteme runtime defaults from `<episteme-root>/episteme.toml`.
///
/// # Errors
///
/// Returns an error when `episteme.toml` exists but cannot be read or parsed.
pub fn load_episteme_runtime_config(
    episteme_root: impl AsRef<Path>,
) -> Result<Option<EpistemeRuntimeConfig>, EpistemeError> {
    let episteme_root = episteme_root.as_ref();
    let path = episteme_root.join(EPISTEME_TOML);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path).map_err(|source| EpistemeError::Io {
        path: path.clone(),
        source,
    })?;
    let parsed = toml::from_str::<EpistemeToml>(&raw).map_err(|source| {
        EpistemeError::EpistemeManifestToml {
            path: path.clone(),
            source,
        }
    })?;
    Ok(Some(EpistemeRuntimeConfig {
        corpus: resolve_config_path(episteme_root, parsed.runtime.corpus),
        structure_runs: resolve_config_path(episteme_root, parsed.runtime.structure_runs),
        evidence_selection_runs: resolve_config_path(
            episteme_root,
            parsed.runtime.evidence_selection_runs,
        ),
        extraction_runs: resolve_config_path(episteme_root, parsed.runtime.extraction_runs),
        ontology_generation_runs: resolve_config_path(
            episteme_root,
            parsed.runtime.ontology_generation_runs,
        ),
        legacy_office_converter: resolve_config_path(
            episteme_root,
            parsed.runtime.legacy_office_converter,
        ),
    }))
}

fn resolve_config_path(episteme_root: &Path, path: Option<PathBuf>) -> Option<PathBuf> {
    path.map(|path| {
        let resolved = if path.is_absolute() {
            path
        } else {
            episteme_root.join(path)
        };
        normalize_path(resolved.as_path())
    })
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(value) => normalized.push(value),
        }
    }
    normalized
}
