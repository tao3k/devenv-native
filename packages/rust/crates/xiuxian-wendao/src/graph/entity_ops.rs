use super::core::{read_lock, write_lock};
use super::{GraphError, KnowledgeGraph};
use crate::entity::{Entity, Relation};
use log::info;
use std::collections::{HashMap, HashSet};

impl KnowledgeGraph {
    /// Add an entity.
    ///
    /// Returns `true` if newly added, `false` if updated in place.
    ///
    /// # Errors
    ///
    /// This currently never returns an error and is modeled as `Result` for API compatibility.
    #[allow(clippy::unnecessary_wraps)]
    pub fn add_entity(&self, entity: Entity) -> Result<bool, GraphError> {
        let mut entities = write_lock::<HashMap<String, Entity>>(&self.entities);
        let mut entities_by_name = write_lock::<HashMap<String, String>>(&self.entities_by_name);
        let mut entities_by_type =
            write_lock::<HashMap<String, Vec<String>>>(&self.entities_by_type);

        let type_str = entity.entity_type.to_string();
        if let Some(existing_id) = entities_by_name.get(&entity.name)
            && let Some(existing) = entities.get_mut(existing_id)
        {
            if existing.entity_type.to_string() != type_str {
                if let Some(ids) = entities_by_type.get_mut(&existing.entity_type.to_string()) {
                    ids.retain(|id| id != existing_id);
                }
                entities_by_type
                    .entry(type_str.clone())
                    .or_default()
                    .push(existing_id.clone());
                existing.entity_type = entity.entity_type;
            }
            existing.description = entity.description;
            existing.metadata = entity.metadata;
            existing.source = entity.source.or(existing.source.clone());
            existing.aliases = entity.aliases;
            existing.confidence = entity.confidence;
            existing.updated_at = entity.updated_at;
            info!("Updated entity: {}", entity.name);
            return Ok(false);
        }

        let entity_id = entity.id.clone();
        entities.insert(entity_id.clone(), entity.clone());
        entities_by_name.insert(entity.name.clone(), entity_id.clone());

        entities_by_type
            .entry(type_str)
            .or_default()
            .push(entity_id.clone());

        info!("Added entity: {} ({})", entity.name, entity.entity_type);
        Ok(true)
    }

    /// Get an entity by ID.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn get_entity(&self, entity_id: &str) -> Option<Entity> {
        read_lock::<HashMap<String, Entity>>(&self.entities)
            .get(entity_id)
            .cloned()
    }

    /// Get an entity by name.
    #[must_use]
    pub fn get_entity_by_name(&self, name: &str) -> Option<Entity> {
        let entities_by_name = read_lock::<HashMap<String, String>>(&self.entities_by_name);
        if let Some(entity_id) = entities_by_name.get(name) {
            return read_lock::<HashMap<String, Entity>>(&self.entities)
                .get(entity_id)
                .cloned();
        }
        None
    }

    /// Get entities by type.
    #[must_use]
    pub fn get_entities_by_type(&self, entity_type: &str) -> Vec<Entity> {
        let entities_by_type = read_lock::<HashMap<String, Vec<String>>>(&self.entities_by_type);
        let entities = read_lock::<HashMap<String, Entity>>(&self.entities);

        if let Some(entity_ids) = entities_by_type.get(entity_type) {
            entity_ids
                .iter()
                .filter_map(|id| entities.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clear all entities and relations.
    pub fn clear(&mut self) {
        write_lock::<HashMap<String, Entity>>(&self.entities).clear();
        write_lock::<HashMap<String, String>>(&self.entities_by_name).clear();
        write_lock::<HashMap<String, Relation>>(&self.relations).clear();
        write_lock::<HashMap<String, HashSet<String>>>(&self.outgoing_relations).clear();
        write_lock::<HashMap<String, HashSet<String>>>(&self.incoming_relations).clear();
        write_lock::<HashMap<String, Vec<String>>>(&self.entities_by_type).clear();
        info!("Knowledge graph cleared");
    }

    /// Get all entities as a vector.
    #[must_use]
    pub fn get_all_entities(&self) -> Vec<Entity> {
        read_lock::<HashMap<String, Entity>>(&self.entities)
            .values()
            .cloned()
            .collect()
    }

    /// Remove an entity by ID.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::EntityNotFound`] if the entity ID is absent.
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn remove_entity(&self, entity_id: &str) -> Result<(), GraphError> {
        let mut entities = write_lock::<HashMap<String, Entity>>(&self.entities);
        let mut entities_by_name = write_lock::<HashMap<String, String>>(&self.entities_by_name);
        let mut entities_by_type =
            write_lock::<HashMap<String, Vec<String>>>(&self.entities_by_type);
        let mut relations = write_lock::<HashMap<String, Relation>>(&self.relations);
        let mut outgoing = write_lock::<HashMap<String, HashSet<String>>>(&self.outgoing_relations);
        let mut incoming = write_lock::<HashMap<String, HashSet<String>>>(&self.incoming_relations);

        if let Some(entity) = entities.remove(entity_id) {
            entities_by_name.remove(&entity.name);

            if let Some(ids) = entities_by_type.get_mut(&entity.entity_type.to_string()) {
                ids.retain(|id| id != entity_id);
            }

            let rel_ids_to_remove: Vec<String> = relations
                .keys()
                .filter(|id: &&String| {
                    if let Some(rel) = relations.get(id.as_str()) {
                        rel.source == entity.name || rel.target == entity.name
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            for rid in rel_ids_to_remove {
                relations.remove(&rid);
                outgoing.remove(&entity.name);
                incoming.remove(&entity.name);
            }

            info!("Removed entity: {} ({})", entity.name, entity_id);
            Ok(())
        } else {
            Err(GraphError::EntityNotFound(entity_id.to_string()))
        }
    }
}
