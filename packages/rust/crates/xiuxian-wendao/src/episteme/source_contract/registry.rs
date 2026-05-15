//! Episteme repository registry loading.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use xiuxian_git_repo::{
    RepoError, RepoRefreshPolicy, RepoSpec, SyncMode, resolve_repository_source,
};

const REGISTRY_LOAD_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_registry_load.v1";
const REGISTRY_REFERENCE_GRAPH_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_registry_reference_graph.v1";
const ONTOLOGY_MANIFEST_RELATIVE_PATH: &str = "ontology/manifest.toml";
const DEFAULT_SUBDIR: &str = ".";

/// User-declared episteme registry entry.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeRegistryEntry {
    /// Stable registry id from deployment config.
    pub id: String,
    /// Local episteme repository path.
    pub path: Option<PathBuf>,
    /// Git episteme repository URL.
    pub url: Option<String>,
    /// Whether this entry should be loaded.
    pub enabled: bool,
    /// Optional subdirectory inside the local path or Git checkout.
    pub subdir: PathBuf,
}

impl EpistemeRegistryEntry {
    /// Create a local episteme registry entry.
    #[must_use]
    pub fn local(id: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            id: id.into(),
            path: Some(path.into()),
            url: None,
            enabled: true,
            subdir: PathBuf::from(DEFAULT_SUBDIR),
        }
    }

    /// Create a Git episteme registry entry.
    #[must_use]
    pub fn git(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            path: None,
            url: Some(url.into()),
            enabled: true,
            subdir: PathBuf::from(DEFAULT_SUBDIR),
        }
    }

    /// Attach a subdirectory to this registry entry.
    #[must_use]
    pub fn with_subdir(mut self, subdir: impl Into<PathBuf>) -> Self {
        self.subdir = subdir.into();
        self
    }

    /// Mark this registry entry as enabled or disabled.
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Runtime source kind for a loaded episteme repository.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoadedEpistemeSourceKind {
    /// Operator-provided local path.
    Local,
    /// Rust-managed Git checkout.
    Git,
}

/// One loaded episteme registry entry.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedEpistemeRegistryEntry {
    /// Stable registry id.
    pub id: String,
    /// Loaded source kind.
    pub source_kind: LoadedEpistemeSourceKind,
    /// Root directory that contains `ontology/manifest.toml`.
    pub episteme_root: PathBuf,
    /// Root source directory before applying `subdir`.
    pub source_root: PathBuf,
    /// Git URL for managed Git entries.
    pub url: Option<String>,
    /// Resolved Git revision for managed Git entries when available.
    pub resolved_revision: Option<String>,
    /// Subdirectory applied under the source root.
    pub subdir: String,
}

/// Receipt emitted after loading episteme registry entries.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeRegistryLoadReceipt {
    /// Receipt schema version.
    pub schema_version: &'static str,
    /// Number of entries loaded.
    pub loaded_count: usize,
    /// Loaded entries.
    pub entries: Vec<LoadedEpistemeRegistryEntry>,
}

/// Reference graph emitted for a loaded episteme registry.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeRegistryReferenceGraphReceipt {
    /// Receipt schema version.
    pub schema_version: &'static str,
    /// Number of loaded entries represented in this graph.
    pub entry_count: usize,
    /// Number of unique domain ids represented in this graph.
    pub domain_count: usize,
    /// Per-registry domain and extension facts.
    pub entries: Vec<EpistemeRegistryReferenceGraphEntry>,
    /// Resolved extension links.
    pub reference_links: Vec<EpistemeRegistryReferenceGraphLink>,
}

/// Domain and extension facts for one loaded registry entry.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeRegistryReferenceGraphEntry {
    /// Stable registry id.
    pub registry_id: String,
    /// Loaded episteme root.
    pub episteme_root: PathBuf,
    /// Domain ids declared by the episteme manifest.
    pub domain_ids: Vec<String>,
    /// Extension target domain ids declared by the episteme manifest.
    pub extension_targets: Vec<String>,
}

/// One resolved reference from a registry entry to a target domain owner.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeRegistryReferenceGraphLink {
    /// Registry id that declared the reference.
    #[serde(rename = "sourceRegistryId")]
    pub source_registry: String,
    /// Target domain id.
    #[serde(rename = "targetDomainId")]
    pub target_domain: String,
    /// Registry id that owns the target domain.
    #[serde(rename = "targetRegistryId")]
    pub target_registry: String,
}

