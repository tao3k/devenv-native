use crate::graph::{GraphError, KnowledgeGraph, read_lock};
use serde_json::{Value, json, to_string};

impl KnowledgeGraph {
    /// Export graph as JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::InvalidRelation`] when serialization fails.
    pub fn export_as_json(&self) -> Result<String, GraphError> {
        let entities =
            read_lock::<std::collections::HashMap<String, crate::entity::Entity>>(&self.entities);
        let relations = read_lock::<std::collections::HashMap<String, crate::entity::Relation>>(
            &self.relations,
        );

        let entities_json: Vec<Value> = entities
            .values()
            .map(|entity| {
                json!({
                    "id": entity.id,
                    "name": entity.name,
                    "entity_type": entity.entity_type.to_string(),
                    "description": entity.description,
                    "source": entity.source,
                    "aliases": entity.aliases,
                    "confidence": entity.confidence,
                })
            })
            .collect();

        let relations_json: Vec<Value> = relations
            .values()
            .map(|relation| {
                json!({
                    "id": relation.id,
                    "source": relation.source,
                    "target": relation.target,
                    "relation_type": relation.relation_type.to_string(),
                    "description": relation.description,
                    "source_doc": relation.source_doc,
                    "confidence": relation.confidence,
                })
            })
            .collect();

        let export = json!({
            "version": 1,
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "total_entities": entities_json.len(),
            "total_relations": relations_json.len(),
            "entities": entities_json,
            "relations": relations_json,
        });

        to_string(&export)
            .map_err(|error| GraphError::InvalidRelation("export".to_string(), error.to_string()))
    }
}
