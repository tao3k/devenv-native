use crate::entity::{Entity, Relation};
use crate::graph::{KnowledgeGraph, read_lock};
use std::collections::{HashMap, HashSet};
use std::sync::RwLockReadGuard;

impl KnowledgeGraph {
    /// Multi-hop search: traverse both outgoing AND incoming relations.
    ///
    /// Unlike the previous version (outgoing only), this walks edges
    /// bidirectionally to discover entities connected in either direction.
    #[must_use]
    pub fn multi_hop_search(&self, start_name: &str, max_hops: usize) -> Vec<Entity> {
        let graph = TraversalGraph::read(self);
        let mut state = TraversalState::new(start_name);
        run_bidirectional_traversal(&graph, &mut state, max_hops);
        state.found_entities
    }
}

struct TraversalGraph<'a> {
    entities_by_name: RwLockReadGuard<'a, HashMap<String, String>>,
    entities: RwLockReadGuard<'a, HashMap<String, Entity>>,
    outgoing: RwLockReadGuard<'a, HashMap<String, HashSet<String>>>,
    incoming: RwLockReadGuard<'a, HashMap<String, HashSet<String>>>,
    relations: RwLockReadGuard<'a, HashMap<String, Relation>>,
}

impl<'a> TraversalGraph<'a> {
    fn read(graph: &'a KnowledgeGraph) -> Self {
        Self {
            entities_by_name: read_lock::<HashMap<String, String>>(&graph.entities_by_name),
            entities: read_lock::<HashMap<String, Entity>>(&graph.entities),
            outgoing: read_lock::<HashMap<String, HashSet<String>>>(&graph.outgoing_relations),
            incoming: read_lock::<HashMap<String, HashSet<String>>>(&graph.incoming_relations),
            relations: read_lock::<HashMap<String, Relation>>(&graph.relations),
        }
    }
}

struct TraversalState {
    visited: HashSet<String>,
    found_entities: Vec<Entity>,
    frontier: Vec<String>,
}

impl TraversalState {
    fn new(start_name: &str) -> Self {
        Self {
            visited: HashSet::new(),
            found_entities: Vec::new(),
            frontier: vec![start_name.to_string()],
        }
    }

    fn advance_to(&mut self, next_frontier: Vec<String>) -> bool {
        if next_frontier.is_empty() {
            return false;
        }
        self.frontier = next_frontier;
        true
    }
}

fn run_bidirectional_traversal(
    graph: &TraversalGraph<'_>,
    state: &mut TraversalState,
    max_hops: usize,
) {
    for _hop in 0..max_hops {
        let next_frontier = expand_frontier(graph, state);
        if !state.advance_to(next_frontier) {
            break;
        }
    }
}

fn expand_frontier(graph: &TraversalGraph<'_>, state: &mut TraversalState) -> Vec<String> {
    state
        .frontier
        .clone()
        .into_iter()
        .flat_map(|entity_name| expand_entity_name(graph, state, entity_name.as_str()))
        .collect()
}

fn expand_entity_name(
    graph: &TraversalGraph<'_>,
    state: &mut TraversalState,
    entity_name: &str,
) -> Vec<String> {
    if !state.visited.insert(entity_name.to_string()) {
        return Vec::new();
    }
    push_found_entity(graph, state, entity_name);
    bidirectional_neighbors(graph, &state.visited, entity_name)
}

fn push_found_entity(graph: &TraversalGraph<'_>, state: &mut TraversalState, entity_name: &str) {
    let Some(entity_id) = graph.entities_by_name.get(entity_name) else {
        return;
    };
    let Some(entity) = graph.entities.get(entity_id) else {
        return;
    };
    if !state
        .found_entities
        .iter()
        .any(|candidate| candidate.id == entity.id)
    {
        state.found_entities.push(entity.clone());
    }
}

fn bidirectional_neighbors(
    graph: &TraversalGraph<'_>,
    visited: &HashSet<String>,
    entity_name: &str,
) -> Vec<String> {
    outgoing_neighbors(graph, visited, entity_name)
        .into_iter()
        .chain(incoming_neighbors(graph, visited, entity_name))
        .collect()
}

fn outgoing_neighbors(
    graph: &TraversalGraph<'_>,
    visited: &HashSet<String>,
    entity_name: &str,
) -> Vec<String> {
    relation_targets(graph.outgoing.get(entity_name), &graph.relations, visited)
}

fn incoming_neighbors(
    graph: &TraversalGraph<'_>,
    visited: &HashSet<String>,
    entity_name: &str,
) -> Vec<String> {
    relation_sources(graph.incoming.get(entity_name), &graph.relations, visited)
}

fn relation_targets(
    relation_ids: Option<&HashSet<String>>,
    relations: &HashMap<String, Relation>,
    visited: &HashSet<String>,
) -> Vec<String> {
    relation_ids
        .into_iter()
        .flatten()
        .filter_map(|relation_id| relations.get(relation_id))
        .filter(|relation| !visited.contains(&relation.target))
        .map(|relation| relation.target.clone())
        .collect()
}

fn relation_sources(
    relation_ids: Option<&HashSet<String>>,
    relations: &HashMap<String, Relation>,
    visited: &HashSet<String>,
) -> Vec<String> {
    relation_ids
        .into_iter()
        .flatten()
        .filter_map(|relation_id| relations.get(relation_id))
        .filter(|relation| !visited.contains(&relation.source))
        .map(|relation| relation.source.clone())
        .collect()
}