/// Error returned by episteme registry loading.
#[derive(Debug, Error)]
pub enum EpistemeRegistryError {
    /// A registry entry id is empty or unsafe.
    #[error("invalid episteme registry id `{0}`; use ASCII letters, digits, '.', '_', or '-'")]
    InvalidId(String),
    /// A registry entry declares an invalid source shape.
    #[error("episteme registry `{id}` must declare exactly one of `path` or `url`")]
    InvalidSourceShape {
        /// Registry id.
        id: String,
    },
    /// A registry entry declares an unsafe subdirectory.
    #[error("episteme registry `{id}` has unsafe subdir `{subdir}`")]
    UnsafeSubdir {
        /// Registry id.
        id: String,
        /// Rejected subdir.
        subdir: String,
    },
    /// A local path is missing or is not a directory.
    #[error("episteme registry `{id}` path `{path}` is not a directory")]
    InvalidLocalPath {
        /// Registry id.
        id: String,
        /// Rejected path.
        path: PathBuf,
    },
    /// The materialized root is not an episteme repository.
    #[error("episteme registry `{id}` root `{root}` is missing ontology/manifest.toml")]
    MissingManifest {
        /// Registry id.
        id: String,
        /// Materialized episteme root.
        root: PathBuf,
    },
    /// A Git-backed entry could not be materialized.
    #[error("failed to materialize Git episteme registry `{id}` from `{url}`: {source}")]
    GitMaterialization {
        /// Registry id.
        id: String,
        /// Git URL.
        url: String,
        /// Repository substrate error.
        #[source]
        source: RepoError,
    },
    /// A loaded registry manifest could not be read.
    #[error("failed to read episteme registry `{id}` manifest `{path}`: {source}")]
    ManifestRead {
        /// Registry id.
        id: String,
        /// Manifest path.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// A loaded registry manifest could not be parsed.
    #[error("failed to parse episteme registry `{id}` manifest `{path}`: {source}")]
    ManifestToml {
        /// Registry id.
        id: String,
        /// Manifest path.
        path: PathBuf,
        /// TOML parse error.
        #[source]
        source: Box<toml::de::Error>,
    },
    /// A loaded registry manifest declares no domain ids.
    #[error("episteme registry `{id}` manifest declares no domains")]
    MissingDomains {
        /// Registry id.
        id: String,
    },
    /// A loaded registry manifest declares an invalid domain id.
    #[error("episteme registry `{id}` manifest declares invalid domain id `{domain_id}`")]
    InvalidDomainId {
        /// Registry id.
        id: String,
        /// Rejected domain id.
        domain_id: String,
    },
    /// Two loaded registry entries declare the same domain id.
    #[error(
        "episteme domain `{domain_id}` is declared by both registry `{first_registry_id}` and `{duplicate_registry_id}`"
    )]
    DuplicateDomainId {
        /// Duplicate domain id.
        domain_id: String,
        /// First registry id that declared the domain.
        first_registry_id: String,
        /// Later registry id that declared the same domain.
        duplicate_registry_id: String,
    },
    /// An extension target is not owned by any loaded registry entry.
    #[error(
        "episteme registry `{id}` extends `{target_domain_id}`, but no loaded registry owns it"
    )]
    MissingExtensionTarget {
        /// Registry id.
        id: String,
        /// Missing target domain id.
        target_domain_id: String,
    },
}

/// Load all enabled episteme registry entries with normal ensure semantics.
///
/// # Errors
///
/// Returns an error when any enabled entry has invalid shape, an unsafe path,
/// missing episteme manifest, or failed Git materialization.
pub fn load_episteme_registry_entries(
    entries: &[EpistemeRegistryEntry],
    project_root: impl AsRef<Path>,
) -> Result<EpistemeRegistryLoadReceipt, EpistemeRegistryError> {
    load_episteme_registry_entries_with_mode(entries, project_root, SyncMode::Ensure)
}

/// Load all enabled episteme registry entries with an explicit repository sync mode.
///
/// # Errors
///
/// Returns an error when any enabled entry has invalid shape, an unsafe path,
/// missing episteme manifest, or failed Git materialization.
pub fn load_episteme_registry_entries_with_mode(
    entries: &[EpistemeRegistryEntry],
    project_root: impl AsRef<Path>,
    mode: SyncMode,
) -> Result<EpistemeRegistryLoadReceipt, EpistemeRegistryError> {
    let mut loaded = Vec::new();
    for entry in entries {
        if !entry.enabled {
            continue;
        }
        loaded.push(load_episteme_registry_entry(
            entry,
            project_root.as_ref(),
            mode,
        )?);
    }
    Ok(EpistemeRegistryLoadReceipt {
        schema_version: REGISTRY_LOAD_SCHEMA_VERSION,
        loaded_count: loaded.len(),
        entries: loaded,
    })
}

