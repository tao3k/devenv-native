//! Import Qianji Episteme review artifacts into deterministic candidate rows.

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::candidate_review::{
    EpistemeOntologyCandidateReviewReport, EpistemeOntologyCandidateReviewRequest,
    review_episteme_ontology_candidates,
};

const IMPORT_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_qianji_review_candidate_import.v1";
const QIANJI_RESPONSE_SCHEMA: &str = "qianji.openai_compatible_llm_response.v1";
const EPISTEME_REVIEW_SCHEMA: &str = "xiuxian.wendao.episteme.reasoning_fill_review.v1";
const OBJECTS_TSV: &str = "candidate_objects.tsv";
const RELATIONS_TSV: &str = "candidate_relations.tsv";
const EVIDENCE_TSV: &str = "candidate_evidence.tsv";
const IMPORT_REPORT_JSON: &str = "qianji_review_candidate_import_report.json";
const OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND: &str = "object_model_object_type_candidate";
const OBJECT_MODEL_LINK_TYPE_PATCH_KIND: &str = "object_model_link_type_candidate";

/// Request for importing Qianji Episteme review artifacts as candidate rows.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyQianjiReviewCandidateImportRequest {
    run_dir: PathBuf,
    review_artifacts: Vec<PathBuf>,
}

impl EpistemeOntologyQianjiReviewCandidateImportRequest {
    /// Create an import request for the ontology-generation run directory.
    #[must_use]
    pub fn new(run_dir: impl Into<PathBuf>) -> Self {
        Self {
            run_dir: run_dir.into(),
            review_artifacts: Vec::new(),
        }
    }

    /// Add a Qianji OpenAI-compatible review artifact to import.
    #[must_use]
    pub fn with_review_artifact(mut self, path: impl Into<PathBuf>) -> Self {
        self.review_artifacts.push(path.into());
        self
    }

    /// Ontology-generation run directory receiving candidate rows.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Report emitted after importing Qianji review artifacts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyQianjiReviewCandidateImportReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Run directory receiving candidate rows and review outputs.
    pub run_dir: PathBuf,
    /// Source Qianji review artifacts imported.
    pub qianji_review_artifacts: Vec<PathBuf>,
    /// Generated candidate object TSV path.
    pub candidate_objects_tsv: PathBuf,
    /// Generated candidate relation TSV path.
    pub candidate_relations_tsv: PathBuf,
    /// Generated candidate evidence TSV path.
    pub candidate_evidence_tsv: PathBuf,
    /// Generated import report path.
    pub import_report_json: PathBuf,
    /// Number of imported object candidates.
    pub candidate_object_count: usize,
    /// Number of imported relation candidates.
    pub candidate_relation_count: usize,
    /// Number of imported evidence rows.
    pub candidate_evidence_count: usize,
    /// Number of canonical review artifacts that produced no candidate rows.
    pub zero_candidate_review_count: usize,
    /// Total number of model-declared review blockers.
    pub review_blocker_count: usize,
    /// Review-gate report over the imported candidates.
    pub candidate_review: EpistemeOntologyCandidateReviewReport,
    /// Whether imported rows are ontology truth.
    pub ontology_truth: bool,
    /// Whether imported rows allow raw-to-RDF promotion.
    pub raw_to_rdf_promotion_allowed: bool,
}

