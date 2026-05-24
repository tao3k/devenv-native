use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::Serialize;

pub(super) const STRUCTURAL_FACTS_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_structural_facts.v1";
pub(super) const STRUCTURAL_FACTS_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_structural_facts_report.v1";

/// Validation policy used while compiling structural facts seed rows.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpistemeOntologyStructuralFactsValidationMode {
    /// Validate file presence and size without hashing source bytes.
    #[default]
    MetadataOnly,
    /// Validate file presence, size, and SHA-256.
    FullHash,
}

/// Request for compiling deterministic structural facts seed artifacts.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyStructuralFactsRequest {
    /// Episteme repository root.
    pub episteme_root: PathBuf,
    /// Raw corpus root resolved by the caller from episteme configuration.
    pub corpus_root: PathBuf,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Validation policy for source file checks.
    pub validation_mode: EpistemeOntologyStructuralFactsValidationMode,
}

impl EpistemeOntologyStructuralFactsRequest {
    /// Create a structural facts request.
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
            validation_mode: EpistemeOntologyStructuralFactsValidationMode::default(),
        }
    }

    /// Set the source validation policy.
    #[must_use]
    pub fn with_validation_mode(
        mut self,
        validation_mode: EpistemeOntologyStructuralFactsValidationMode,
    ) -> Self {
        self.validation_mode = validation_mode;
        self
    }
}

/// Config-driven request for compiling structural facts from an Episteme root.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyStructuralFactsConfiguredRequest {
    /// Episteme repository root.
    pub episteme_root: PathBuf,
    /// Optional raw corpus root override.
    pub corpus_root: Option<PathBuf>,
    /// Optional run artifact root override.
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Validation policy for source file checks.
    pub validation_mode: EpistemeOntologyStructuralFactsValidationMode,
}

impl EpistemeOntologyStructuralFactsConfiguredRequest {
    /// Create a config-driven structural facts request.
    #[must_use]
    pub fn new(episteme_root: impl Into<PathBuf>, run_id: impl Into<String>) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            corpus_root: None,
            run_root: None,
            run_id: run_id.into(),
            validation_mode: EpistemeOntologyStructuralFactsValidationMode::default(),
        }
    }

    /// Override the source corpus root instead of resolving it from config.
    #[must_use]
    pub fn with_corpus_root(mut self, corpus_root: impl Into<PathBuf>) -> Self {
        self.corpus_root = Some(corpus_root.into());
        self
    }

    /// Override the structural facts run root instead of using config defaults.
    #[must_use]
    pub fn with_run_root(mut self, run_root: impl Into<PathBuf>) -> Self {
        self.run_root = Some(run_root.into());
        self
    }

    /// Set the source validation policy.
    #[must_use]
    pub fn with_validation_mode(
        mut self,
        validation_mode: EpistemeOntologyStructuralFactsValidationMode,
    ) -> Self {
        self.validation_mode = validation_mode;
        self
    }
}

/// Source-contract summary represented in a structural facts seed.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsSourceContractSummary {
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

