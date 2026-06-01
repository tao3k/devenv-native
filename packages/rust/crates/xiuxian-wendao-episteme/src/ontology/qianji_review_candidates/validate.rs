//! Deterministic validation for Qianji review import contracts.

use std::path::Path;

use anyhow::{Result, bail};

use super::types::{
    EpistemeCandidatePatch, EpistemeObjectModelLinkTypePatch, EpistemeObjectModelObjectTypePatch,
    EpistemeReview,
};

pub(super) fn validate_review(review: &EpistemeReview, path: &Path) -> Result<()> {
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

pub(super) fn validate_patch_contract(
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

pub(super) fn validate_object_type_patch(
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

pub(super) fn validate_link_type_patch(
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
