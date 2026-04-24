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
use xiuxian_wendao_core::WendaoResourceUri;
use xiuxian_wendao_parsers::parse_frontmatter;

use crate::{build_embedded_wendao_registry, embedded_resource_text};

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

        let mut entities_added = 0usize;
        let mut relations_linked = 0usize;

        for file in files {
            if !is_skill_descriptor_path(file.path()) {
                continue;
            }

            let Some(markdown) = embedded_resource_text(file.path()) else {
                return Err(Error::Internal(format!(
                    "embedded resource `{}` declared in registry but not found in binary",
                    file.path()
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
                continue;
            };

            if self
                .graph
                .add_entity(build_skill_entity(
                    skill_name.as_str(),
                    file.path(),
                    frontmatter.description.as_deref(),
                    frontmatter.routing_keywords.as_slice(),
                    frontmatter.intents.as_slice(),
                ))
                .map_err(|error| Error::Internal(format!("Graph operation failed: {error}")))?
            {
                entities_added = entities_added.saturating_add(1);
            }
            let (intent_entities, intent_relations) =
                self.index_skill_intents(skill_name.as_str(), file.path(), &frontmatter.intents)?;
            entities_added = entities_added.saturating_add(intent_entities);
            relations_linked = relations_linked.saturating_add(intent_relations);

            let mut ids = file.link_targets_by_id().iter().collect::<Vec<_>>();
            ids.sort_by_key(|(left, _)| *left);

            for (id, targets) in ids {
                let config_type = registry
                    .get(id.as_str())
                    .map(|block| block.config_type.trim().to_ascii_lowercase());
                for target in dedup_targets(targets) {
                    let parsed_uri = WendaoResourceUri::parse(target.target_path.as_str())
                        .map_err(|error| {
                            Error::Internal(format!(
                                "invalid embedded skill link `{}` (id=`{id}` file=`{}`): {error}",
                                target.target_path,
                                file.path()
                            ))
                        })?;

                    let (reference_entity, reference_name) = build_reference_entity(
                        &parsed_uri,
                        file.path(),
                        id.as_str(),
                        target.reference_type.as_deref(),
                        config_type.as_deref(),
                    );

                    if self.graph.add_entity(reference_entity).map_err(|error| {
                        Error::Internal(format!("Graph operation failed: {error}"))
                    })? {
                        entities_added = entities_added.saturating_add(1);
                    }

                    self.graph
                        .add_relation(build_reference_relation(&ReferenceRelationInput {
                            skill_name: skill_name.as_str(),
                            reference_name: reference_name.as_str(),
                            source_path: file.path(),
                            reference_id: id.as_str(),
                            reference_path: parsed_uri.entity_name(),
                            target_uri: target.target_path.as_str(),
                            explicit_reference_type: target.reference_type.as_deref(),
                            config_type: config_type.as_deref(),
                        }))
                        .map_err(|error| {
                            Error::Internal(format!("Graph operation failed: {error}"))
                        })?;
                    relations_linked = relations_linked.saturating_add(1);
                }
            }
        }

        Ok((entities_added, relations_linked))
    }

    /// Trigger graph indexing for only the embedded skill references.
    ///
    /// # Errors
    /// Returns an error when graph operations fail.
    pub fn index_embedded_skill_references_only(&self) -> Result<(usize, usize)> {
        self.index_embedded_skill_references()
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
