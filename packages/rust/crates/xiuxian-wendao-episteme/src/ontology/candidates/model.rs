use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xiuxian_wendao_parsers::EpistemeFileRow;

pub(super) const CANDIDATE_GENERATION_SCHEMA: &str =
    "xiuxian_wendao.episteme_ontology_candidate_generation.v1";
pub(super) const OBJECTS_TSV: &str = "candidate_objects.tsv";
pub(super) const RELATIONS_TSV: &str = "candidate_relations.tsv";
pub(super) const EVIDENCE_TSV: &str = "candidate_evidence.tsv";
pub(super) const REVIEW_LEDGER_ORG: &str = "review_ledger.org";
pub(super) const RECEIPT_JSON: &str = "receipt.json";
pub(super) const PROMOTION_STATUS: &str = "blocked_pending_review";
pub(super) const REVIEW_STATUS: &str = "review_required";
pub(super) const STATUS_CANDIDATE: &str = "candidate";
pub(super) const OBJECT_TERM_KIND: &str = "ontology_candidate.object_term";
pub(super) const RELATION_TERM_KIND: &str = "ontology_candidate.relation_term";
pub(super) const SOURCE_ARTIFACT_KIND: &str = "ontology_candidate.source_artifact";
pub(super) const EXTRACTION_EVIDENCE_KIND: &str = "ontology_candidate.extraction_evidence";
pub(super) const SUGGESTED_OBJECT_TYPE_RELATION: &str =
    "ontology_candidate.source_artifact.suggested_object_type";
pub(super) const EVIDENCE_SUPPORTS_SOURCE_RELATION: &str =
    "ontology_candidate.extraction_evidence.supports_source_artifact";
pub(super) const RAW_TO_RDF_PROMOTION_ALLOWED: bool = false;
pub(super) const ONTOLOGY_TRUTH: bool = false;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct CandidateObjectRow {
    pub(super) candidate_id: String,
    pub(super) candidate_kind: &'static str,
    pub(super) status: &'static str,
    pub(super) label: String,
    pub(super) suggested_term_key: String,
    pub(super) suggested_term_label: String,
    pub(super) source_file_id: String,
    pub(super) source_queue_id: String,
    pub(super) source_path: String,
    pub(super) category: String,
    pub(super) language: String,
    pub(super) extraction_route: String,
    pub(super) extraction_run_id: String,
    pub(super) source_sha256: String,
    pub(super) evidence_sha256: String,
    pub(super) text_char_count: String,
    pub(super) review_status: &'static str,
    pub(super) promotion_status: &'static str,
    pub(super) raw_to_rdf_promotion_allowed: bool,
    pub(super) ontology_truth: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct CandidateRelationRow {
    pub(super) candidate_id: String,
    pub(super) relation_kind: &'static str,
    pub(super) source_candidate_id: String,
    pub(super) target_candidate_id: String,
    pub(super) source_file_id: String,
    pub(super) source_queue_id: String,
    pub(super) extraction_run_id: String,
    pub(super) evidence_sha256: String,
    pub(super) review_status: &'static str,
    pub(super) promotion_status: &'static str,
    pub(super) ontology_truth: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct CandidateEvidenceRow {
    pub(super) evidence_id: String,
    pub(super) evidence_kind: &'static str,
    pub(super) source_file_id: String,
    pub(super) source_queue_id: String,
    pub(super) source_path: String,
    pub(super) source_sha256: String,
    pub(super) extraction_run_id: String,
    pub(super) cache_output_path: String,
    pub(super) evidence_sha256: String,
    pub(super) text_char_count: String,
    pub(super) review_status: &'static str,
    pub(super) promotion_status: &'static str,
    pub(super) ontology_truth: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct MappingTerm {
    pub(super) candidate_id: String,
    pub(super) stable_key: String,
    pub(super) label: String,
    pub(super) note: String,
    pub(super) term_kind: MappingTermKind,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum MappingTermKind {
    Object,
    Relation,
}

impl MappingTermKind {
    pub(super) fn candidate_kind(self) -> &'static str {
        match self {
            Self::Object => OBJECT_TERM_KIND,
            Self::Relation => RELATION_TERM_KIND,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct CacheEvidence {
    pub(super) run_id: String,
    pub(super) output_path: String,
    pub(super) queue_id: String,
    pub(super) file_id: String,
    pub(super) relative_path: String,
    pub(super) category: String,
    pub(super) language: String,
    pub(super) extraction_route: String,
    pub(super) source_sha256: String,
    pub(super) text_sha256: String,
    pub(super) text_char_count: usize,
    pub(super) extracted_text: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct CacheOutputRow {
    pub(super) status: String,
    pub(super) queue_id: String,
    pub(super) file_id: String,
    pub(super) relative_path: String,
    pub(super) category: String,
    pub(super) language: String,
    pub(super) extraction_route: String,
    pub(super) source_sha256: String,
    pub(super) text_sha256: Option<String>,
    pub(super) text_char_count: Option<usize>,
    pub(super) extracted_text: Option<String>,
    pub(super) ontology_truth: Option<bool>,
    pub(super) raw_to_rdf_promotion_allowed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CandidateGenerationReceipt {
    pub(super) schema_version: &'static str,
    pub(super) run_id: String,
    pub(super) domain: String,
    pub(super) source_revision: String,
    pub(super) extraction_run_ids: Vec<String>,
    pub(super) source_file_count: usize,
    pub(super) mapping_term_count: usize,
    pub(super) extraction_evidence_count: usize,
    pub(super) candidate_object_count: usize,
    pub(super) candidate_relation_count: usize,
    pub(super) candidate_evidence_count: usize,
    pub(super) raw_to_rdf_promotion_allowed: bool,
    pub(super) ontology_truth: bool,
}

pub(super) struct CandidateGenerationInputs {
    pub(super) run_id: String,
    pub(super) domain: String,
    pub(super) primary_language: String,
    pub(super) source_manifest_path: String,
    pub(super) mapping_ledger_path: String,
    pub(super) files: Vec<EpistemeFileRow>,
    pub(super) mapping_terms: Vec<MappingTerm>,
    pub(super) cache_evidence: Vec<CacheEvidence>,
    pub(super) source_revision: String,
}

pub(super) struct CandidateRows {
    pub(super) objects: Vec<CandidateObjectRow>,
    pub(super) relations: Vec<CandidateRelationRow>,
    pub(super) evidence: Vec<CandidateEvidenceRow>,
}

pub(super) struct CandidateGenerationOutputPaths {
    pub(super) run_dir: PathBuf,
    pub(super) objects_tsv: PathBuf,
    pub(super) relations_tsv: PathBuf,
    pub(super) evidence_tsv: PathBuf,
    pub(super) review_ledger_org: PathBuf,
    pub(super) receipt_json: PathBuf,
}

impl CandidateGenerationOutputPaths {
    pub(super) fn new(run_root: &Path, run_id: &str) -> Self {
        let run_dir = run_root.join(run_id);
        Self {
            objects_tsv: run_dir.join(OBJECTS_TSV),
            relations_tsv: run_dir.join(RELATIONS_TSV),
            evidence_tsv: run_dir.join(EVIDENCE_TSV),
            review_ledger_org: run_dir.join(REVIEW_LEDGER_ORG),
            receipt_json: run_dir.join(RECEIPT_JSON),
            run_dir,
        }
    }
}
