use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::Serialize;

pub(super) const STRUCTURAL_IDF_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_structural_idf.v1";
pub(super) const STRUCTURAL_IDF_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_structural_idf_report.v1";

/// Validation policy used while compiling structural IDF seed rows.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpistemeOntologyStructuralIdfValidationMode {
    /// Validate file presence and size without hashing source bytes.
    #[default]
    MetadataOnly,
    /// Validate file presence, size, and SHA-256.
    FullHash,
}

/// Request for compiling deterministic structural IDF seed artifacts.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyStructuralIdfRequest {
    /// Episteme repository root.
    pub episteme_root: PathBuf,
    /// Raw corpus root resolved by the caller from episteme configuration.
    pub corpus_root: PathBuf,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Validation policy for source file checks.
    pub validation_mode: EpistemeOntologyStructuralIdfValidationMode,
}

impl EpistemeOntologyStructuralIdfRequest {
    /// Create a structural IDF request.
    #[must_use]
    pub fn new(
        episteme_root: impl Into<PathBuf>,
        corpus_root: impl Into<PathBuf>,
        run_id: impl Into<String>,
    ) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            corpus_root: corpus_root.into(),
            run_id: run_id.into(),
            validation_mode: EpistemeOntologyStructuralIdfValidationMode::default(),
        }
    }

    /// Set the source validation policy.
    #[must_use]
    pub fn with_validation_mode(
        mut self,
        validation_mode: EpistemeOntologyStructuralIdfValidationMode,
    ) -> Self {
        self.validation_mode = validation_mode;
        self
    }
}

/// Source-contract summary represented in a structural IDF seed.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfSourceContractSummary {
    /// Ontology domain id.
    pub domain_id: String,
    /// Source contract id from the source manifest.
    pub source_contract_id: String,
    /// Source manifest path relative to the Episteme repository.
    pub source_manifest_path: String,
    /// Files TSV path relative to the Episteme repository.
    pub files_tsv_path: String,
    /// Primary source language.
    pub primary_language: String,
    /// Number of files represented by this source contract.
    pub file_count: usize,
}

/// Document-level structural IDF row.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfDocumentRow {
    /// Stable document seed id.
    pub document_id: String,
    /// Source file id from `files.tsv`.
    pub file_id: String,
    /// Ontology domain id.
    pub domain_id: String,
    /// Source contract id.
    pub source_contract_id: String,
    /// Source manifest path relative to the Episteme repository.
    pub source_manifest_path: String,
    /// Source path relative to the corpus root.
    pub relative_path: String,
    /// Lowercase source extension.
    pub extension: String,
    /// Expected byte size from `files.tsv`.
    pub byte_size: u64,
    /// Expected SHA-256 from `files.tsv`.
    pub sha256: String,
    /// Source category.
    pub category: String,
    /// Source language.
    pub language: String,
    /// Intended extraction route.
    pub extraction_route: String,
    /// Whether the source path existed at compile time.
    pub source_exists: bool,
    /// Whether source byte size matched `files.tsv`.
    pub byte_size_matches: bool,
    /// SHA-256 match when full-hash validation is enabled.
    pub sha256_matches: Option<bool>,
    /// Raw rows are not ontology truth in this seed.
    pub ontology_truth: bool,
    /// Deterministic row status.
    pub status: String,
}

/// Structural anchor row emitted for corpus roots, path segments, and files.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfAnchorRow {
    /// Stable anchor id.
    pub anchor_id: String,
    /// Anchor kind.
    pub anchor_kind: String,
    /// Owning document id for document-root anchors.
    pub document_id: String,
    /// Source file id for document-root anchors.
    pub file_id: String,
    /// Parent anchor id when known.
    pub parent_anchor_id: String,
    /// Ontology domain id.
    pub domain_id: String,
    /// Source contract id.
    pub source_contract_id: String,
    /// Source-relative path represented by this anchor.
    pub relative_path: String,
    /// Tree depth under the source-contract root.
    pub path_depth: usize,
    /// Stable per-run reading/order key.
    pub order_key: usize,
    /// Source language.
    pub language: String,
    /// Intended extraction route.
    pub extraction_route: String,
    /// Source content hash for document-root anchors.
    pub source_content_hash: String,
    /// Raw anchors are not ontology truth in this seed.
    pub ontology_truth: bool,
    /// Deterministic row status.
    pub status: String,
}

