//! Source-contract admission helpers for Episteme repositories.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;
use xiuxian_wendao_parsers::{
    EpistemeSourceContractParseError, EpistemeSourceManifest, parse_episteme_source_manifest_toml,
};

use crate::ontology::ONTOLOGY_MANIFEST_RELATIVE_PATH;

#[derive(Debug, Deserialize)]
struct EpistemeOntologyManifest {
    #[serde(default)]
    active_source_contract: Option<EpistemeActiveSourceContract>,
    #[serde(default)]
    domains: Vec<EpistemeDomainManifest>,
}

/// Active source-contract selector declared in `ontology/manifest.toml`.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct EpistemeActiveSourceContract {
    /// Domain id selected for runtime admission.
    pub domain_id: String,
    /// Source manifest path relative to `ontology/`.
    pub source_manifest: String,
    /// Mapping ledger path relative to `ontology/`.
    pub mapping_ledger: String,
}

/// Domain declaration from `ontology/manifest.toml`.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct EpistemeDomainManifest {
    /// Stable Episteme domain id.
    pub id: String,
    /// Source manifest paths relative to `ontology/`.
    #[serde(default)]
    pub source_manifests: Vec<String>,
    /// Mapping ledger paths relative to `ontology/`.
    #[serde(default)]
    pub mapping_ledgers: Vec<String>,
}

/// Selected source-contract paths resolved from an Episteme repository.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeSourceContractPaths {
    domain_id: String,
    source_manifest_relative_path: String,
    mapping_ledger_relative_path: String,
}

impl EpistemeSourceContractPaths {
    /// Selected domain id.
    #[must_use]
    pub fn domain_id(&self) -> &str {
        self.domain_id.as_str()
    }

    /// Source manifest path relative to the Episteme repository root.
    #[must_use]
    pub fn source_manifest_relative_path(&self) -> &str {
        self.source_manifest_relative_path.as_str()
    }

    /// Mapping ledger path relative to the Episteme repository root.
    #[must_use]
    pub fn mapping_ledger_relative_path(&self) -> &str {
        self.mapping_ledger_relative_path.as_str()
    }

    /// Source manifest absolute path for the provided Episteme repository root.
    #[must_use]
    pub fn source_manifest_path(&self, episteme_root: impl AsRef<Path>) -> PathBuf {
        episteme_root
            .as_ref()
            .join(self.source_manifest_relative_path.as_str())
    }

    /// Mapping ledger absolute path for the provided Episteme repository root.
    #[must_use]
    pub fn mapping_ledger_path(&self, episteme_root: impl AsRef<Path>) -> PathBuf {
        episteme_root
            .as_ref()
            .join(self.mapping_ledger_relative_path.as_str())
    }

    /// Corpus contract directory derived from the selected source manifest.
    ///
    /// # Errors
    ///
    /// Returns an error when the selected source manifest path has no parent
    /// directory.
    pub fn corpus_dir(&self, episteme_root: impl AsRef<Path>) -> Result<PathBuf, EpistemeError> {
        let manifest_relative_path = Path::new(self.source_manifest_relative_path.as_str());
        let Some(relative_dir) = manifest_relative_path.parent() else {
            return Err(EpistemeError::InvalidEpistemeManifest(format!(
                "source manifest path must have a parent directory: {}",
                self.source_manifest_relative_path
            )));
        };
        Ok(episteme_root.as_ref().join(relative_dir))
    }

