//! Deterministic source-revision hashing for semantic projections.

use crate::semantic_ssot::types::{
    SemanticConfidenceSource, SemanticObject, SemanticObjectKind, SemanticProjection,
    SemanticRelationKind, SemanticRepository, SemanticStatus,
};
use std::collections::BTreeMap;

pub(super) fn semantic_object_by_id(
    repository: &SemanticRepository,
) -> BTreeMap<&str, &SemanticObject> {
    repository
        .objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<BTreeMap<_, _>>()
}

pub(super) fn semantic_projection_source_revision_from_map(
    projection: &SemanticProjection,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
) -> Option<String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"semantic-ssot-source-revision-v1\0");
    update_hash_field(&mut hasher, "projection", projection.projection.as_str());
    for object_id in &projection.source_objects {
        let object = object_by_id.get(object_id.as_str())?;
        update_hash_field(&mut hasher, "object.id", object.id.as_str());
        update_hash_field(&mut hasher, "object.kind", object_kind_token(&object.kind));
        update_hash_field(&mut hasher, "object.title", object.title.as_str());
        update_hash_field(&mut hasher, "object.status", status_token(&object.status));
        update_hash_field(
            &mut hasher,
            "object.confidence.score",
            &format!("{:.12}", object.confidence.score),
        );
        update_hash_field(
            &mut hasher,
            "object.confidence.source",
            confidence_source_token(&object.confidence.source),
        );
        for owner in &object.owners {
            update_hash_field(&mut hasher, "object.owner.scope", owner.scope.as_str());
            update_hash_field(&mut hasher, "object.owner.role", owner.role.as_str());
        }
        update_hash_field(
            &mut hasher,
            "object.provenance.source",
            object.provenance.source.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "object.provenance.recorded_by",
            object.provenance.recorded_by.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "object.provenance.recorded_at",
            object.provenance.recorded_at.as_str(),
        );
        for command in &object.verification.required {
            update_hash_field(
                &mut hasher,
                "object.verification.required",
                command.as_str(),
            );
        }
        for evidence in &object.verification.evidence {
            update_hash_field(
                &mut hasher,
                "object.verification.evidence",
                evidence.as_str(),
            );
        }
        for relation in &object.relations {
            update_hash_field(
                &mut hasher,
                "object.relation.kind",
                relation_kind_token(&relation.kind),
            );
            update_hash_field(
                &mut hasher,
                "object.relation.target",
                relation.target.as_str(),
            );
        }
        update_hash_field(
            &mut hasher,
            "object.source_path",
            object.source_path.to_string_lossy().as_ref(),
        );
        update_hash_field(&mut hasher, "object.body", object.body.as_str());
    }
    Some(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn update_hash_field(hasher: &mut blake3::Hasher, name: &str, value: &str) {
    hasher.update(name.as_bytes());
    hasher.update(b"\0");
    hasher.update(value.as_bytes());
    hasher.update(b"\0");
}

fn object_kind_token(kind: &SemanticObjectKind) -> &'static str {
    kind.id_prefix()
}

fn status_token(status: &SemanticStatus) -> &'static str {
    match status {
        SemanticStatus::Draft => "draft",
        SemanticStatus::Candidate => "candidate",
        SemanticStatus::Active => "active",
        SemanticStatus::Superseded => "superseded",
        SemanticStatus::Deprecated => "deprecated",
        SemanticStatus::Retired => "retired",
    }
}

fn confidence_source_token(source: &SemanticConfidenceSource) -> &'static str {
    match source {
        SemanticConfidenceSource::HumanSigned => "human_signed",
        SemanticConfidenceSource::Verified => "verified",
        SemanticConfidenceSource::LlmSuggested => "llm_suggested",
    }
}

fn relation_kind_token(kind: &SemanticRelationKind) -> &'static str {
    match kind {
        SemanticRelationKind::Contains => "contains",
        SemanticRelationKind::DependsOn => "depends_on",
        SemanticRelationKind::Constrains => "constrains",
        SemanticRelationKind::Implements => "implements",
        SemanticRelationKind::Governs => "governs",
        SemanticRelationKind::Affects => "affects",
        SemanticRelationKind::Validates => "validates",
        SemanticRelationKind::Supersedes => "supersedes",
        SemanticRelationKind::ProjectsTo => "projects_to",
        SemanticRelationKind::ConsumedBy => "consumed_by",
    }
}
