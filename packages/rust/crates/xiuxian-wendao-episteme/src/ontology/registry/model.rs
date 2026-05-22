//! Typed `ontology/registry.json` DTOs for Episteme read-model admission.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Repository-relative ontology registry snapshot path.
pub const ONTOLOGY_REGISTRY_RELATIVE_PATH: &str = "ontology/registry.json";

/// Deterministic ontology registry snapshot emitted by the source compiler.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistrySnapshot {
    /// Snapshot schema version.
    pub schema_version: u32,
    /// Ontology registry identifier.
    pub ontology: String,
    /// API compatibility policy.
    pub compatibility: String,
    /// Source-contract boundary facts.
    pub source_contract: EpistemeOntologyRegistrySourceContract,
    /// Reference nouns that shape generated read-model vocabulary.
    #[serde(default)]
    pub reference_nouns: Vec<String>,
    /// Declared ontology domains.
    #[serde(default)]
    pub domains: Vec<EpistemeOntologyRegistryDomain>,
    /// Flattened read-only validation rules.
    #[serde(default)]
    pub rules: Vec<EpistemeOntologyRegistryRule>,
    /// Flattened policy artifacts.
    #[serde(default)]
    pub policies: Vec<EpistemeOntologyRegistryPolicy>,
    /// Dataset mapping contracts.
    #[serde(default)]
    pub dataset_mappings: Vec<EpistemeOntologyRegistryDatasetMapping>,
    /// RDF terms extracted by the compiler.
    #[serde(default)]
    pub rdf_terms: EpistemeOntologyRegistryRdfTerms,
    /// SDK/API read-model surface extracted by the compiler.
    #[serde(default)]
    pub api: EpistemeOntologyRegistryApiSurface,
}

/// Source-contract boundary facts inside the registry snapshot.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistrySourceContract {
    /// Manifest file relative to `ontology/`.
    pub manifest: String,
    /// Source artifact mode.
    pub artifact_mode: EpistemeOntologyRegistryArtifactMode,
    /// Whether source mutation is allowed.
    #[serde(default)]
    pub mutation_allowed: bool,
    /// Runtime compilation owner.
    pub runtime_compilation_owner: String,
    /// SDK generation owner.
    pub sdk_generation_owner: String,
    /// API-surface file relative to `ontology/`.
    #[serde(default)]
    pub api_surface: Option<String>,
}

/// One registry domain.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryDomain {
    /// Stable domain id.
    pub id: String,
    /// Optional ordering category.
    #[serde(default)]
    pub category: Option<EpistemeOntologyRegistryCategory>,
    /// Optional ontology layer.
    #[serde(default)]
    pub layer: Option<String>,
    /// Human-readable domain name.
    pub name: String,
    /// RDF source files relative to `ontology/`.
    #[serde(default)]
    pub rdf_files: Vec<String>,
    /// SQL validation rule files relative to `ontology/`.
    #[serde(default)]
    pub rules: Vec<String>,
    /// Policy files relative to `ontology/`.
    #[serde(default)]
    pub policies: Vec<String>,
    /// Dataset mapping files relative to `ontology/`.
    #[serde(default)]
    pub dataset_mappings: Vec<String>,
}

/// Flattened registry rule entry.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryRule {
    /// Domain owning the rule.
    pub domain: String,
    /// Rule kind.
    pub kind: EpistemeOntologyRegistryKind,
    /// Rule path relative to `ontology/`.
    pub path: String,
}

/// Flattened registry policy entry.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryPolicy {
    /// Domain owning the policy.
    pub domain: String,
    /// Policy kind.
    pub kind: EpistemeOntologyRegistryKind,
    /// Policy path relative to `ontology/`.
    pub path: String,
}

/// Dataset mapping entry.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryDatasetMapping {
    /// Domain owning the mapping.
    pub domain: String,
    /// Mapping kind.
    pub kind: EpistemeOntologyRegistryKind,
    /// Stable mapping id.
    pub mapping_id: String,
    /// Mapping TOML path relative to `ontology/`.
    pub path: String,
    /// Mapping ledger Org path relative to `ontology/`.
    pub ledger_org: String,
    /// Materialization SQL paths relative to `ontology/`.
    #[serde(default)]
    pub materialization: BTreeMap<String, String>,
    /// Raw table names consumed by the mapping.
    #[serde(default)]
    pub raw_tables: Vec<String>,
    /// Validation rule paths relative to `ontology/`.
    #[serde(default)]
    pub validation_rules: Vec<String>,
}

/// RDF terms extracted from source RDF files.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryRdfTerms {
    /// RDF classes.
    #[serde(default)]
    pub classes: Vec<EpistemeOntologyRegistryRdfClassTerm>,
    /// RDF object properties.
    #[serde(default)]
    pub object_properties: Vec<EpistemeOntologyRegistryObjectPropertyTerm>,
}

/// One RDF class term.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryRdfClassTerm {
    /// Domain owning the RDF term.
    pub domain: String,
    /// RDF IRI.
    pub iri: String,
    /// Human-readable label.
    pub label: String,
    /// API candidate name.
    pub api_candidate: String,
    /// RDF source file relative to `ontology/`.
    pub source_file: String,
}

