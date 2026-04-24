use crate::enhancer::classify_skill_reference;
use crate::entity::{Entity, Relation};
use serde_json::json;
use std::path::Path;
use xiuxian_wendao_core::WendaoResourceUri;

use crate::skill_runtime::zhixing::indexer::resource_graph::helpers::normalize_token;

pub(crate) struct ReferenceRelationInput<'a> {
    pub(crate) skill_name: &'a str,
    pub(crate) reference_name: &'a str,
    pub(crate) source_path: &'a str,
    pub(crate) reference_id: &'a str,
    pub(crate) reference_path: &'a str,
    pub(crate) target_uri: &'a str,
    pub(crate) explicit_reference_type: Option<&'a str>,
    pub(crate) config_type: Option<&'a str>,
}

pub(crate) fn build_reference_entity(
    uri: &WendaoResourceUri,
    source_path: &str,
    reference_id: &str,
    explicit_reference_type: Option<&str>,
    config_type: Option<&str>,
) -> (Entity, String) {
    let reference_name = reference_leaf_name(uri.entity_name());
    let stable_token = normalize_token(uri.entity_name());
    let semantics =
        classify_skill_reference(explicit_reference_type, config_type, uri.entity_name());
    let mut entity = Entity::new(
        format!("zhixing:skill_ref:{}:{stable_token}", uri.semantic_name()),
        reference_name.clone(),
        semantics.entity,
        format!(
            "Semantic skill reference `{}` from `{}`",
            uri.entity_name(),
            uri.semantic_name()
        ),
    );
    entity.source = Some(source_path.to_string());
    entity
        .metadata
        .insert("zhixing_domain".to_string(), json!("skill_reference"));
    entity
        .metadata
        .insert("source_skill_doc".to_string(), json!(source_path));
    entity.metadata.insert(
        "skill_semantic_name".to_string(),
        json!(uri.semantic_name()),
    );
    entity
        .metadata
        .insert("reference_id".to_string(), json!(reference_id));
    entity
        .metadata
        .insert("reference_path".to_string(), json!(uri.entity_name()));
    entity.metadata.insert(
        "reference_uri".to_string(),
        json!(format!(
            "wendao://skills/{}/references/{}",
            uri.semantic_name(),
            uri.entity_name()
        )),
    );
    if let Some(reference_type) = semantics.reference_type {
        entity
            .metadata
            .insert("reference_type".to_string(), json!(reference_type));
    }
    (entity, reference_name)
}

pub(crate) fn build_reference_relation(input: &ReferenceRelationInput<'_>) -> Relation {
    let semantics = classify_skill_reference(
        input.explicit_reference_type,
        input.config_type,
        input.reference_path,
    );
    let relation_type = semantics.relation;
    let relation_label = relation_type.to_string();
    let mut relation = Relation::new(
        input.skill_name.to_string(),
        input.reference_name.to_string(),
        relation_type,
        format!(
            "Skill `{}` {} `{}`",
            input.skill_name, relation_label, input.reference_name
        ),
    )
    .with_source_doc(Some(input.source_path.to_string()))
    .with_metadata("reference_id".to_string(), json!(input.reference_id))
    .with_metadata("reference_uri".to_string(), json!(input.target_uri));

    if let Some(reference_type) = semantics.reference_type {
        relation = relation.with_metadata("reference_type".to_string(), json!(reference_type));
    }

    relation
}

pub(crate) fn reference_leaf_name(entity_path: &str) -> String {
    let path = Path::new(entity_path);
    path.file_stem()
        .or_else(|| path.file_name())
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(|| entity_path.trim().to_string(), ToString::to_string)
}