/// Document-level structural facts row.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsDocumentRow {
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
pub struct EpistemeOntologyStructuralFactsAnchorRow {
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
pub struct EpistemeOntologyStructuralFactsRelationRow {
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

/// Full structural facts snapshot.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsSnapshot {
    /// Snapshot schema version.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Source contract summaries.
    pub source_contracts: Vec<EpistemeOntologyStructuralFactsSourceContractSummary>,
    /// Document rows.
    pub documents: Vec<EpistemeOntologyStructuralFactsDocumentRow>,
    /// Structural anchor rows.
    pub anchors: Vec<EpistemeOntologyStructuralFactsAnchorRow>,
    /// Structural relation rows.
    pub relations: Vec<EpistemeOntologyStructuralFactsRelationRow>,
}

/// Report emitted after compiling structural facts seed artifacts.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReport {
    /// Report schema version.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Concrete run directory.
    pub run_dir: PathBuf,
    /// Full snapshot JSON path.
    pub structural_facts_json: PathBuf,
    /// Org ledger path.
    pub structural_facts_org: PathBuf,
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
    /// Structural RDF seed Turtle path.
    pub rdf_seed_ttl: PathBuf,
    /// Structural read-model object TSV path.
    pub read_model_objects_tsv: PathBuf,
    /// Structural read-model object JSON path.
    pub read_model_objects_json: PathBuf,
    /// Structural read-model object Parquet path.
    pub read_model_objects_parquet: PathBuf,
    /// Structural read-model relation TSV path.
    pub read_model_relations_tsv: PathBuf,
    /// Structural read-model relation JSON path.
    pub read_model_relations_json: PathBuf,
    /// Structural read-model relation Parquet path.
    pub read_model_relations_parquet: PathBuf,
    /// Structural read-model projection-state JSON path.
    pub read_model_projection_state_json: PathBuf,
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
    /// Number of structural read-model object rows emitted.
    pub read_model_object_count: usize,
    /// Number of structural read-model relation rows emitted.
    pub read_model_relation_count: usize,
    /// Number of structural read-model projection-state rows emitted.
    pub read_model_projection_state_count: usize,
    /// Whether structural read-model quality checks passed.
    pub read_model_quality_passed: bool,
    /// Structural read-model quality issues. A successful report has none.
    pub read_model_quality_issues: Vec<String>,
    /// File counts by extraction route.
    pub route_counts: BTreeMap<String, usize>,
    /// File counts by category.
    pub category_counts: BTreeMap<String, usize>,
    /// Non-promotion safety flags.
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralFactsSafetyFlags,
    /// Validation policy used for this run.
    pub validation_mode: EpistemeOntologyStructuralFactsValidationMode,
    /// Whether source bytes were hash-checked.
    pub full_hash_checked: bool,
    /// Number of detected hash drifts. A successful report always has zero.
    pub hash_drift_count: usize,
}

/// Safety flags preserved in structural facts reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsSafetyFlags {
    /// Whether OCR, ASR, or LLM extraction ran during this seed build.
    pub extraction_executed: bool,
    /// Whether this run mutated ontology source files.
    pub source_mutation_allowed: bool,
    /// Whether raw source rows are treated as ontology truth.
    pub ontology_truth: bool,
}

pub(super) struct StructuralFactsOutputPaths {
    pub(super) run_dir: PathBuf,
    pub(super) structural_facts_json: PathBuf,
    pub(super) structural_facts_org: PathBuf,
    pub(super) documents_tsv: PathBuf,
    pub(super) documents_json: PathBuf,
    pub(super) anchors_tsv: PathBuf,
    pub(super) anchors_json: PathBuf,
    pub(super) relations_tsv: PathBuf,
    pub(super) relations_json: PathBuf,
    pub(super) rdf_seed_ttl: PathBuf,
    pub(super) read_model_objects_tsv: PathBuf,
    pub(super) read_model_objects_json: PathBuf,
    pub(super) read_model_objects_parquet: PathBuf,
    pub(super) read_model_relations_tsv: PathBuf,
    pub(super) read_model_relations_json: PathBuf,
    pub(super) read_model_relations_parquet: PathBuf,
    pub(super) read_model_projection_state_json: PathBuf,
}

impl StructuralFactsOutputPaths {
    pub(super) fn new(run_root: &Path, run_id: &str) -> Self {
        let run_dir = run_root.join(run_id);
        Self {
            structural_facts_json: run_dir.join("structural_facts.json"),
            structural_facts_org: run_dir.join("structural_facts.org"),
            documents_tsv: run_dir.join("structural_facts_documents.tsv"),
            documents_json: run_dir.join("structural_facts_documents.json"),
            anchors_tsv: run_dir.join("structural_facts_anchors.tsv"),
            anchors_json: run_dir.join("structural_facts_anchors.json"),
            relations_tsv: run_dir.join("structural_facts_relations.tsv"),
            relations_json: run_dir.join("structural_facts_relations.json"),
            rdf_seed_ttl: run_dir.join("structural_facts_rdf_seed.ttl"),
            read_model_objects_tsv: run_dir.join("structural_facts_read_model_objects.tsv"),
            read_model_objects_json: run_dir.join("structural_facts_read_model_objects.json"),
            read_model_objects_parquet: run_dir.join("structural_facts_read_model_objects.parquet"),
            read_model_relations_tsv: run_dir.join("structural_facts_read_model_relations.tsv"),
            read_model_relations_json: run_dir.join("structural_facts_read_model_relations.json"),
            read_model_relations_parquet: run_dir
                .join("structural_facts_read_model_relations.parquet"),
            read_model_projection_state_json: run_dir
                .join("structural_facts_read_model_projection_state.json"),
            run_dir,
        }
    }
}
