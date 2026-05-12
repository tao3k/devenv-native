//! Entity deduplication, normalization, and similarity scoring.

use crate::entity::Entity;
use crate::graph::{GraphError, KnowledgeGraph, read_lock};
use crate::search::normalized_score;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use unicode_normalization::UnicodeNormalization;

/// Result of deduplication operation.
#[derive(Debug, Clone, Default)]
pub struct DeduplicationResult {
    /// Number of duplicate groups found
    pub duplicate_groups_found: usize,
    /// Number of entities merged (removed)
    pub entities_merged: usize,
}

#[derive(Debug, Default)]
struct EntityMergeDraft {
    canonical: Option<Entity>,
    aliases: Vec<String>,
    sources: Vec<String>,
    max_confidence: f32,
}

impl KnowledgeGraph {
    /// Calculate similarity between two entity names (0.0 to 1.0).
    #[must_use]
    pub fn name_similarity(name1: &str, name2: &str) -> f32 {
        let n1 = normalize_name(name1);
        let n2 = normalize_name(name2);

        if n1 == n2 {
            return 1.0;
        }

        // Exact substring match
        if n1.contains(&n2) || n2.contains(&n1) {
            return 0.9;
        }

        // Edit-distance-based similarity
        let similarity = normalized_score(&n1, &n2, false);

        // Apply bonus for word overlap
        let words1: HashSet<&str> = n1.split_whitespace().collect();
        let words2: HashSet<&str> = n2.split_whitespace().collect();
        let overlap = bounded_usize_to_f32(words1.intersection(&words2).count());
        let word_bonus = if !words1.is_empty() && !words2.is_empty() {
            overlap / bounded_usize_to_f32(words1.len() + words2.len()) * 0.2
        } else {
            0.0
        };

        (similarity + word_bonus).clamp(0.0, 1.0)
    }

    /// Find potential duplicate entities.
    pub fn find_duplicates(&self, threshold: f32) -> Vec<Vec<String>> {
        let names = entity_name_snapshot(self);
        let mut groups: Vec<Vec<String>> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();

        for (name, id) in &names {
            if visited.contains(id.as_str()) {
                continue;
            }

            let group = duplicate_group_for_entity(name, id, &names, &mut visited, threshold);
            if group.len() > 1 {
                groups.push(group);
            }
        }

        groups
    }

    /// Merge multiple entities into a single canonical entity.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::EntityNotFound`] when none of the provided IDs
    /// resolve to an entity, or any error from removing or re-adding the
    /// canonical entity during the merge transaction.
    pub fn merge_entities(
        &self,
        entity_ids: &[String],
        canonical_name: &str,
    ) -> Result<Entity, GraphError> {
        let entities = read_lock(&self.entities);
        let mut draft = collect_entity_merge_draft(&entities, entity_ids);
        drop(entities);

        let Some(mut canonical) = draft.canonical.take() else {
            return Err(GraphError::EntityNotFound(entity_ids.join(", ")));
        };

        apply_merge_draft(&mut canonical, canonical_name, draft);
        self.replace_merged_entities(entity_ids, &canonical)?;
        Ok(canonical)
    }

    /// Auto-deduplicate the graph based on similarity threshold.
    pub fn deduplicate(&self, threshold: f32) -> DeduplicationResult {
        let duplicates = self.find_duplicates(threshold);

        let mut merged_count = 0;
        let duplicate_groups = duplicates.len();

        for group in &duplicates {
            if group.len() > 1 {
                let canonical_name = self.find_canonical_name(group);
                if self.merge_entities(group, &canonical_name).is_ok() {
                    merged_count += group.len() - 1;
                }
            }
        }

        DeduplicationResult {
            duplicate_groups_found: duplicate_groups,
            entities_merged: merged_count,
        }
    }

    fn replace_merged_entities(
        &self,
        entity_ids: &[String],
        canonical: &Entity,
    ) -> Result<(), GraphError> {
        for id in entity_ids {
            self.remove_entity(id)?;
        }

        self.add_entity(canonical.clone())?;
        Ok(())
    }

