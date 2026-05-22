//! `ontology/manifest.toml` admission and conservative source validation.

use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

/// Repository-relative ontology manifest path used by Episteme repositories.
pub const ONTOLOGY_MANIFEST_RELATIVE_PATH: &str = "ontology/manifest.toml";
const SOURCE_CONTRACT_ARTIFACT_MODE: &str = "source_contract";
const PRIVATE_SOURCE_CONTRACT_ARTIFACT_MODE: &str = "private_source_contract";
const EPISTEME_DOMAIN_SCHEME: &str = "episteme://";
const PRIVATE_DOMAIN_PREFIX: &str = "episteme://private/";

/// Top-level source ontology manifest.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct EpistemeOntologyManifest {
    /// Manifest schema version.
    pub schema_version: u32,
    /// Stable manifest name.
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional primary human language for private or vertical ontology packs.
    #[serde(default)]
    pub primary_language: Option<String>,
    /// Optional top-level artifact mode for private extension manifests.
    #[serde(default)]
    pub artifact_mode: Option<EpistemeOntologyArtifactMode>,
    /// Optional top-level mutation flag for private extension manifests.
    #[serde(default)]
    pub mutation_allowed: Option<bool>,
    /// Ownership and mutation boundaries for the ontology source contract.
    pub boundaries: EpistemeOntologyBoundaries,
    /// Optional private extension target declaration.
    #[serde(default)]
    pub extends: Option<EpistemeOntologyExtends>,
    /// Declared ontology domains.
    #[serde(default)]
    pub domains: Vec<EpistemeOntologyDomain>,
    /// Optional extension contract for downstream ontology packs.
    #[serde(default)]
    pub extension_contract: Option<EpistemeOntologyExtensionContract>,
    /// Optional SDK-facing API-surface contract.
    #[serde(default)]
    pub api_surface: Option<EpistemeOntologyApiSurface>,
}

/// Ownership and mutation boundaries declared by the source ontology manifest.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct EpistemeOntologyBoundaries {
    /// Source owner label.
    pub owner: String,
    /// Artifact mode; common manifests use `source_contract`.
    #[serde(default = "default_source_contract_artifact_mode")]
    pub artifact_mode: EpistemeOntologyArtifactMode,
    /// Runtime compilation owner.
    pub runtime_compilation_owner: String,
    /// SQL execution owner.
    #[serde(default)]
    pub sql_execution_owner: Option<String>,
    /// Whether direct source mutation is allowed.
    #[serde(default)]
    pub mutation_allowed: bool,
    /// Common-domain owner for private extension manifests.
    #[serde(default)]
    pub common_domain_owner: Option<String>,
    /// Raw corpus ownership policy for private extension manifests.
    #[serde(default)]
    pub raw_corpus_policy: Option<String>,
    /// Whether raw cache rows may be promoted directly to RDF.
    #[serde(default)]
    pub raw_to_rdf_promotion_allowed: Option<bool>,
}

/// Typed ontology artifact-mode value.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct EpistemeOntologyArtifactMode(String);

impl EpistemeOntologyArtifactMode {
    /// Return the artifact mode as declared by the source ontology manifest.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

fn default_source_contract_artifact_mode() -> EpistemeOntologyArtifactMode {
    EpistemeOntologyArtifactMode(SOURCE_CONTRACT_ARTIFACT_MODE.to_string())
}

/// Private extension target declaration.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct EpistemeOntologyExtends {
    /// Common manifest id being extended.
    pub common_manifest: String,
    /// Common ontology IRI being extended.
    pub common_ontology_iri: String,
}

/// One ontology domain declaration.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct EpistemeOntologyDomain {
    /// Stable domain id, expected to use the `episteme://` scheme.
    pub id: String,
    /// Optional ordering category.
    #[serde(default)]
    pub category: Option<EpistemeOntologyDomainCategory>,
    /// Optional ontology layer label.
    #[serde(default)]
    pub layer: Option<String>,
    /// Human-readable domain name.
    #[serde(default)]
    pub name: String,
    /// Primary Chinese domain label for Chinese-first private packs.
    #[serde(default)]
    pub name_zh: Option<String>,
    /// English domain label for bilingual private packs.
    #[serde(default)]
    pub name_en: Option<String>,
    /// RDF source files relative to `ontology/`.
    #[serde(default)]
    pub rdf_files: Vec<String>,
    /// SQL validation rules relative to `ontology/`.
    #[serde(default)]
    pub rules: Vec<String>,
    /// Policy documents relative to `ontology/`.
    #[serde(default)]
    pub policies: Vec<String>,
    /// Dataset mapping contracts relative to `ontology/`.
    #[serde(default)]
    pub dataset_mappings: Vec<String>,
    /// Source manifests relative to `ontology/`.
    #[serde(default)]
    pub source_manifests: Vec<String>,
    /// Mapping ledgers relative to `ontology/`.
    #[serde(default)]
    pub mapping_ledgers: Vec<String>,
    /// Review ledgers relative to `ontology/`.
    #[serde(default)]
    pub review_ledgers: Vec<String>,
}

