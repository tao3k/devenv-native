use crate::entity::{Entity, EntityType, Relation, RelationType};
use crate::skill_runtime::zhixing::indexer::resource_graph::helpers::{
    dedup_targets, is_skill_descriptor_path, normalize_token,
};
use crate::skill_runtime::zhixing::indexer::resource_graph::references::{
    ReferenceRelationInput, build_reference_entity, build_reference_relation,
};
use crate::skill_runtime::zhixing::indexer::types::ZhixingWendaoIndexer;
use crate::skill_runtime::zhixing::{Error, Result};
use serde_json::json;
use std::collections::HashMap;
use xiuxian_wendao_core::WendaoResourceUri;
use xiuxian_wendao_parsers::parse_frontmatter;

use crate::{WendaoResourceRegistry, build_embedded_wendao_registry, embedded_resource_text};

impl ZhixingWendaoIndexer {
    pub(in crate::skill_runtime::zhixing::indexer) fn index_embedded_skill_references(
        &self,
    ) -> Result<(usize, usize)> {
        let registry = build_embedded_wendao_registry().map_err(|error| {
            Error::Internal(format!(
                "failed to build embedded zhixing skill registry for graph indexing: {error}"
            ))
        })?;
        let mut files = registry.files().collect::<Vec<_>>();
        files.sort_by(|left, right| left.path().cmp(right.path()));

        files
            .into_iter()
            .filter(|file| is_skill_descriptor_path(file.path()))
            .try_fold(
                (0usize, 0usize),
                |(entities_added, relations_linked), file| {
                    self.index_embedded_skill_reference_file(
                        &registry,
                        file.path(),
                        file.link_targets_by_id(),
                    )
                    .map(|(file_entities, file_relations)| {
                        (
                            entities_added.saturating_add(file_entities),
                            relations_linked.saturating_add(file_relations),
                        )
                    })
                },
            )
    }

    /// Trigger graph indexing for only the embedded skill references.
    ///
    /// # Errors
    /// Returns an error when graph operations fail.
    /// Tuple API boundary: this public API preserves byte or count pairs used by existing addressing contracts.
    pub fn index_embedded_skill_references_only(&self) -> Result<(usize, usize)> {
        self.index_embedded_skill_references()
    }

    fn index_embedded_skill_reference_file(
        &self,
        registry: &WendaoResourceRegistry,
        file_path: &str,
        link_targets_by_id: &HashMap<String, Vec<crate::WendaoResourceLinkTarget>>,
    ) -> Result<(usize, usize)> {
        let Some(markdown) = embedded_resource_text(file_path) else {
            return Err(Error::Internal(format!(
                "embedded resource `{file_path}` declared in registry but not found in binary"
            )));
        };

        let frontmatter = parse_frontmatter(markdown);
        let Some(skill_name) = frontmatter
            .name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_ascii_lowercase)
        else {
            return Ok((0, 0));
        };

        let skill_entity_added = usize::from(
            self.graph
                .add_entity(build_skill_entity(
                    skill_name.as_str(),
                    file_path,
                    frontmatter.description.as_deref(),
                    frontmatter.routing_keywords.as_slice(),
                    frontmatter.intents.as_slice(),
                ))
                .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?,
        );
        let (intent_entities, intent_relations) =
            self.index_skill_intents(skill_name.as_str(), file_path, &frontmatter.intents)?;
        let (reference_entities, reference_relations) = self.index_skill_reference_targets(
            registry,
            file_path,
            skill_name.as_str(),
            link_targets_by_id,
        )?;