#[derive(Debug, Deserialize)]
struct QianjiReviewArtifact {
    schema: String,
    #[serde(default)]
    episteme_review: Option<EpistemeReview>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpistemeReview {
    schema: String,
    status: String,
    fill_item_id: String,
    target_ledger_field_group: String,
    #[serde(default)]
    blockers: Vec<String>,
    candidate_patch_count: usize,
    candidate_patches: Vec<EpistemeCandidatePatch>,
    rdf_mutation: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpistemeCandidatePatch {
    patch_kind: String,
    #[serde(default)]
    fill_item_id: String,
    #[serde(default)]
    target_ledger_field_group: String,
    #[serde(default)]
    provisional_object_key: String,
    #[serde(default)]
    provisional_relation_key: String,
    #[serde(default)]
    ontology_class_key: String,
    #[serde(default)]
    relation_property_key: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    source_object_label: String,
    #[serde(default)]
    target_object_label: String,
    #[serde(default)]
    object_type: Option<EpistemeObjectModelObjectTypePatch>,
    #[serde(default)]
    link_type: Option<EpistemeObjectModelLinkTypePatch>,
    #[serde(default)]
    endpoint_object_types: Vec<EpistemeObjectModelEndpointObjectTypePatch>,
    #[serde(default)]
    source_evidence: Vec<EpistemePatchEvidence>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpistemeObjectModelObjectTypePatch {
    api_name: String,
    display_name: String,
    #[serde(default)]
    rdf_class: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpistemeObjectModelLinkTypePatch {
    api_name: String,
    display_name: String,
    rdf_property: String,
    from_object_type: String,
    to_object_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpistemeObjectModelEndpointObjectTypePatch {
    api_name: String,
    display_name: String,
    #[serde(default)]
    rdf_class: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpistemePatchEvidence {
    file_id: String,
    #[serde(default)]
    relative_path: String,
    quote: String,
}

#[derive(Debug)]
struct CandidateObjectRow {
    candidate_id: String,
    label: String,
    suggested_term_key: String,
    source_file_id: String,
    source_path: String,
    evidence_sha256: String,
    text_char_count: usize,
}

#[derive(Debug)]
struct CandidateRelationRow {
    candidate_id: String,
    relation_kind: String,
    source_candidate_id: String,
    target_candidate_id: String,
    source_file_id: String,
    evidence_sha256: String,
}

#[derive(Debug)]
struct CandidateEvidenceRow {
    evidence_id: String,
    source_file_id: String,
    source_path: String,
    evidence_sha256: String,
    text_char_count: usize,
}

/// Import Qianji review artifacts into candidate rows and run the review gate.
///
/// # Errors
///
/// Returns an error when a review artifact is missing, malformed, attempts RDF
/// mutation, contains unsupported patch kinds, or generated review artifacts
/// cannot be written.
pub fn import_episteme_ontology_qianji_review_candidates(
    request: &EpistemeOntologyQianjiReviewCandidateImportRequest,
) -> Result<EpistemeOntologyQianjiReviewCandidateImportReport> {
    if request.review_artifacts.is_empty() {
        bail!("Qianji review candidate import requires at least one review artifact");
    }
    let mut objects = Vec::new();
    let mut relations = Vec::new();
    let mut evidence = Vec::new();
    let mut zero_candidate_review_count = 0;
    let mut review_blocker_count = 0;
    for artifact_path in &request.review_artifacts {
        let review = read_review_artifact(artifact_path.as_path())?;
        review_blocker_count += review.blockers.len();
        if review.candidate_patch_count == 0 {
            zero_candidate_review_count += 1;
        }
        append_review_candidates(
            &review,
            artifact_path.as_path(),
            &mut objects,
            &mut relations,
            &mut evidence,
        )?;
    }

    fs::create_dir_all(request.run_dir()).with_context(|| {
        format!(
            "failed to create Qianji review candidate run dir `{}`",
            request.run_dir().display()
        )
    })?;
    let objects_tsv = request.run_dir().join(OBJECTS_TSV);
    let relations_tsv = request.run_dir().join(RELATIONS_TSV);
    let evidence_tsv = request.run_dir().join(EVIDENCE_TSV);
    let import_report_json = request.run_dir().join(IMPORT_REPORT_JSON);
    write_objects_tsv(objects_tsv.as_path(), &objects)?;
    write_relations_tsv(relations_tsv.as_path(), &relations)?;
    write_evidence_tsv(evidence_tsv.as_path(), &evidence)?;
    let candidate_review = review_episteme_ontology_candidates(
        &EpistemeOntologyCandidateReviewRequest::new(request.run_dir()),
    )?;
    let report = EpistemeOntologyQianjiReviewCandidateImportReport {
        schema_version: IMPORT_SCHEMA_VERSION,
        run_dir: request.run_dir().to_path_buf(),
        qianji_review_artifacts: request.review_artifacts.clone(),
        candidate_objects_tsv: objects_tsv,
        candidate_relations_tsv: relations_tsv,
        candidate_evidence_tsv: evidence_tsv,
        import_report_json,
        candidate_object_count: objects.len(),
        candidate_relation_count: relations.len(),
        candidate_evidence_count: evidence.len(),
        zero_candidate_review_count,
        review_blocker_count,
        candidate_review,
        ontology_truth: false,
        raw_to_rdf_promotion_allowed: false,
    };
    write_json(report.import_report_json.as_path(), &report)?;
    Ok(report)
}

fn read_review_artifact(path: &Path) -> Result<EpistemeReview> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let artifact: QianjiReviewArtifact = serde_json::from_str(raw.as_str())
        .with_context(|| format!("failed to parse `{}`", path.display()))?;
    if artifact.schema != QIANJI_RESPONSE_SCHEMA {
        bail!(
            "Qianji review artifact `{}` has unsupported schema `{}`",
            path.display(),
            artifact.schema
        );
    }
    let review = artifact.episteme_review.with_context(|| {
        format!(
            "Qianji review artifact `{}` has no episteme_review",
            path.display()
        )
    })?;
    validate_review(&review, path)?;
    Ok(review)
}

fn validate_review(review: &EpistemeReview, path: &Path) -> Result<()> {
    if review.schema != EPISTEME_REVIEW_SCHEMA {
        bail!(
            "Qianji review artifact `{}` has unsupported episteme_review schema `{}`",
            path.display(),
            review.schema
        );
    }
    if review.status != "review_only" {
        bail!(
            "Qianji review artifact `{}` episteme_review is not review_only",
            path.display()
        );
    }
    if review.rdf_mutation {
        bail!(
            "Qianji review artifact `{}` attempted RDF mutation",
            path.display()
        );
    }
    if review.candidate_patch_count != review.candidate_patches.len() {
        bail!(
            "Qianji review artifact `{}` candidatePatchCount does not match candidatePatches length",
            path.display()
        );
    }
    if review.candidate_patch_count == 0 && review.blockers.is_empty() {
        bail!(
            "Qianji review artifact `{}` has no candidatePatches and no blockers",
            path.display()
        );
    }
    if review.fill_item_id.trim().is_empty() || review.target_ledger_field_group.trim().is_empty() {
        bail!(
            "Qianji review artifact `{}` has blank fillItemId or targetLedgerFieldGroup",
            path.display()
        );
    }
    Ok(())
}

fn append_review_candidates(
    review: &EpistemeReview,
    artifact_path: &Path,
    objects: &mut Vec<CandidateObjectRow>,
    relations: &mut Vec<CandidateRelationRow>,
    evidence: &mut Vec<CandidateEvidenceRow>,
) -> Result<()> {
    for patch in &review.candidate_patches {
        validate_patch_contract(review, artifact_path, patch)?;
        match patch.patch_kind.as_str() {
            OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND => {
                append_object_model_type_candidate(
                    review,
                    artifact_path,
                    patch,
                    objects,
                    evidence,
                )?;
            }
            OBJECT_MODEL_LINK_TYPE_PATCH_KIND => {
                append_object_model_link_candidate(
                    review,
                    artifact_path,
                    patch,
                    objects,
                    relations,
                    evidence,
                )?;
            }
            "object_candidate" => {
                append_object_candidate(review, artifact_path, patch, objects, evidence)?;
            }
            "relation_candidate" => {
                append_relation_candidate(
                    review,
                    artifact_path,
                    patch,
                    objects,
                    relations,
                    evidence,
                )?;
            }
            _ => {
                bail!(
                    "Qianji review artifact `{}` has unsupported patch kind `{}`",
                    artifact_path.display(),
                    patch.patch_kind
                );
            }
        }
    }
    Ok(())
}

fn validate_patch_contract(
    review: &EpistemeReview,
    artifact_path: &Path,
    patch: &EpistemeCandidatePatch,
) -> Result<()> {
    if !patch.fill_item_id.trim().is_empty() && patch.fill_item_id != review.fill_item_id {
        bail!(
            "Qianji review artifact `{}` patch fillItemId does not match review fillItemId",
            artifact_path.display()
        );
    }
    if !patch.target_ledger_field_group.trim().is_empty()
        && patch.target_ledger_field_group != review.target_ledger_field_group
    {
        bail!(
            "Qianji review artifact `{}` patch targetLedgerFieldGroup does not match review targetLedgerFieldGroup",
            artifact_path.display()
        );
    }
    Ok(())
}

fn append_object_model_type_candidate(
    review: &EpistemeReview,
    artifact_path: &Path,
    patch: &EpistemeCandidatePatch,
    objects: &mut Vec<CandidateObjectRow>,
    evidence: &mut Vec<CandidateEvidenceRow>,
) -> Result<()> {
    let object_type = patch.object_type.as_ref().with_context(|| {
        format!(
            "Qianji review artifact `{}` has {OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND} without objectType",
            artifact_path.display()
        )
    })?;
    validate_object_type_patch(artifact_path, object_type)?;
    let first_evidence =
        first_patch_evidence(artifact_path, patch, object_type.display_name.as_str())?;
    let evidence_text = patch_evidence_text(&patch.source_evidence);
    let evidence_sha256 = format!("sha256:{}", sha256_text(evidence_text.as_str()));
    let candidate_id = object_model_type_candidate_id(review, object_type);
    objects.push(CandidateObjectRow {
        candidate_id: candidate_id.clone(),
        label: object_type.display_name.clone(),
        suggested_term_key: suggested_or_unknown(object_type.rdf_class.as_str()),
        source_file_id: first_evidence.file_id.clone(),
        source_path: first_evidence.relative_path.clone(),
        evidence_sha256: evidence_sha256.clone(),
        text_char_count: evidence_text.chars().count(),
    });
    evidence.push(CandidateEvidenceRow {
        evidence_id: evidence_id(candidate_id.as_str()),
        source_file_id: first_evidence.file_id.clone(),
        source_path: first_evidence.relative_path.clone(),
        evidence_sha256,
        text_char_count: evidence_text.chars().count(),
    });
    Ok(())
}

fn append_object_model_link_candidate(
    review: &EpistemeReview,
    artifact_path: &Path,
    patch: &EpistemeCandidatePatch,
    objects: &mut Vec<CandidateObjectRow>,
    relations: &mut Vec<CandidateRelationRow>,
    evidence: &mut Vec<CandidateEvidenceRow>,
) -> Result<()> {
    let link_type = patch.link_type.as_ref().with_context(|| {
        format!(
            "Qianji review artifact `{}` has {OBJECT_MODEL_LINK_TYPE_PATCH_KIND} without linkType",
            artifact_path.display()
        )
    })?;
    validate_link_type_patch(artifact_path, link_type)?;
    let first_evidence =
        first_patch_evidence(artifact_path, patch, link_type.display_name.as_str())?;
    let evidence_text = patch_evidence_text(&patch.source_evidence);
    let evidence_sha256 = format!("sha256:{}", sha256_text(evidence_text.as_str()));
    let text_char_count = evidence_text.chars().count();
    let source_label = endpoint_display_name(patch, link_type.from_object_type.as_str());
    let target_label = endpoint_display_name(patch, link_type.to_object_type.as_str());
    let source_candidate_id = object_model_link_endpoint_candidate_id(
        review,
        link_type.api_name.as_str(),
        "source",
        source_label.as_str(),
    );
    let target_candidate_id = object_model_link_endpoint_candidate_id(
        review,
        link_type.api_name.as_str(),
        "target",
        target_label.as_str(),
    );
    let relation_candidate_id = object_model_link_candidate_id(review, link_type);

    objects.push(CandidateObjectRow {
        candidate_id: source_candidate_id.clone(),
        label: source_label,
        suggested_term_key: endpoint_rdf_class(patch, link_type.from_object_type.as_str()),
        source_file_id: first_evidence.file_id.clone(),
        source_path: first_evidence.relative_path.clone(),
        evidence_sha256: evidence_sha256.clone(),
        text_char_count,
    });
    objects.push(CandidateObjectRow {
        candidate_id: target_candidate_id.clone(),
        label: target_label,
        suggested_term_key: endpoint_rdf_class(patch, link_type.to_object_type.as_str()),
        source_file_id: first_evidence.file_id.clone(),
        source_path: first_evidence.relative_path.clone(),
        evidence_sha256: evidence_sha256.clone(),
        text_char_count,
    });
    relations.push(CandidateRelationRow {
        candidate_id: relation_candidate_id.clone(),
        relation_kind: suggested_or_unknown(link_type.rdf_property.as_str()),
        source_candidate_id,
        target_candidate_id,
        source_file_id: first_evidence.file_id.clone(),
        evidence_sha256: evidence_sha256.clone(),
    });
    evidence.push(CandidateEvidenceRow {
        evidence_id: evidence_id(relation_candidate_id.as_str()),
        source_file_id: first_evidence.file_id.clone(),
        source_path: first_evidence.relative_path.clone(),
        evidence_sha256,
        text_char_count,
    });
    Ok(())
}

fn validate_object_type_patch(
    artifact_path: &Path,
    object_type: &EpistemeObjectModelObjectTypePatch,
) -> Result<()> {
    if object_type.api_name.trim().is_empty()
        || object_type.display_name.trim().is_empty()
        || object_type.rdf_class.trim().is_empty()
    {
        bail!(
            "Qianji review artifact `{}` has objectType with blank apiName, displayName, or rdfClass",
            artifact_path.display()
        );
    }
    Ok(())
}

fn validate_link_type_patch(
    artifact_path: &Path,
    link_type: &EpistemeObjectModelLinkTypePatch,
) -> Result<()> {
    if link_type.api_name.trim().is_empty()
        || link_type.display_name.trim().is_empty()
        || link_type.rdf_property.trim().is_empty()
        || link_type.from_object_type.trim().is_empty()
        || link_type.to_object_type.trim().is_empty()
    {
        bail!(
            "Qianji review artifact `{}` has linkType with blank apiName, displayName, rdfProperty, fromObjectType, or toObjectType",
            artifact_path.display()
        );
    }
    Ok(())
}

fn append_object_candidate(
    review: &EpistemeReview,
    artifact_path: &Path,
    patch: &EpistemeCandidatePatch,
    objects: &mut Vec<CandidateObjectRow>,
    evidence: &mut Vec<CandidateEvidenceRow>,
) -> Result<()> {
    if patch.label.trim().is_empty() {
        bail!(
            "Qianji review artifact `{}` has object_candidate with blank label",
            artifact_path.display()
        );
    }
    let first_evidence = first_patch_evidence(artifact_path, patch, patch.label.as_str())?;
    let evidence_text = patch_evidence_text(&patch.source_evidence);
    let evidence_sha256 = format!("sha256:{}", sha256_text(evidence_text.as_str()));
    let candidate_id = object_candidate_id(review, patch);
    objects.push(CandidateObjectRow {
        candidate_id: candidate_id.clone(),
        label: patch.label.clone(),
        suggested_term_key: suggested_or_unknown(patch.ontology_class_key.as_str()),
        source_file_id: first_evidence.file_id.clone(),
        source_path: first_evidence.relative_path.clone(),
        evidence_sha256: evidence_sha256.clone(),
        text_char_count: evidence_text.chars().count(),
    });
    evidence.push(CandidateEvidenceRow {
        evidence_id: evidence_id(candidate_id.as_str()),
        source_file_id: first_evidence.file_id.clone(),
        source_path: first_evidence.relative_path.clone(),
        evidence_sha256,
        text_char_count: evidence_text.chars().count(),
    });
    Ok(())
}

fn append_relation_candidate(
    review: &EpistemeReview,
    artifact_path: &Path,
    patch: &EpistemeCandidatePatch,
    objects: &mut Vec<CandidateObjectRow>,
    relations: &mut Vec<CandidateRelationRow>,
    evidence: &mut Vec<CandidateEvidenceRow>,
) -> Result<()> {
    if patch.source_object_label.trim().is_empty() || patch.target_object_label.trim().is_empty() {
        bail!(
            "Qianji review artifact `{}` has relation_candidate with blank endpoint label",
            artifact_path.display()
        );
    }
    let first_evidence =
        first_patch_evidence(artifact_path, patch, patch.relation_label().as_str())?;
    let evidence_text = patch_evidence_text(&patch.source_evidence);
    let evidence_sha256 = format!("sha256:{}", sha256_text(evidence_text.as_str()));
    let text_char_count = evidence_text.chars().count();
    let source_candidate_id = relation_endpoint_candidate_id(review, patch, "source");
    let target_candidate_id = relation_endpoint_candidate_id(review, patch, "target");
    let relation_candidate_id = relation_candidate_id(review, patch);

    objects.push(CandidateObjectRow {
        candidate_id: source_candidate_id.clone(),
        label: patch.source_object_label.clone(),
        suggested_term_key: "unknown_candidate".to_owned(),
        source_file_id: first_evidence.file_id.clone(),
        source_path: first_evidence.relative_path.clone(),
        evidence_sha256: evidence_sha256.clone(),
        text_char_count,
    });
    objects.push(CandidateObjectRow {
        candidate_id: target_candidate_id.clone(),
        label: patch.target_object_label.clone(),
        suggested_term_key: "unknown_candidate".to_owned(),
        source_file_id: first_evidence.file_id.clone(),
        source_path: first_evidence.relative_path.clone(),
        evidence_sha256: evidence_sha256.clone(),
        text_char_count,
    });
    relations.push(CandidateRelationRow {
        candidate_id: relation_candidate_id.clone(),
        relation_kind: suggested_or_unknown(patch.relation_property_key.as_str()),
        source_candidate_id,
        target_candidate_id,
        source_file_id: first_evidence.file_id.clone(),
        evidence_sha256: evidence_sha256.clone(),
    });
    evidence.push(CandidateEvidenceRow {
        evidence_id: evidence_id(relation_candidate_id.as_str()),
        source_file_id: first_evidence.file_id.clone(),
        source_path: first_evidence.relative_path.clone(),
        evidence_sha256,
        text_char_count,
    });
    Ok(())
}

fn first_patch_evidence<'a>(
    artifact_path: &Path,
    patch: &'a EpistemeCandidatePatch,
    label: &str,
) -> Result<&'a EpistemePatchEvidence> {
    patch.source_evidence.first().with_context(|| {
        format!(
            "Qianji review artifact `{}` {} `{}` has no sourceEvidence",
            artifact_path.display(),
            patch.patch_kind,
            label
        )
    })
}

fn object_candidate_id(review: &EpistemeReview, patch: &EpistemeCandidatePatch) -> String {
    let seed = if patch.provisional_object_key.trim().is_empty() {
        format!("{}:{}", review.fill_item_id, patch.label)
    } else {
        format!("{}:{}", review.fill_item_id, patch.provisional_object_key)
    };
    format!("qianji.object.{}", short_hash(seed.as_str()))
}

fn object_model_type_candidate_id(
    review: &EpistemeReview,
    object_type: &EpistemeObjectModelObjectTypePatch,
) -> String {
    let seed = format!("{}:{}", review.fill_item_id, object_type.api_name);
    format!("qianji.object.{}", short_hash(seed.as_str()))
}

fn object_model_link_endpoint_candidate_id(
    review: &EpistemeReview,
    link_api_name: &str,
    role: &str,
    label: &str,
) -> String {
    let seed = format!("{}:{link_api_name}:{role}:{label}", review.fill_item_id);
    format!("qianji.object.{}", short_hash(seed.as_str()))
}

fn object_model_link_candidate_id(
    review: &EpistemeReview,
    link_type: &EpistemeObjectModelLinkTypePatch,
) -> String {
    let seed = format!(
        "{}:{}:{}:{}",
        review.fill_item_id,
        link_type.api_name,
        link_type.from_object_type,
        link_type.to_object_type
    );
    format!("qianji.relation.{}", short_hash(seed.as_str()))
}

fn relation_endpoint_candidate_id(
    review: &EpistemeReview,
    patch: &EpistemeCandidatePatch,
    role: &str,
) -> String {
    let label = match role {
        "source" => patch.source_object_label.as_str(),
        "target" => patch.target_object_label.as_str(),
        _ => "",
    };
    let seed = format!(
        "{}:{}:{}:{}",
        review.fill_item_id,
        relation_key_seed(patch),
        role,
        label
    );
    format!("qianji.object.{}", short_hash(seed.as_str()))
}

fn relation_candidate_id(review: &EpistemeReview, patch: &EpistemeCandidatePatch) -> String {
    let seed = format!(
        "{}:{}:{}:{}:{}",
        review.fill_item_id,
        relation_key_seed(patch),
        patch.source_object_label,
        patch.relation_property_key,
        patch.target_object_label
    );
    format!("qianji.relation.{}", short_hash(seed.as_str()))
}

fn relation_key_seed(patch: &EpistemeCandidatePatch) -> String {
    if patch.provisional_relation_key.trim().is_empty() {
        "unknown_relation".to_owned()
    } else {
        patch.provisional_relation_key.clone()
    }
}

fn suggested_or_unknown(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "unknown_candidate".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn endpoint_display_name(patch: &EpistemeCandidatePatch, api_name: &str) -> String {
    patch
        .endpoint_object_types
        .iter()
        .find(|endpoint| endpoint.api_name == api_name)
        .map(|endpoint| endpoint.display_name.trim())
        .filter(|label| !label.is_empty())
        .unwrap_or(api_name)
        .to_owned()
}

fn endpoint_rdf_class(patch: &EpistemeCandidatePatch, api_name: &str) -> String {
    patch
        .endpoint_object_types
        .iter()
        .find(|endpoint| endpoint.api_name == api_name)
        .map(|endpoint| endpoint.rdf_class.trim())
        .filter(|rdf_class| !rdf_class.is_empty())
        .map_or_else(|| "unknown_candidate".to_owned(), ToOwned::to_owned)
}

fn evidence_id(candidate_id: &str) -> String {
    format!("qianji.evidence.{}", short_hash(candidate_id))
}

impl EpistemeCandidatePatch {
    fn relation_label(&self) -> String {
        format!(
            "{} -> {} -> {}",
            self.source_object_label, self.relation_property_key, self.target_object_label
        )
    }
}

fn patch_evidence_text(evidence: &[EpistemePatchEvidence]) -> String {
    evidence
        .iter()
        .map(|row| row.quote.trim())
        .filter(|quote| !quote.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn write_objects_tsv(path: &Path, rows: &[CandidateObjectRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "candidate_id\tcandidate_kind\tstatus\tlabel\tsuggested_term_key\tsuggested_term_label\tsource_file_id\tsource_queue_id\tsource_path\tcategory\tlanguage\textraction_route\textraction_run_id\tsource_sha256\tevidence_sha256\ttext_char_count\treview_status\tpromotion_status\traw_to_rdf_promotion_allowed\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\tontology_candidate.qianji_object_patch\tcandidate\t{}\t{}\t{}\t{}\t\t{}\t\t\tqianji_episteme_review\t\t\t{}\t{}\treview_required\tblocked_pending_review\tfalse\tfalse",
            tsv(row.candidate_id.as_str()),
            tsv(row.label.as_str()),
            tsv(row.suggested_term_key.as_str()),
            tsv(row.suggested_term_key.as_str()),
            tsv(row.source_file_id.as_str()),
            tsv(row.source_path.as_str()),
            tsv(row.evidence_sha256.as_str()),
            row.text_char_count
        )?;
    }
    Ok(())
}

fn write_relations_tsv(path: &Path, rows: &[CandidateRelationRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "candidate_id\trelation_kind\tsource_candidate_id\ttarget_candidate_id\tsource_file_id\tsource_queue_id\textraction_run_id\tevidence_sha256\treview_status\tpromotion_status\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t\t\t{}\treview_required\tblocked_pending_review\tfalse",
            tsv(row.candidate_id.as_str()),
            tsv(row.relation_kind.as_str()),
            tsv(row.source_candidate_id.as_str()),
            tsv(row.target_candidate_id.as_str()),
            tsv(row.source_file_id.as_str()),
            tsv(row.evidence_sha256.as_str())
        )?;
    }
    Ok(())
}

fn write_evidence_tsv(path: &Path, rows: &[CandidateEvidenceRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "evidence_id\tevidence_kind\tsource_file_id\tsource_queue_id\tsource_path\tsource_sha256\textraction_run_id\tcache_output_path\tevidence_sha256\ttext_char_count\treview_status\tpromotion_status\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\tontology_candidate.qianji_review_evidence\t{}\t\t{}\t\t\t\t{}\t{}\treview_required\tblocked_pending_review\tfalse",
            tsv(row.evidence_id.as_str()),
            tsv(row.source_file_id.as_str()),
            tsv(row.source_path.as_str()),
            tsv(row.evidence_sha256.as_str()),
            row.text_char_count
        )?;
    }
    Ok(())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, value)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

fn create_file(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn tsv(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ").trim().to_owned()
}

fn sha256_text(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn short_hash(value: &str) -> String {
    sha256_text(value).chars().take(16).collect()
}