    /// Build a repository-relative corpus artifact path beside the source
    /// manifest.
    #[must_use]
    pub fn corpus_relative_path(&self, file_name: &str) -> String {
        let manifest_relative_path = Path::new(self.source_manifest_relative_path.as_str());
        manifest_relative_path
            .parent()
            .map_or_else(PathBuf::new, Path::to_path_buf)
            .join(file_name)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

/// Error returned by Rust-owned Episteme source-contract admission.
#[derive(Debug, Error)]
pub enum EpistemeError {
    /// A file or directory could not be accessed.
    #[error("failed to access `{path}`: {source}")]
    Io {
        /// Path that failed.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// A parser-owned source-contract input is malformed.
    #[error("failed to parse episteme source-contract source contract `{path}`: {source}")]
    Parse {
        /// Source contract path.
        path: PathBuf,
        /// Parser-owned parse error.
        #[source]
        source: EpistemeSourceContractParseError,
    },
    /// The episteme source-contract ontology manifest is malformed.
    #[error("failed to parse episteme source-contract manifest `{path}`: {source}")]
    EpistemeManifestToml {
        /// Manifest path.
        path: PathBuf,
        /// Underlying TOML error.
        #[source]
        source: toml::de::Error,
    },
    /// The episteme config cannot select one source contract.
    #[error("episteme source-contract manifest is invalid: {0}")]
    InvalidEpistemeManifest(String),
}

/// Return the configured corpus-root environment variable for an Episteme
/// repository.
///
/// The value is read from the source manifest selected by
/// `ontology/manifest.toml`; it is not hardcoded by Rust.
///
/// # Errors
///
/// Returns an error when the Episteme config or selected source manifest cannot
/// be read or parsed.
pub fn configured_episteme_corpus_root_env(
    episteme_root: impl AsRef<Path>,
) -> Result<String, EpistemeError> {
    Ok(read_source_manifest(episteme_root)?.corpus_root_env)
}

/// Read the parser-owned source manifest selected by `ontology/manifest.toml`.
///
/// # Errors
///
/// Returns an error when manifest selection fails, the selected source manifest
/// cannot be read, or the selected source manifest domain does not match the
/// selected ontology domain.
pub fn read_source_manifest(
    episteme_root: impl AsRef<Path>,
) -> Result<EpistemeSourceManifest, EpistemeError> {
    let episteme_root = episteme_root.as_ref();
    let paths = source_contract_paths(episteme_root)?;
    let path = paths.source_manifest_path(episteme_root);
    let raw = read_to_string(&path)?;
    let manifest = parse_episteme_source_manifest_toml(&raw)
        .map_err(|source| EpistemeError::Parse { path, source })?;
    if manifest.domain != paths.domain_id() {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "selected source manifest domain `{}` does not match selected manifest domain `{}`",
            manifest.domain,
            paths.domain_id()
        )));
    }
    Ok(manifest)
}