/// Validate domain references across loaded episteme registry entries.
///
/// # Errors
///
/// Returns an error when a loaded manifest cannot be parsed, a domain id is
/// duplicated, or a declared extension target is not owned by any loaded
/// registry entry.
pub fn validate_episteme_registry_reference_graph(
    receipt: &EpistemeRegistryLoadReceipt,
) -> Result<EpistemeRegistryReferenceGraphReceipt, EpistemeRegistryError> {
    let mut domain_owner = BTreeMap::<String, String>::new();
    let mut graph_entries = Vec::with_capacity(receipt.entries.len());

    for loaded in &receipt.entries {
        let facts = read_registry_manifest_facts(loaded)?;
        for domain_id in &facts.domain_ids {
            if let Some(first_registry_id) =
                domain_owner.insert(domain_id.clone(), loaded.id.clone())
            {
                return Err(EpistemeRegistryError::DuplicateDomainId {
                    domain_id: domain_id.clone(),
                    first_registry_id,
                    duplicate_registry_id: loaded.id.clone(),
                });
            }
        }
        graph_entries.push(EpistemeRegistryReferenceGraphEntry {
            registry_id: loaded.id.clone(),
            episteme_root: loaded.episteme_root.clone(),
            domain_ids: facts.domain_ids,
            extension_targets: facts.extension_targets,
        });
    }

    let mut reference_links = Vec::new();
    for entry in &graph_entries {
        for target_domain_id in &entry.extension_targets {
            let Some(target_registry_id) = domain_owner.get(target_domain_id) else {
                return Err(EpistemeRegistryError::MissingExtensionTarget {
                    id: entry.registry_id.clone(),
                    target_domain_id: target_domain_id.clone(),
                });
            };
            reference_links.push(EpistemeRegistryReferenceGraphLink {
                source_registry: entry.registry_id.clone(),
                target_domain: target_domain_id.clone(),
                target_registry: target_registry_id.clone(),
            });
        }
    }

    Ok(EpistemeRegistryReferenceGraphReceipt {
        schema_version: REGISTRY_REFERENCE_GRAPH_SCHEMA_VERSION,
        entry_count: graph_entries.len(),
        domain_count: domain_owner.len(),
        entries: graph_entries,
        reference_links,
    })
}

fn load_episteme_registry_entry(
    entry: &EpistemeRegistryEntry,
    project_root: &Path,
    mode: SyncMode,
) -> Result<LoadedEpistemeRegistryEntry, EpistemeRegistryError> {
    let id = normalized_registry_id(entry.id.as_str())?;
    let subdir = normalized_subdir(&id, &entry.subdir)?;
    match (
        normalized_path(entry.path.as_deref()),
        normalized_url(entry.url.as_deref()),
    ) {
        (Some(path), None) => load_local_entry(id, project_root, path.as_path(), subdir.as_path()),
        (None, Some(url)) => load_git_entry(id, project_root, url, subdir.as_path(), mode),
        _ => Err(EpistemeRegistryError::InvalidSourceShape { id }),
    }
}

fn load_local_entry(
    id: String,
    project_root: &Path,
    path: &Path,
    subdir: &Path,
) -> Result<LoadedEpistemeRegistryEntry, EpistemeRegistryError> {
    let source_root = resolve_project_path(project_root, path);
    if !source_root.is_dir() {
        return Err(EpistemeRegistryError::InvalidLocalPath {
            id,
            path: source_root,
        });
    }
    let episteme_root = apply_subdir(source_root.as_path(), subdir);
    ensure_manifest(&id, episteme_root.as_path())?;
    Ok(LoadedEpistemeRegistryEntry {
        id,
        source_kind: LoadedEpistemeSourceKind::Local,
        episteme_root,
        source_root,
        url: None,
        resolved_revision: None,
        subdir: receipt_subdir(subdir),
    })
}

fn load_git_entry(
    id: String,
    project_root: &Path,
    url: String,
    subdir: &Path,
    mode: SyncMode,
) -> Result<LoadedEpistemeRegistryEntry, EpistemeRegistryError> {
    let spec = RepoSpec {
        id: format!("episteme-{id}"),
        local_path: None,
        remote_url: Some(url.clone()),
        revision: None,
        refresh: RepoRefreshPolicy::Fetch,
    };
    let materialized = resolve_repository_source(&spec, project_root, mode).map_err(|source| {
        EpistemeRegistryError::GitMaterialization {
            id: id.clone(),
            url: url.clone(),
            source,
        }
    })?;
    let episteme_root = apply_subdir(materialized.checkout_root.as_path(), subdir);
    ensure_manifest(&id, episteme_root.as_path())?;
    let resolved_revision = materialized
        .tracking_revision
        .or(materialized.mirror_revision);
    Ok(LoadedEpistemeRegistryEntry {
        id,
        source_kind: LoadedEpistemeSourceKind::Git,
        episteme_root,
        source_root: materialized.checkout_root,
        url: Some(url),
        resolved_revision,
        subdir: receipt_subdir(subdir),
    })
}