/// Structural relation row linking anchors and documents.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfRelationRow {
    /// Stable relation id.
    pub relation_id: String,
    /// Relation kind.
    pub relation_kind: String,
    /// Source anchor id.
    pub source_anchor_id: String,
    /// Target anchor id.
    pub target_anchor_id: String,
    /// Owning document id when applicable.
    pub document_id: String,
    /// Source file id when applicable.
    pub file_id: String,
    /// Ontology domain id.
    pub domain_id: String,
    /// Source contract id.
    pub source_contract_id: String,
    /// Source path relative to the corpus root.
    pub evidence_path: String,
    /// Stable per-run relation order.
    pub order_key: usize,
    /// Raw relations are not ontology truth in this seed.
    pub ontology_truth: bool,
    /// Deterministic row status.
    pub status: String,
}

/// Full structural IDF snapshot.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfSnapshot {
    /// Snapshot schema version.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Source contract summaries.
    pub source_contracts: Vec<EpistemeOntologyStructuralIdfSourceContractSummary>,
    /// Document rows.
    pub documents: Vec<EpistemeOntologyStructuralIdfDocumentRow>,
    /// Structural anchor rows.
    pub anchors: Vec<EpistemeOntologyStructuralIdfAnchorRow>,
    /// Structural relation rows.
    pub relations: Vec<EpistemeOntologyStructuralIdfRelationRow>,
}

/// Report emitted after compiling structural IDF seed artifacts.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfReport {
    /// Report schema version.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Concrete run directory.
    pub run_dir: PathBuf,
    /// Full snapshot JSON path.
    pub structural_idf_json: PathBuf,
    /// Org ledger path.
    pub structural_idf_org: PathBuf,
    /// Documents TSV path.
    pub documents_tsv: PathBuf,
    /// Documents JSON path.
    pub documents_json: PathBuf,
    /// Anchors TSV path.
    pub anchors_tsv: PathBuf,
    /// Anchors JSON path.
    pub anchors_json: PathBuf,
    /// Relations TSV path.
    pub relations_tsv: PathBuf,
    /// Relations JSON path.
    pub relations_json: PathBuf,
    /// Number of ontology domains carrying source manifests.
    pub domain_count: usize,
    /// Number of source manifests compiled.
    pub source_manifest_count: usize,
    /// Number of source file rows compiled.
    pub file_count: usize,
    /// Number of document rows emitted.
    pub document_count: usize,
    /// Number of anchor rows emitted.
    pub anchor_count: usize,
    /// Number of structural relation rows emitted.
    pub relation_count: usize,
    /// File counts by extraction route.
    pub route_counts: BTreeMap<String, usize>,
    /// File counts by category.
    pub category_counts: BTreeMap<String, usize>,
    /// Non-promotion safety flags.
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralIdfSafetyFlags,
    /// Validation policy used for this run.
    pub validation_mode: EpistemeOntologyStructuralIdfValidationMode,
    /// Whether source bytes were hash-checked.
    pub full_hash_checked: bool,
    /// Number of detected hash drifts. A successful report always has zero.
    pub hash_drift_count: usize,
}

/// Safety flags preserved in structural IDF reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfSafetyFlags {
    /// Whether OCR, ASR, or LLM extraction ran during this seed build.
    pub extraction_executed: bool,
    /// Whether this run mutated ontology source files.
    pub source_mutation_allowed: bool,
    /// Whether raw source rows are treated as ontology truth.
    pub ontology_truth: bool,
}

pub(super) struct StructuralIdfOutputPaths {
    pub(super) run_dir: PathBuf,
    pub(super) structural_idf_json: PathBuf,
    pub(super) structural_idf_org: PathBuf,
    pub(super) documents_tsv: PathBuf,
    pub(super) documents_json: PathBuf,
    pub(super) anchors_tsv: PathBuf,
    pub(super) anchors_json: PathBuf,
    pub(super) relations_tsv: PathBuf,
    pub(super) relations_json: PathBuf,
}

impl StructuralIdfOutputPaths {
    pub(super) fn new(run_root: &Path, run_id: &str) -> Self {
        let run_dir = run_root.join(run_id);
        Self {
            structural_idf_json: run_dir.join("structural_idf.json"),
            structural_idf_org: run_dir.join("structural_idf.org"),
            documents_tsv: run_dir.join("structural_idf_documents.tsv"),
            documents_json: run_dir.join("structural_idf_documents.json"),
            anchors_tsv: run_dir.join("structural_idf_anchors.tsv"),
            anchors_json: run_dir.join("structural_idf_anchors.json"),
            relations_tsv: run_dir.join("structural_idf_relations.tsv"),
            relations_json: run_dir.join("structural_idf_relations.json"),
            run_dir,
        }
    }
}