/// One RDF object-property term.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryObjectPropertyTerm {
    /// Domain owning the RDF term.
    pub domain: String,
    /// RDF IRI.
    pub iri: String,
    /// Human-readable label.
    pub label: String,
    /// API candidate name.
    pub api_candidate: String,
    /// Optional source-class IRI.
    #[serde(default)]
    pub from_iri: Option<String>,
    /// Optional target-class IRI.
    #[serde(default)]
    pub to_iri: Option<String>,
    /// RDF source file relative to `ontology/`.
    pub source_file: String,
}

/// API/read-model surface extracted from the ontology registry snapshot.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryApiSurface {
    /// Object types.
    #[serde(default, rename = "object_types")]
    pub objects: Vec<EpistemeOntologyRegistryObjectType>,
    /// Link types.
    #[serde(default, rename = "link_types")]
    pub links: Vec<EpistemeOntologyRegistryLinkType>,
    /// Action types.
    #[serde(default, rename = "action_types")]
    pub actions: Vec<EpistemeOntologyRegistryActionType>,
    /// Query types.
    #[serde(default, rename = "query_types")]
    pub queries: Vec<EpistemeOntologyRegistryQueryType>,
    /// Interface types.
    #[serde(default, rename = "interface_types")]
    pub interfaces: Vec<EpistemeOntologyRegistryInterfaceType>,
}

/// API object type.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryObjectType {
    /// Object API name.
    pub api_name: String,
    /// Domain owning the object type.
    pub domain: String,
    /// RDF class IRI.
    pub rdf_class: String,
    /// Primary key fields.
    #[serde(default)]
    pub primary_key: Vec<String>,
    /// Display-name property.
    pub display_name_property: String,
    /// Implemented interfaces.
    #[serde(default)]
    pub interfaces: Vec<String>,
}

/// API link type.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryLinkType {
    /// Link API name.
    pub api_name: String,
    /// Domain owning the link type.
    pub domain: String,
    /// Source object type.
    pub from_object_type: EpistemeOntologyRegistryObjectTypeRef,
    /// Target object type.
    pub to_object_type: EpistemeOntologyRegistryObjectTypeRef,
    /// RDF property IRI.
    pub rdf_property: String,
    /// Cardinality label.
    pub cardinality: String,
}

/// API action type.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryActionType {
    /// Action API name.
    pub api_name: String,
    /// Domain owning the action type.
    pub domain: String,
    /// Object types affected by the action.
    #[serde(default)]
    pub affected_object_types: Vec<String>,
    /// Whether the action requires evidence.
    #[serde(default)]
    pub requires_evidence: bool,
    /// Validation rule paths relative to `ontology/`.
    #[serde(default)]
    pub validation_rules: Vec<String>,
}

/// API query type.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryQueryType {
    /// Query API name.
    pub api_name: String,
    /// Domain owning the query type.
    pub domain: String,
    /// Query parameters.
    #[serde(default)]
    pub parameters: Vec<String>,
    /// Returned object type.
    pub returns: String,
}

/// API interface type.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryInterfaceType {
    /// Interface API name.
    pub api_name: String,
    /// Object types implementing the interface.
    #[serde(default)]
    pub implemented_by: Vec<String>,
}

/// Read-model input accepted by Rust after snapshot admission.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyRegistryReadModelInput {
    /// Admitted registry snapshot.
    pub snapshot: EpistemeOntologyRegistrySnapshot,
    /// Deterministic admission/read-model counts.
    pub report: EpistemeOntologyRegistrySnapshotReport,
}

/// Admission/read-model counts for an ontology registry snapshot.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyRegistrySnapshotReport {
    /// Number of declared domains.
    pub domains: usize,
    /// Number of RDF files declared by domains.
    pub rdf_files: usize,
    /// Number of flattened rule entries.
    pub rules: usize,
    /// Number of flattened policy entries.
    pub policies: usize,
    /// Number of dataset mapping entries.
    pub dataset_mappings: usize,
    /// Number of RDF class terms.
    pub rdf_classes: usize,
    /// Number of RDF object-property terms.
    pub rdf_object_properties: usize,
    /// Number of API object types.
    pub api_objects: usize,
    /// Number of API link types.
    pub api_links: usize,
    /// Number of API action types.
    pub api_actions: usize,
    /// Number of API query types.
    pub api_queries: usize,
    /// Number of API interface types.
    pub api_interfaces: usize,
    /// Number of reference nouns.
    pub reference_nouns: usize,
}

/// Source-contract artifact mode token in an ontology registry snapshot.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct EpistemeOntologyRegistryArtifactMode(String);

impl EpistemeOntologyRegistryArtifactMode {
    /// Return the artifact-mode token.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Ordering category token in an ontology registry snapshot.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct EpistemeOntologyRegistryCategory(String);

impl EpistemeOntologyRegistryCategory {
    /// Return the category token.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Registry rule, policy, or mapping kind token.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct EpistemeOntologyRegistryKind(String);

impl EpistemeOntologyRegistryKind {
    /// Return the kind token.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// API object-type reference token used by link definitions.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct EpistemeOntologyRegistryObjectTypeRef(String);

impl EpistemeOntologyRegistryObjectTypeRef {
    /// Return the object-type reference token.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