    /// Find the most canonical name from a group of entity IDs.
    fn find_canonical_name(&self, entity_ids: &[String]) -> String {
        let entities = read_lock(&self.entities);

        let mut best: Option<(usize, String)> = None;

        for id in entity_ids {
            if let Some(entity) = entities.get(id) {
                let score = entity.description.len() + entity.aliases.len() * 10;
                if let Some((best_score, _)) = &best {
                    if score > *best_score {
                        best = Some((score, entity.name.clone()));
                    }
                } else {
                    best = Some((score, entity.name.clone()));
                }
            }
        }

        best.map_or_else(
            || entity_ids.first().cloned().unwrap_or_default(),
            |(_, name)| name,
        )
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn entity_name_snapshot(graph: &KnowledgeGraph) -> Vec<(String, String)> {
    read_lock(&graph.entities)
        .values()
        .map(|entity: &Entity| (entity.name.clone(), entity.id.clone()))
        .collect()
}

fn duplicate_group_for_entity(
    name: &str,
    id: &str,
    names: &[(String, String)],
    visited: &mut HashSet<String>,
    threshold: f32,
) -> Vec<String> {
    let duplicate_ids = duplicate_entity_ids(name, id, names, visited, threshold);
    visited.extend(std::iter::once(id.to_string()).chain(duplicate_ids.iter().cloned()));
    std::iter::once(id.to_string())
        .chain(duplicate_ids)
        .collect()
}

fn duplicate_entity_ids(
    name: &str,
    id: &str,
    names: &[(String, String)],
    visited: &HashSet<String>,
    threshold: f32,
) -> Vec<String> {
    names
        .iter()
        .filter(|(other_name, other_id)| {
            is_duplicate_candidate(name, id, other_name, other_id, visited, threshold)
        })
        .map(|(_, other_id)| other_id.clone())
        .collect()
}

fn is_duplicate_candidate(
    name: &str,
    id: &str,
    other_name: &str,
    other_id: &str,
    visited: &HashSet<String>,
    threshold: f32,
) -> bool {
    id != other_id
        && !visited.contains(other_id)
        && KnowledgeGraph::name_similarity(name, other_name) >= threshold
}

fn collect_entity_merge_draft(
    entities: &HashMap<String, Entity>,
    entity_ids: &[String],
) -> EntityMergeDraft {
    entity_ids
        .iter()
        .filter_map(|id| entities.get(id))
        .fold(EntityMergeDraft::default(), add_entity_to_merge_draft)
}

fn add_entity_to_merge_draft(mut draft: EntityMergeDraft, entity: &Entity) -> EntityMergeDraft {
    if let Some(canonical) = draft.canonical.as_mut() {
        collect_merge_aliases(canonical, entity, &mut draft.aliases);
        collect_merge_source(entity, &mut draft.sources);
        draft.max_confidence = draft.max_confidence.max(entity.confidence);
    } else {
        draft.canonical = Some(entity.clone());
    }
    draft
}

fn collect_merge_aliases(canonical: &Entity, entity: &Entity, aliases: &mut Vec<String>) {
    aliases.extend(
        entity
            .aliases
            .iter()
            .filter(|alias| !canonical.aliases.contains(alias))
            .cloned(),
    );
    if !canonical.aliases.contains(&entity.name) {
        aliases.push(entity.name.clone());
    }
}

fn collect_merge_source(entity: &Entity, sources: &mut Vec<String>) {
    if let Some(ref source) = entity.source
        && !sources.contains(source)
    {
        sources.push(source.clone());
    }
}

fn apply_merge_draft(canonical: &mut Entity, canonical_name: &str, draft: EntityMergeDraft) {
    if !canonical_name.is_empty() {
        canonical.name = canonical_name.to_string();
    }
    canonical.aliases = merged_aliases(canonical, draft.aliases);

    if !draft.sources.is_empty() {
        canonical
            .metadata
            .insert("merged_sources".to_string(), json!(draft.sources));
    }

    canonical.confidence = draft.max_confidence.max(canonical.confidence);
    canonical.updated_at = chrono::Utc::now();
}

fn merged_aliases(canonical: &Entity, aliases: Vec<String>) -> Vec<String> {
    let mut existing_aliases = canonical.aliases.clone();
    existing_aliases.extend(aliases);
    existing_aliases.sort();
    existing_aliases.dedup();
    existing_aliases
}

fn bounded_usize_to_f32(value: usize) -> f32 {
    u16::try_from(value).map_or(f32::from(u16::MAX), f32::from)
}

/// Normalize entity name for comparison (Unicode NFKC + lowercase).
fn normalize_name(name: &str) -> String {
    let normalized: String = name.nfkc().collect();
    normalized
        .to_lowercase()
        .trim()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
}
