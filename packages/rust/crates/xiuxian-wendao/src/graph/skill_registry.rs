//! Bulk skill entity registration (Bridge 4: Core 2 → Core 1).
#![allow(clippy::doc_markdown)]
//!
//! Accepts parsed skill docs and creates SKILL, TOOL, CONCEPT entities
//! with CONTAINS and RELATED_TO relations in the KnowledgeGraph.

use super::{GraphError, KnowledgeGraph};
use crate::entity::{Entity, EntityType, Relation, RelationType};
use log::info;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct SkillCollection {
    skills: HashMap<String, Vec<String>>,
    tool_keywords: HashMap<String, HashSet<String>>,
    entities_added: usize,
}

fn truncate_description(content: &str) -> String {
    content.chars().take(200).collect()
}

fn skill_entity(skill_name: &str, content: &str) -> Entity {
    let description = if content.is_empty() {
        format!("Skill: {skill_name}")
    } else {
        truncate_description(content)
    };
    Entity::new(
        format!("skill:{}", skill_name.to_lowercase().replace(' ', "_")),
        skill_name.to_string(),
        EntityType::Skill,
        description,
    )
}

fn resolved_tool_name(doc: &SkillDoc) -> Option<String> {
    let tool_name = if doc.tool_name.is_empty() {
        doc.id.clone()
    } else {
        doc.tool_name.clone()
    };
    (!tool_name.is_empty()).then_some(tool_name)
}

fn tool_entity(tool_name: &str, content: &str) -> Entity {
    Entity::new(
        format!("tool:{}", tool_name.to_lowercase().replace([' ', '.'], "_")),
        tool_name.to_string(),
        EntityType::Tool,
        truncate_description(content),
    )
}

fn normalized_keywords(keywords: &[String]) -> HashSet<String> {
    keywords
        .iter()
        .filter(|keyword| !keyword.is_empty())
        .map(|keyword| keyword.to_lowercase())
        .collect()
}

fn all_keywords(tool_keywords: &HashMap<String, HashSet<String>>) -> HashSet<String> {
    let mut keywords = HashSet::new();
    for keyword_set in tool_keywords.values() {
        keywords.extend(keyword_set.iter().cloned());
    }
    keywords
}

/// A parsed skill document for bulk registration.
#[derive(Debug, Clone, Default)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct SkillDoc {
    /// Document ID
    pub id: String,
    /// "skill" or "command"
    pub doc_type: String,
    /// Parent skill name
    pub skill_name: String,
    /// Tool name (for commands)
    pub tool_name: String,
    /// Text content / description
    pub content: String,
    /// Routing keywords
    pub routing_keywords: Vec<String>,
}

/// Result of skill entity registration.
#[derive(Debug, Clone, Default)]
pub struct SkillRegistrationResult {
    /// Number of entities added
    pub entities_added: usize,
    /// Number of relations added
    pub relations_added: usize,
}

impl KnowledgeGraph {
    fn register_skill_doc(
        &self,
        doc: &SkillDoc,
        skills: &mut HashMap<String, Vec<String>>,
    ) -> Result<usize, GraphError> {
        if doc.skill_name.is_empty() {
            return Ok(0);
        }
        let was_added = self.add_entity(skill_entity(&doc.skill_name, &doc.content))?;
        skills.entry(doc.skill_name.clone()).or_default();
        Ok(usize::from(was_added))
    }

    fn ensure_skill_entity_for_tool(
        &self,
        skill_name: &str,
        skills: &HashMap<String, Vec<String>>,
    ) -> Result<usize, GraphError> {
        if skill_name.is_empty() || skills.contains_key(skill_name) {
            return Ok(0);
        }
        if self.get_entity_by_name(skill_name).is_some() {
            return Ok(0);
        }
        let was_added = self.add_entity(skill_entity(skill_name, ""))?;
        Ok(usize::from(was_added))
    }