/// Typed ontology domain category value.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct EpistemeOntologyDomainCategory(String);

impl EpistemeOntologyDomainCategory {
    /// Return the domain category as declared by the source ontology manifest.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Extension contract declaration for downstream ontology packs.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct EpistemeOntologyExtensionContract {
    /// Example contract path relative to `ontology/`.
    pub example: String,
    /// Field path that declares the extended ontology domain.
    pub extends_field: String,
    /// Field path that declares the extension namespace.
    pub namespace_field: String,
    /// Allowed extension sections.
    #[serde(default)]
    pub allowed_sections: Vec<String>,
    /// Rule-mount policy identifier.
    pub rule_mount: String,
}

/// SDK-facing API-surface declaration.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct EpistemeOntologyApiSurface {
    /// API-surface TOML file relative to `ontology/`.
    pub file: String,
    /// Compatibility policy identifier.
    pub compatibility: String,
    /// Reference nouns that shape generated API vocabulary.
    #[serde(default)]
    pub reference_nouns: Vec<String>,
}

/// Validation summary for an ontology source contract.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyContractReport {
    /// Number of declared ontology domains.
    pub domain_count: usize,
    /// Number of declared RDF files.
    pub rdf_file_count: usize,
    /// Number of declared SQL rule files.
    pub rule_count: usize,
    /// Number of declared policy files.
    pub policy_count: usize,
    /// Number of declared dataset mapping files.
    pub dataset_mapping_count: usize,
    /// Whether an API-surface contract is declared.
    pub api_surface_declared: bool,
}

