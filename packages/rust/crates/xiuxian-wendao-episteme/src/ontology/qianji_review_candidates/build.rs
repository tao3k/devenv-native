//! Candidate row construction from validated Qianji review patches.

use std::path::Path;

use anyhow::{Context, Result, bail};

use super::{
    ids::{
        endpoint_display_name, endpoint_rdf_class, evidence_id, evidence_sha256,
        object_candidate_id, object_model_link_candidate_id,
        object_model_link_endpoint_candidate_id, object_model_type_candidate_id,
        patch_evidence_text, relation_candidate_id, relation_endpoint_candidate_id, relation_label,
        suggested_or_unknown,
    },
    types::{
        CandidateEvidenceRow, CandidateObjectRow, CandidateRelationRow, EpistemeCandidatePatch,
        EpistemePatchEvidence, EpistemeReview, OBJECT_MODEL_LINK_TYPE_PATCH_KIND,
        OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND,
    },
    validate::{validate_link_type_patch, validate_object_type_patch, validate_patch_contract},
};

pub(super) fn append_review_candidates(
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
    let evidence_sha256 = evidence_sha256(evidence_text.as_str());
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
    let evidence_sha256 = evidence_sha256(evidence_text.as_str());
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
    let evidence_sha256 = evidence_sha256(evidence_text.as_str());
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
    let label = relation_label(patch);
    let first_evidence = first_patch_evidence(artifact_path, patch, label.as_str())?;
    let evidence_text = patch_evidence_text(&patch.source_evidence);
    let evidence_sha256 = evidence_sha256(evidence_text.as_str());
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