fn normalized_registry_id(raw: &str) -> Result<String, EpistemeRegistryError> {
    let id = raw.trim();
    if id.is_empty()
        || !id.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
    {
        return Err(EpistemeRegistryError::InvalidId(raw.to_string()));
    }
    Ok(id.to_string())
}

fn normalized_path(raw: Option<&Path>) -> Option<PathBuf> {
    let path = raw?;
    let value = path.to_string_lossy();
    (!value.trim().is_empty()).then(|| PathBuf::from(value.trim()))
}

fn normalized_url(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn normalized_subdir(id: &str, raw: &Path) -> Result<PathBuf, EpistemeRegistryError> {
    let value = raw.to_string_lossy();
    let trimmed = value.trim();
    let subdir = if trimmed.is_empty() {
        PathBuf::from(DEFAULT_SUBDIR)
    } else {
        PathBuf::from(trimmed)
    };
    if !is_safe_relative_path(subdir.as_path()) {
        return Err(EpistemeRegistryError::UnsafeSubdir {
            id: id.to_string(),
            subdir: trimmed.to_string(),
        });
    }
    Ok(subdir)
}

fn is_safe_relative_path(path: &Path) -> bool {
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::CurDir | Component::Normal(_)))
}

fn resolve_project_path(project_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

fn apply_subdir(source_root: &Path, subdir: &Path) -> PathBuf {
    if subdir == Path::new(DEFAULT_SUBDIR) {
        source_root.to_path_buf()
    } else {
        source_root.join(subdir)
    }
}

fn ensure_manifest(id: &str, episteme_root: &Path) -> Result<(), EpistemeRegistryError> {
    if episteme_root
        .join(ONTOLOGY_MANIFEST_RELATIVE_PATH)
        .is_file()
    {
        Ok(())
    } else {
        Err(EpistemeRegistryError::MissingManifest {
            id: id.to_string(),
            root: episteme_root.to_path_buf(),
        })
    }
}

fn receipt_subdir(subdir: &Path) -> String {
    subdir.to_string_lossy().replace('\\', "/")
}

#[derive(Debug, Deserialize)]
struct RegistryOntologyManifest {
    #[serde(default)]
    extends: Option<RegistryManifestExtends>,
    #[serde(default)]
    domains: Vec<RegistryManifestDomain>,
}

#[derive(Debug, Deserialize)]
struct RegistryManifestExtends {
    #[serde(default)]
    manifest: Option<String>,
    #[serde(default)]
    common_manifest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegistryManifestDomain {
    id: String,
}

struct RegistryManifestFacts {
    domain_ids: Vec<String>,
    extension_targets: Vec<String>,
}

fn read_registry_manifest_facts(
    loaded: &LoadedEpistemeRegistryEntry,
) -> Result<RegistryManifestFacts, EpistemeRegistryError> {
    let manifest_path = loaded.episteme_root.join(ONTOLOGY_MANIFEST_RELATIVE_PATH);
    let raw = fs::read_to_string(manifest_path.as_path()).map_err(|source| {
        EpistemeRegistryError::ManifestRead {
            id: loaded.id.clone(),
            path: manifest_path.clone(),
            source,
        }
    })?;
    let manifest = toml::from_str::<RegistryOntologyManifest>(&raw).map_err(|source| {
        EpistemeRegistryError::ManifestToml {
            id: loaded.id.clone(),
            path: manifest_path,
            source: Box::new(source),
        }
    })?;
    let domain_ids = normalized_manifest_domain_ids(loaded.id.as_str(), manifest.domains)?;
    let extension_targets = normalized_extension_targets(manifest.extends);
    Ok(RegistryManifestFacts {
        domain_ids,
        extension_targets,
    })
}

fn normalized_manifest_domain_ids(
    id: &str,
    domains: Vec<RegistryManifestDomain>,
) -> Result<Vec<String>, EpistemeRegistryError> {
    if domains.is_empty() {
        return Err(EpistemeRegistryError::MissingDomains { id: id.to_string() });
    }
    let mut seen = BTreeSet::new();
    let mut domain_ids = Vec::new();
    for domain in domains {
        let domain_id = domain.id.trim();
        if domain_id.is_empty() {
            return Err(EpistemeRegistryError::InvalidDomainId {
                id: id.to_string(),
                domain_id: domain.id,
            });
        }
        if seen.insert(domain_id.to_string()) {
            domain_ids.push(domain_id.to_string());
        }
    }
    Ok(domain_ids)
}

fn normalized_extension_targets(extends: Option<RegistryManifestExtends>) -> Vec<String> {
    let Some(extends) = extends else {
        return Vec::new();
    };
    [extends.manifest, extends.common_manifest]
        .into_iter()
        .flatten()
        .filter_map(|target| {
            let trimmed = target.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