/// Error returned by Rust-owned ontology contract admission.
#[derive(Debug, Error)]
pub enum EpistemeOntologyError {
    /// A file or directory could not be accessed.
    #[error("failed to access `{path}`: {source}")]
    Io {
        /// Path that failed.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// The ontology manifest TOML is malformed.
    #[error("failed to parse ontology manifest `{path}`: {source}")]
    ManifestToml {
        /// Manifest path.
        path: PathBuf,
        /// Underlying TOML error.
        #[source]
        source: toml::de::Error,
    },
    /// The ontology source contract is invalid.
    #[error("ontology source contract is invalid: {0}")]
    InvalidContract(String),
}

/// Return the absolute ontology manifest path for an Episteme repository root.
#[must_use]
pub fn ontology_manifest_path(episteme_root: impl AsRef<Path>) -> PathBuf {
    episteme_root.as_ref().join(ONTOLOGY_MANIFEST_RELATIVE_PATH)
}

/// Read `ontology/manifest.toml` from an Episteme repository.
///
/// # Errors
///
/// Returns an error when the manifest cannot be read or parsed.
pub fn read_ontology_manifest(
    episteme_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyManifest, EpistemeOntologyError> {
    let path = ontology_manifest_path(episteme_root);
    let raw = read_to_string(&path)?;
    toml::from_str::<EpistemeOntologyManifest>(&raw)
        .map_err(|source| EpistemeOntologyError::ManifestToml { path, source })
}

/// Validate the ontology manifest and all declared source artifacts.
///
/// # Errors
///
/// Returns an error when boundaries are unsafe, domain ids are invalid or
/// duplicated, or any declared artifact path escapes `ontology/` or is missing.
pub fn validate_ontology_contract(
    episteme_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyContractReport, EpistemeOntologyError> {
    let episteme_root = episteme_root.as_ref();
    let manifest = read_ontology_manifest(episteme_root)?;
    validate_manifest_shape(&manifest)?;
    validate_manifest_artifacts(episteme_root, &manifest)
}

fn validate_manifest_shape(
    manifest: &EpistemeOntologyManifest,
) -> Result<(), EpistemeOntologyError> {
    if manifest.schema_version != 1 {
        return Err(invalid_contract(format!(
            "unsupported ontology manifest schema_version: {}",
            manifest.schema_version
        )));
    }
    if manifest.name.trim().is_empty() {
        return Err(invalid_contract("manifest name must not be blank"));
    }
    if manifest.boundaries.owner.trim().is_empty() {
        return Err(invalid_contract("boundaries.owner must not be blank"));
    }
    let artifact_mode = effective_artifact_mode(manifest);
    if artifact_mode != SOURCE_CONTRACT_ARTIFACT_MODE
        && artifact_mode != PRIVATE_SOURCE_CONTRACT_ARTIFACT_MODE
    {
        return Err(invalid_contract(format!(
            "artifact mode must be `{SOURCE_CONTRACT_ARTIFACT_MODE}` or `{PRIVATE_SOURCE_CONTRACT_ARTIFACT_MODE}`"
        )));
    }
    if manifest
        .boundaries
        .runtime_compilation_owner
        .trim()
        .is_empty()
    {
        return Err(invalid_contract(
            "boundaries.runtime_compilation_owner must not be blank",
        ));
    }
    if effective_mutation_allowed(manifest) {
        return Err(invalid_contract(
            "boundaries.mutation_allowed must be false for source ontology admission",
        ));
    }
    if manifest
        .boundaries
        .raw_to_rdf_promotion_allowed
        .unwrap_or(false)
    {
        return Err(invalid_contract(
            "boundaries.raw_to_rdf_promotion_allowed must be false for source ontology admission",
        ));
    }
    if manifest.domains.is_empty() {
        return Err(invalid_contract(
            "manifest must declare at least one domain",
        ));
    }
    validate_private_extension_shape(manifest, artifact_mode)?;
    validate_domain_ids(&manifest.domains, artifact_mode)
}

fn effective_artifact_mode(manifest: &EpistemeOntologyManifest) -> &str {
    manifest
        .artifact_mode
        .as_ref()
        .unwrap_or(&manifest.boundaries.artifact_mode)
        .as_str()
}

fn effective_mutation_allowed(manifest: &EpistemeOntologyManifest) -> bool {
    manifest
        .mutation_allowed
        .unwrap_or(manifest.boundaries.mutation_allowed)
}

fn validate_private_extension_shape(
    manifest: &EpistemeOntologyManifest,
    artifact_mode: &str,
) -> Result<(), EpistemeOntologyError> {
    if artifact_mode != PRIVATE_SOURCE_CONTRACT_ARTIFACT_MODE {
        return Ok(());
    }
    let Some(extends) = &manifest.extends else {
        return Err(invalid_contract(
            "private source contracts must declare [extends]",
        ));
    };
    if !extends.common_manifest.starts_with(EPISTEME_DOMAIN_SCHEME) {
        return Err(invalid_contract(format!(
            "extends.common_manifest must use {EPISTEME_DOMAIN_SCHEME} scheme: {}",
            extends.common_manifest
        )));
    }
    if extends.common_ontology_iri.trim().is_empty() {
        return Err(invalid_contract(
            "extends.common_ontology_iri must not be blank",
        ));
    }
    if manifest
        .primary_language
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(invalid_contract(
            "private source contracts must declare primary_language",
        ));
    }
    Ok(())
}

fn validate_domain_ids(
    domains: &[EpistemeOntologyDomain],
    artifact_mode: &str,
) -> Result<(), EpistemeOntologyError> {
    let mut seen = BTreeSet::new();
    for domain in domains {
        if domain.id.trim().is_empty() {
            return Err(invalid_contract("domain id must not be blank"));
        }
        validate_domain_scheme(&domain.id, artifact_mode)?;
        if !seen.insert(domain.id.as_str()) {
            return Err(invalid_contract(format!(
                "duplicate ontology domain id: {}",
                domain.id
            )));
        }
        if domain_display_name(domain).is_empty() {
            return Err(invalid_contract(format!(
                "domain {} name must not be blank",
                domain.id
            )));
        }
    }
    Ok(())
}

fn validate_domain_scheme(
    domain_id: &str,
    artifact_mode: &str,
) -> Result<(), EpistemeOntologyError> {
    match artifact_mode {
        SOURCE_CONTRACT_ARTIFACT_MODE if domain_id.starts_with(EPISTEME_DOMAIN_SCHEME) => Ok(()),
        SOURCE_CONTRACT_ARTIFACT_MODE => Err(invalid_contract(format!(
            "domain id must use {EPISTEME_DOMAIN_SCHEME} scheme: {domain_id}"
        ))),
        PRIVATE_SOURCE_CONTRACT_ARTIFACT_MODE if domain_id.starts_with(PRIVATE_DOMAIN_PREFIX) => {
            Ok(())
        }
        PRIVATE_SOURCE_CONTRACT_ARTIFACT_MODE => Err(invalid_contract(format!(
            "private source-contract domain id must use {PRIVATE_DOMAIN_PREFIX} prefix: {domain_id}"
        ))),
        _ => Err(invalid_contract(format!(
            "unsupported artifact mode for domain id validation: {artifact_mode}"
        ))),
    }
}

fn domain_display_name(domain: &EpistemeOntologyDomain) -> &str {
    if !domain.name.trim().is_empty() {
        return domain.name.trim();
    }
    domain
        .name_zh
        .as_deref()
        .or(domain.name_en.as_deref())
        .map(str::trim)
        .unwrap_or_default()
}

fn validate_manifest_artifacts(
    episteme_root: &Path,
    manifest: &EpistemeOntologyManifest,
) -> Result<EpistemeOntologyContractReport, EpistemeOntologyError> {
    let mut report = EpistemeOntologyContractReport {
        domain_count: manifest.domains.len(),
        rdf_file_count: 0,
        rule_count: 0,
        policy_count: 0,
        dataset_mapping_count: 0,
        api_surface_declared: manifest.api_surface.is_some(),
    };

    for domain in &manifest.domains {
        validate_domain_artifacts(episteme_root, domain, &mut report)?;
    }
    if let Some(extension) = &manifest.extension_contract {
        ensure_existing_ontology_artifact(episteme_root, &extension.example, "extension example")?;
    }
    if let Some(api_surface) = &manifest.api_surface {
        ensure_existing_ontology_artifact(episteme_root, &api_surface.file, "api_surface.file")?;
    }

    Ok(report)
}

fn validate_domain_artifacts(
    episteme_root: &Path,
    domain: &EpistemeOntologyDomain,
    report: &mut EpistemeOntologyContractReport,
) -> Result<(), EpistemeOntologyError> {
    validate_artifact_list(episteme_root, &domain.rdf_files, "rdf_files")?;
    validate_artifact_list(episteme_root, &domain.rules, "rules")?;
    validate_artifact_list(episteme_root, &domain.policies, "policies")?;
    validate_artifact_list(episteme_root, &domain.dataset_mappings, "dataset_mappings")?;
    validate_artifact_list(episteme_root, &domain.source_manifests, "source_manifests")?;
    validate_artifact_list(episteme_root, &domain.mapping_ledgers, "mapping_ledgers")?;
    validate_artifact_list(episteme_root, &domain.review_ledgers, "review_ledgers")?;

    report.rdf_file_count += domain.rdf_files.len();
    report.rule_count += domain.rules.len();
    report.policy_count += domain.policies.len();
    report.dataset_mapping_count += domain.dataset_mappings.len();
    Ok(())
}

fn validate_artifact_list(
    episteme_root: &Path,
    paths: &[String],
    field: &str,
) -> Result<(), EpistemeOntologyError> {
    for path in paths {
        ensure_existing_ontology_artifact(episteme_root, path, field)?;
    }
    Ok(())
}

fn ensure_existing_ontology_artifact(
    episteme_root: &Path,
    raw: &str,
    field: &str,
) -> Result<(), EpistemeOntologyError> {
    let path = resolve_ontology_artifact_path(episteme_root, raw, field)?;
    if !path.is_file() {
        return Err(invalid_contract(format!(
            "`{field}` entry does not exist or is not a file: {raw}"
        )));
    }
    Ok(())
}

fn resolve_ontology_artifact_path(
    episteme_root: &Path,
    raw: &str,
    field: &str,
) -> Result<PathBuf, EpistemeOntologyError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(invalid_contract(format!(
            "`{field}` entries must not be blank"
        )));
    }
    let path = Path::new(trimmed);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(invalid_contract(format!(
            "`{field}` entries must be safe paths relative to ontology/: {trimmed}"
        )));
    }
    Ok(episteme_root.join("ontology").join(path))
}

fn read_to_string(path: &Path) -> Result<String, EpistemeOntologyError> {
    fs::read_to_string(path).map_err(|source| EpistemeOntologyError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn invalid_contract(message: impl Into<String>) -> EpistemeOntologyError {
    EpistemeOntologyError::InvalidContract(message.into())
}
