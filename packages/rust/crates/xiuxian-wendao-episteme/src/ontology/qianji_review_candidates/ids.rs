//! Stable identifiers and compact evidence helpers for imported candidates.

use sha2::{Digest, Sha256};

use super::types::{
    EpistemeCandidatePatch, EpistemeObjectModelLinkTypePatch, EpistemeObjectModelObjectTypePatch,
    EpistemePatchEvidence, EpistemeReview,
};

pub(super) fn object_candidate_id(
    review: &EpistemeReview,
    patch: &EpistemeCandidatePatch,
) -> String {
    let seed = if patch.provisional_object_key.trim().is_empty() {
        format!("{}:{}", review.fill_item_id, patch.label)
    } else {
        format!("{}:{}", review.fill_item_id, patch.provisional_object_key)
    };
    format!("qianji.object.{}", short_hash(seed.as_str()))
}

pub(super) fn object_model_type_candidate_id(
    review: &EpistemeReview,
    object_type: &EpistemeObjectModelObjectTypePatch,
) -> String {
    let seed = format!("{}:{}", review.fill_item_id, object_type.api_name);
    format!("qianji.object.{}", short_hash(seed.as_str()))
}

pub(super) fn object_model_link_endpoint_candidate_id(
    review: &EpistemeReview,
    link_api_name: &str,
    role: &str,
    label: &str,
) -> String {
    let seed = format!("{}:{link_api_name}:{role}:{label}", review.fill_item_id);
    format!("qianji.object.{}", short_hash(seed.as_str()))
}

pub(super) fn object_model_link_candidate_id(
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

pub(super) fn relation_endpoint_candidate_id(
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

pub(super) fn relation_candidate_id(
    review: &EpistemeReview,
    patch: &EpistemeCandidatePatch,
) -> String {
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

pub(super) fn relation_key_seed(patch: &EpistemeCandidatePatch) -> String {
    if patch.provisional_relation_key.trim().is_empty() {
        "unknown_relation".to_owned()
    } else {
        patch.provisional_relation_key.clone()
    }
}

pub(super) fn relation_label(patch: &EpistemeCandidatePatch) -> String {
    format!(
        "{} -> {} -> {}",
        patch.source_object_label, patch.relation_property_key, patch.target_object_label
    )
}

pub(super) fn suggested_or_unknown(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "unknown_candidate".to_owned()
    } else {
        trimmed.to_owned()
    }
}

pub(super) fn endpoint_display_name(patch: &EpistemeCandidatePatch, api_name: &str) -> String {
    patch
        .endpoint_object_types
        .iter()
        .find(|endpoint| endpoint.api_name == api_name)
        .map(|endpoint| endpoint.display_name.trim())
        .filter(|label| !label.is_empty())
        .unwrap_or(api_name)
        .to_owned()
}

pub(super) fn endpoint_rdf_class(patch: &EpistemeCandidatePatch, api_name: &str) -> String {
    patch
        .endpoint_object_types
        .iter()
        .find(|endpoint| endpoint.api_name == api_name)
        .map(|endpoint| endpoint.rdf_class.trim())
        .filter(|rdf_class| !rdf_class.is_empty())
        .map_or_else(|| "unknown_candidate".to_owned(), ToOwned::to_owned)
}

pub(super) fn evidence_id(candidate_id: &str) -> String {
    format!("qianji.evidence.{}", short_hash(candidate_id))
}

pub(super) fn patch_evidence_text(evidence: &[EpistemePatchEvidence]) -> String {
    evidence
        .iter()
        .map(|row| row.quote.trim())
        .filter(|quote| !quote.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn evidence_sha256(evidence_text: &str) -> String {
    format!("sha256:{}", sha256_text(evidence_text))
}

fn sha256_text(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn short_hash(value: &str) -> String {
    sha256_text(value).chars().take(16).collect()
}