        Ok((
            skill_entity_added
                .saturating_add(intent_entities)
                .saturating_add(reference_entities),
            intent_relations.saturating_add(reference_relations),
        ))
    }

    fn index_skill_reference_targets(
        &self,
        registry: &WendaoResourceRegistry,
        file_path: &str,
        skill_name: &str,
        link_targets_by_id: &HashMap<String, Vec<crate::WendaoResourceLinkTarget>>,
    ) -> Result<(usize, usize)> {
        let mut ids = link_targets_by_id.iter().collect::<Vec<_>>();
        ids.sort_by_key(|(left, _)| *left);
        ids.into_iter().try_fold(
            (0usize, 0usize),
            |(entities_added, relations_linked), (id, targets)| {
                let config_type = registry
                    .get(id.as_str())
                    .map(|block| block.config_type.trim().to_ascii_lowercase());
                self.index_skill_reference_target_group(
                    file_path,
                    skill_name,
                    id.as_str(),
                    targets,
                    config_type.as_deref(),
                )
                .map(|(group_entities, group_relations)| {
                    (
                        entities_added.saturating_add(group_entities),
                        relations_linked.saturating_add(group_relations),
                    )
                })
            },
        )
    }

    fn index_skill_reference_target_group(
        &self,
        file_path: &str,
        skill_name: &str,
        id: &str,
        targets: &[crate::WendaoResourceLinkTarget],
        config_type: Option<&str>,
    ) -> Result<(usize, usize)> {
        dedup_targets(targets).into_iter().try_fold(
            (0usize, 0usize),
            |(entities_added, relations_linked), target| {
                let parsed_uri =
                    WendaoResourceUri::parse(target.target_path.as_str()).map_err(|error| {
                        Error::Internal(format!(
                            "invalid embedded skill link `{}` (id=`{id}` file=`{file_path}`): {error}",
                            target.target_path
                        ))
                    })?;

                let (reference_entity, reference_name) = build_reference_entity(
                    &parsed_uri,
                    file_path,
                    id,
                    target.reference_type.as_deref(),
                    config_type,
                );

                let entity_added = usize::from(self.graph.add_entity(reference_entity).map_err(
                    |error| Error::Internal(format!("Graph operation failed: {error}")),
                )?);

                self.graph
                    .add_relation(build_reference_relation(&ReferenceRelationInput {
                        skill_name,
                        reference_name: reference_name.as_str(),
                        source_path: file_path,
                        reference_id: id,
                        reference_path: parsed_uri.entity_name(),
                        target_uri: target.target_path.as_str(),
                        explicit_reference_type: target.reference_type.as_deref(),
                        config_type,
                    }))
                    .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?;
                Ok((
                    entities_added.saturating_add(entity_added),
                    relations_linked.saturating_add(1),
                ))
            },
        )
    }

    fn index_skill_intents(
        &self,
        skill_name: &str,
        source_path: &str,
        intents: &[String],
    ) -> Result<(usize, usize)> {
        let mut entities_added = 0usize;
        let mut relations_added = 0usize;
        let mut normalized_intents = intents
            .iter()
            .map(|intent| intent.trim())
            .filter(|intent| !intent.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        normalized_intents.sort();
        normalized_intents.dedup();

        for intent in normalized_intents {
            let intent_name = format!("intent:{intent}");
            let intent_id = normalize_token(intent.as_str());
            let mut intent_entity = Entity::new(
                format!("zhixing:intent:{intent_id}"),
                intent_name.clone(),
                EntityType::Concept,
                format!("Intent promoted from skill `{skill_name}`"),
            );
            intent_entity.source = Some(source_path.to_string());
            intent_entity
                .metadata
                .insert("zhixing_domain".to_string(), json!("skill_intent"));
            intent_entity
                .metadata
                .insert("skill_semantic_name".to_string(), json!(skill_name));
            intent_entity
                .metadata
                .insert("source_skill_doc".to_string(), json!(source_path));
            intent_entity
                .metadata
                .insert("intent".to_string(), json!(intent.as_str()));
            if self
                .graph
                .add_entity(intent_entity)
                .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?
            {
                entities_added = entities_added.saturating_add(1);
            }
            self.graph
                .add_relation(
                    Relation::new(
                        skill_name.to_string(),
                        intent_name,
                        RelationType::Governs,
                        format!("Skill `{skill_name}` governs intent `{intent}`"),
                    )
                    .with_source_doc(Some(source_path.to_string()))
                    .with_metadata("intent".to_string(), json!(intent.as_str())),
                )
                .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?;
            relations_added = relations_added.saturating_add(1);
        }

        Ok((entities_added, relations_added))
    }
}

fn build_skill_entity(
    skill_name: &str,
    source_path: &str,
    description: Option<&str>,
    routing_keywords: &[String],
    intents: &[String],
) -> Entity {
    let summary = description
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(
            || format!("Skill descriptor for `{skill_name}`"),
            ToString::to_string,
        );
    let mut entity = Entity::new(
        format!("zhixing:skill:{skill_name}"),
        skill_name.to_string(),
        EntityType::Skill,
        summary,
    );
    entity.source = Some(source_path.to_string());
    entity
        .metadata
        .insert("zhixing_domain".to_string(), json!("skill"));
    entity
        .metadata
        .insert("skill_semantic_name".to_string(), json!(skill_name));
    entity
        .metadata
        .insert("source_skill_doc".to_string(), json!(source_path));
    if !routing_keywords.is_empty() {
        entity.metadata.insert(
            "routing_keywords".to_string(),
            json!(routing_keywords.to_vec()),
        );
    }
    if !intents.is_empty() {
        entity
            .metadata
            .insert("intents".to_string(), json!(intents.to_vec()));
    }
    entity
}
