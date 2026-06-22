use std::collections::HashSet;

use anyhow::{Context, Result};

use super::model::{DraftInputs, ReviewRecord};

pub(super) fn validate_review_gate(inputs: &DraftInputs) -> Result<()> {
    if !inputs.quality.review_gate_passed {
        anyhow::bail!("ontology RDF draft export requires `reviewGatePassed=true`");
    }
    validate_quality_counts(inputs)?;
    validate_review_row_count(inputs)?;
    require_object_reviews(inputs)?;
    require_relation_reviews(inputs)?;
    require_evidence_reviews(inputs)?;
    validate_relation_candidate_references(inputs)?;
    Ok(())
}

pub(super) fn require_review<'a>(
    inputs: &'a DraftInputs,
    record_id: &str,
) -> Result<&'a ReviewRecord> {
    inputs
        .reviews_by_id
        .get(record_id)
        .with_context(|| format!("review row for `{record_id}` is missing"))
}

fn validate_quality_counts(inputs: &DraftInputs) -> Result<()> {
    if inputs.quality.candidate_object_count != inputs.objects.len() {
        anyhow::bail!(
            "quality report object count {} does not match candidate object count {}",
            inputs.quality.candidate_object_count,
            inputs.objects.len()
        );
    }
    if inputs.quality.candidate_relation_count != inputs.relations.len() {
        anyhow::bail!(
            "quality report relation count {} does not match candidate relation count {}",
            inputs.quality.candidate_relation_count,
            inputs.relations.len()
        );
    }
    if inputs.quality.candidate_evidence_count != inputs.evidence.len() {
        anyhow::bail!(
            "quality report evidence count {} does not match candidate evidence count {}",
            inputs.quality.candidate_evidence_count,
            inputs.evidence.len()
        );
    }
    Ok(())
}

fn validate_review_row_count(inputs: &DraftInputs) -> Result<()> {
    let expected_review_rows =
        inputs.objects.len() + inputs.relations.len() + inputs.evidence.len();
    if inputs.quality.review_row_count != expected_review_rows
        || inputs.reviews_by_id.len() != expected_review_rows
    {
        anyhow::bail!(
            "review row count must match candidate rows; report={}, parsed={}, expected={}",
            inputs.quality.review_row_count,
            inputs.reviews_by_id.len(),
            expected_review_rows
        );
    }
    Ok(())
}

fn require_object_reviews(inputs: &DraftInputs) -> Result<()> {
    for object in &inputs.objects {
        validate_safe_candidate_flags(
            object.raw_to_rdf_promotion_allowed,
            object.ontology_truth,
            object.candidate_id.as_str(),
        )?;
        require_review(inputs, object.candidate_id.as_str())?;
    }
    Ok(())
}

fn require_relation_reviews(inputs: &DraftInputs) -> Result<()> {
    for relation in &inputs.relations {
        validate_safe_candidate_flags(
            false,
            relation.ontology_truth,
            relation.candidate_id.as_str(),
        )?;
        require_review(inputs, relation.candidate_id.as_str())?;
    }
    Ok(())
}

fn require_evidence_reviews(inputs: &DraftInputs) -> Result<()> {
    for evidence in &inputs.evidence {
        validate_safe_candidate_flags(
            false,
            evidence.ontology_truth,
            evidence.evidence_id.as_str(),
        )?;
        require_review(inputs, evidence.evidence_id.as_str())?;
    }
    Ok(())
}

fn validate_relation_candidate_references(inputs: &DraftInputs) -> Result<()> {
    let object_ids = inputs
        .objects
        .iter()
        .map(|object| object.candidate_id.as_str())
        .collect::<HashSet<_>>();
    for relation in &inputs.relations {
        if !object_ids.contains(relation.source_candidate_id.as_str()) {
            anyhow::bail!(
                "relation `{}` references unknown source candidate `{}`",
                relation.candidate_id,
                relation.source_candidate_id
            );
        }
        if !object_ids.contains(relation.target_candidate_id.as_str()) {
            anyhow::bail!(
                "relation `{}` references unknown target candidate `{}`",
                relation.candidate_id,
                relation.target_candidate_id
            );
        }
    }
    Ok(())
}

fn validate_safe_candidate_flags(
    raw_to_rdf_promotion_allowed: bool,
    ontology_truth: bool,
    record_id: &str,
) -> Result<()> {
    if raw_to_rdf_promotion_allowed {
        anyhow::bail!("record `{record_id}` attempted raw-to-RDF promotion");
    }
    if ontology_truth {
        anyhow::bail!("record `{record_id}` is already marked as ontology truth");
    }
    Ok(())
}
