use crate::entity::{Entity, Relation};
use crate::graph::{KnowledgeGraph, read_lock};
use std::collections::{HashMap, HashSet};

impl KnowledgeGraph {
    /// Multi-hop search: traverse both outgoing AND incoming relations.
    ///
    /// Unlike the previous version (outgoing only), this walks edges
    /// bidirectionally to discover entities connected in either direction.
    #[must_use]
    pub fn multi_hop_search(&self, start_name: &str, max_hops: usize) -> Vec<Entity> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut found_entities: Vec<Entity> = Vec::new();
        let mut frontier: Vec<String> = vec![start_name.to_string()];

        let entities_by_name = read_lock::<HashMap<String, String>>(&self.entities_by_name);
        let entities = read_lock::<HashMap<String, Entity>>(&self.entities);
        let outgoing = read_lock::<HashMap<String, HashSet<String>>>(&self.outgoing_relations);
        let incoming = read_lock::<HashMap<String, HashSet<String>>>(&self.incoming_relations);
        let relations = read_lock::<HashMap<String, Relation>>(&self.relations);

        for _hop in 0..max_hops {
            let mut next_frontier: Vec<String> = Vec::new();

            for entity_name in &frontier {
                if visited.contains(entity_name) {
                    continue;
                }
                visited.insert(entity_name.clone());

                if let Some(entity_id) = entities_by_name.get(entity_name)
                    && let Some(entity) = entities.get(entity_id)
                    && !found_entities.iter().any(|e| e.id == entity.id)
                {
                    found_entities.push(entity.clone());
                }

                // Walk outgoing relations (source -> target).
                if let Some(rel_ids) = outgoing.get(entity_name) {
                    for rel_id in rel_ids {
                        if let Some(relation) = relations.get(rel_id)
                            && !visited.contains(&relation.target)
                        {
                            next_frontier.push(relation.target.clone());
                        }
                    }
                }

                // Walk incoming relations (target <- source).
                if let Some(rel_ids) = incoming.get(entity_name) {
                    for rel_id in rel_ids {
                        if let Some(relation) = relations.get(rel_id)
                            && !visited.contains(&relation.source)
                        {
                            next_frontier.push(relation.source.clone());
                        }
                    }
                }
            }

            if next_frontier.is_empty() {
                break;
            }
            frontier = next_frontier;
        }

        found_entities
    }
}