/// Select the active Episteme source-contract paths from `ontology/manifest.toml`.
///
/// # Errors
///
/// Returns an error when the ontology manifest is missing, malformed, unsafe,
/// or cannot unambiguously select one source contract.
pub fn source_contract_paths(
    episteme_root: impl AsRef<Path>,
) -> Result<EpistemeSourceContractPaths, EpistemeError> {
    let episteme_root = episteme_root.as_ref();
    let manifest_path = episteme_root.join(ONTOLOGY_MANIFEST_RELATIVE_PATH);
    if !manifest_path.is_file() {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "missing episteme config: {ONTOLOGY_MANIFEST_RELATIVE_PATH}"
        )));
    }

    let raw = read_to_string(&manifest_path)?;
    let manifest = toml::from_str::<EpistemeOntologyManifest>(&raw).map_err(|source| {
        EpistemeError::EpistemeManifestToml {
            path: manifest_path.clone(),
            source,
        }
    })?;
    let selected = select_source_contract_paths(&manifest)?;

    Ok(EpistemeSourceContractPaths {
        domain_id: selected.domain_id,
        source_manifest_relative_path: format!("ontology/{}", selected.source_manifest),
        mapping_ledger_relative_path: format!("ontology/{}", selected.mapping_ledger),
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SelectedSourceContractPaths {
    domain_id: String,
    source_manifest: String,
    mapping_ledger: String,
}

fn select_source_contract_paths(
    manifest: &EpistemeOntologyManifest,
) -> Result<SelectedSourceContractPaths, EpistemeError> {
    if let Some(active) = &manifest.active_source_contract {
        return select_active_source_contract(manifest, active);
    }

    let declared = declared_source_contracts(manifest)?;
    match declared.as_slice() {
        [selected] => Ok(selected.clone()),
        [] => Err(EpistemeError::InvalidEpistemeManifest(
            "expected at least one declared source manifest and mapping ledger".to_string(),
        )),
        many => Err(EpistemeError::InvalidEpistemeManifest(format!(
            "found {} selectable source contracts; add [active_source_contract] to ontology/manifest.toml",
            many.len()
        ))),
    }
}

fn select_active_source_contract(
    manifest: &EpistemeOntologyManifest,
    active: &EpistemeActiveSourceContract,
) -> Result<SelectedSourceContractPaths, EpistemeError> {
    let source_manifest = normalize_episteme_source_contract_manifest_path(
        &active.source_manifest,
        "source_manifest",
    )?;
    let mapping_ledger =
        normalize_episteme_source_contract_manifest_path(&active.mapping_ledger, "mapping_ledger")?;
    let domain = manifest
        .domains
        .iter()
        .find(|domain| domain.id == active.domain_id)
        .ok_or_else(|| {
            EpistemeError::InvalidEpistemeManifest(format!(
                "active source contract references unknown domain_id: {}",
                active.domain_id
            ))
        })?;
    let declared_source_manifests = normalized_domain_paths(domain, "source_manifests")?;
    if !declared_source_manifests.contains(&source_manifest) {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "active source_manifest is not declared by domain {}: {}",
            active.domain_id, source_manifest
        )));
    }
    let declared_mapping_ledgers = normalized_domain_paths(domain, "mapping_ledgers")?;
    if !declared_mapping_ledgers.contains(&mapping_ledger) {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "active mapping_ledger is not declared by domain {}: {}",
            active.domain_id, mapping_ledger
        )));
    }

    Ok(SelectedSourceContractPaths {
        domain_id: active.domain_id.clone(),
        source_manifest,
        mapping_ledger,
    })
}

fn declared_source_contracts(
    manifest: &EpistemeOntologyManifest,
) -> Result<Vec<SelectedSourceContractPaths>, EpistemeError> {
    let mut selected = Vec::new();
    for domain in &manifest.domains {
        let source_manifests = normalized_domain_paths(domain, "source_manifests")?;
        let mapping_ledgers = normalized_domain_paths(domain, "mapping_ledgers")?;
        if source_manifests.len() == 1 && mapping_ledgers.len() == 1 {
            selected.push(SelectedSourceContractPaths {
                domain_id: domain.id.clone(),
                source_manifest: source_manifests[0].clone(),
                mapping_ledger: mapping_ledgers[0].clone(),
            });
        } else if !source_manifests.is_empty() || !mapping_ledgers.is_empty() {
            return Err(EpistemeError::InvalidEpistemeManifest(format!(
                "domain {} declares {} source manifests and {} mapping ledgers; add [active_source_contract]",
                domain.id,
                source_manifests.len(),
                mapping_ledgers.len()
            )));
        }
    }
    Ok(selected)
}

fn normalized_domain_paths(
    domain: &EpistemeDomainManifest,
    field: &str,
) -> Result<Vec<String>, EpistemeError> {
    let values = match field {
        "source_manifests" => &domain.source_manifests,
        "mapping_ledgers" => &domain.mapping_ledgers,
        _ => unreachable!("unsupported episteme domain path field"),
    };
    values
        .iter()
        .map(|path| normalize_episteme_source_contract_manifest_path(path, field))
        .collect()
}

fn normalize_episteme_source_contract_manifest_path(
    raw: &str,
    field: &str,
) -> Result<String, EpistemeError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "`{field}` entries must not be blank"
        )));
    }
    let path = Path::new(trimmed);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "`{field}` entries must be safe paths relative to ontology/: {trimmed}"
        )));
    }
    Ok(trimmed.replace('\\', "/"))
}

fn read_to_string(path: &Path) -> Result<String, EpistemeError> {
    fs::read_to_string(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
#[path = "../tests/unit/source_contract.rs"]
mod tests;