    fn register_command_doc(
        &self,
        doc: &SkillDoc,
        skills: &mut HashMap<String, Vec<String>>,
        tool_keywords: &mut HashMap<String, HashSet<String>>,
    ) -> Result<usize, GraphError> {
        let Some(tool_name) = resolved_tool_name(doc) else {
            return Ok(0);
        };

        let mut entities_added =
            usize::from(self.add_entity(tool_entity(&tool_name, &doc.content))?);

        if !doc.skill_name.is_empty() {
            entities_added += self.ensure_skill_entity_for_tool(&doc.skill_name, skills)?;
            skills
                .entry(doc.skill_name.clone())
                .or_default()
                .push(tool_name.clone());
        }

        let keywords = normalized_keywords(&doc.routing_keywords);
        if !keywords.is_empty() {
            tool_keywords.insert(tool_name, keywords);
        }

        Ok(entities_added)
    }

    fn collect_skill_collection(&self, docs: &[SkillDoc]) -> Result<SkillCollection, GraphError> {
        let mut collection = SkillCollection::default();

        for doc in docs {
            let added = match doc.doc_type.as_str() {
                "skill" => self.register_skill_doc(doc, &mut collection.skills)?,
                "command" => self.register_command_doc(
                    doc,
                    &mut collection.skills,
                    &mut collection.tool_keywords,
                )?,
                _ => 0,
            };
            collection.entities_added = collection.entities_added.saturating_add(added);
        }

        Ok(collection)
    }

    fn register_contains_relations(
        &self,
        skills: &HashMap<String, Vec<String>>,
    ) -> Result<usize, GraphError> {
        let mut relations_added = 0usize;
        for (skill_name, tool_names) in skills {
            for tool_name in tool_names {
                let relation = Relation::new(
                    skill_name.clone(),
                    tool_name.clone(),
                    RelationType::Contains,
                    format!("{skill_name} contains {tool_name}"),
                );
                self.add_relation(relation)?;
                relations_added = relations_added.saturating_add(1);
            }
        }
        Ok(relations_added)
    }

    fn register_keyword_entities(
        &self,
        tool_keywords: &HashMap<String, HashSet<String>>,
    ) -> Result<usize, GraphError> {
        let mut entities_added = 0usize;
        for keyword in all_keywords(tool_keywords) {
            let concept_name = format!("keyword:{keyword}");
            let entity = Entity::new(
                format!("concept:{}", keyword.replace(' ', "_")),
                concept_name,
                EntityType::Concept,
                format!("Routing keyword: {keyword}"),
            );
            entities_added = entities_added.saturating_add(usize::from(self.add_entity(entity)?));
        }
        Ok(entities_added)
    }

    fn register_keyword_relations(
        &self,
        tool_keywords: &HashMap<String, HashSet<String>>,
    ) -> Result<usize, GraphError> {
        let mut relations_added = 0usize;
        for (tool_name, keyword_set) in tool_keywords {
            for keyword in keyword_set {
                let relation = Relation::new(
                    tool_name.clone(),
                    format!("keyword:{keyword}"),
                    RelationType::RelatedTo,
                    format!("{tool_name} has keyword {keyword}"),
                );
                self.add_relation(relation)?;
                relations_added = relations_added.saturating_add(1);
            }
        }
        Ok(relations_added)
    }

    /// Batch-register skill docs as entities and relations.
    ///
    /// Creates:
    /// - SKILL entities for each unique skill
    /// - TOOL entities for each command
    /// - CONTAINS relations: Skill → Tool
    /// - CONCEPT entities for each routing keyword
    /// - RELATED_TO relations: Tool → keyword:*
    ///
    /// Called during `omni sync` / `omni reindex`.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when entity/relation validation fails.
    pub fn register_skill_entities(
        &self,
        docs: &[SkillDoc],
    ) -> Result<SkillRegistrationResult, GraphError> {
        let mut collection = self.collect_skill_collection(docs)?;
        let mut relations_added = self.register_contains_relations(&collection.skills)?;
        collection.entities_added += self.register_keyword_entities(&collection.tool_keywords)?;
        relations_added += self.register_keyword_relations(&collection.tool_keywords)?;

        info!(
            "Skill entities registered: +{} entities, +{relations_added} relations",
            collection.entities_added
        );
        Ok(SkillRegistrationResult {
            entities_added: collection.entities_added,
            relations_added,
        })
    }
}
