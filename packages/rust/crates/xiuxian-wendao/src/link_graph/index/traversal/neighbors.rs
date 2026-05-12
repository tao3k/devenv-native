use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::models::{
    LinkGraphDirection, LinkGraphNeighbor, LinkGraphRelatedPprDiagnostics,
    LinkGraphRelatedPprOptions,
};
use std::collections::{HashSet, VecDeque};

struct NeighborQueueEntry {
    doc_id: String,
    distance: usize,
}

struct NeighborSearchState {
    visited: HashSet<String>,
    queue: VecDeque<NeighborQueueEntry>,
    results: Vec<LinkGraphNeighbor>,
}

impl LinkGraphIndex {
    /// Return the neighbor count for a note.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn neighbor_count(&self, stem_or_id: &str, direction: LinkGraphDirection) -> usize {
        let Some(doc_id) = self.resolve_doc_id(stem_or_id) else {
            return 0;
        };
        match direction {
            LinkGraphDirection::Outgoing => self.outgoing.get(doc_id).map_or(0, HashSet::len),
            LinkGraphDirection::Incoming => self.incoming.get(doc_id).map_or(0, HashSet::len),
            LinkGraphDirection::Both => {
                let out_set = self.outgoing.get(doc_id);
                let in_set = self.incoming.get(doc_id);
                match (out_set, in_set) {
                    (Some(out), Some(in_)) => out.union(in_).count(),
                    (Some(out), None) => out.len(),
                    (None, Some(in_)) => in_.len(),
                    (None, None) => 0,
                }
            }
        }
    }

    /// Return neighbors for a note within a specific hop distance.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn neighbors(
        &self,
        stem_or_id: &str,
        direction: LinkGraphDirection,
        max_distance: usize,
        limit: usize,
    ) -> Vec<LinkGraphNeighbor> {
        let Some(start_id) = self.neighbor_start_id(stem_or_id) else {
            return Vec::new();
        };
        let results = self.collect_neighbors(start_id, direction, max_distance, limit);
        ranked_neighbors(results, limit)
    }

    fn neighbor_start_id(&self, stem_or_id: &str) -> Option<String> {
        self.resolve_doc_id(stem_or_id)
            .map(std::string::ToString::to_string)
    }

    fn collect_neighbors(
        &self,
        start_id: String,
        direction: LinkGraphDirection,
        max_distance: usize,
        limit: usize,
    ) -> Vec<LinkGraphNeighbor> {
        let mut state = NeighborSearchState::new(start_id);
        while let Some(entry) = state.queue.pop_front() {
            if state.should_skip_entry(entry.distance, max_distance, limit) {
                continue;
            }
            self.visit_neighbor_step(&mut state, &entry, direction);
        }
        state.results
    }

    fn visit_neighbor_step(
        &self,
        state: &mut NeighborSearchState,
        entry: &NeighborQueueEntry,
        direction: LinkGraphDirection,
    ) {
        for edge_direction in active_neighbor_directions(direction) {
            self.visit_neighbor_direction(state, entry, edge_direction);
        }
    }

    fn visit_neighbor_direction(
        &self,
        state: &mut NeighborSearchState,
        entry: &NeighborQueueEntry,
        direction: LinkGraphDirection,
    ) {
        let Some(neighbor_ids) = self.neighbor_ids_for_direction(&entry.doc_id, direction) else {
            return;
        };
        let next_distance = entry.distance + 1;
        for neighbor_id in neighbor_ids {
            self.visit_neighbor_id(state, neighbor_id, next_distance, direction);
        }
    }

    fn neighbor_ids_for_direction(
        &self,
        doc_id: &str,
        direction: LinkGraphDirection,
    ) -> Option<&HashSet<String>> {
        match direction {
            LinkGraphDirection::Outgoing => self.outgoing.get(doc_id),
            LinkGraphDirection::Incoming => self.incoming.get(doc_id),
            LinkGraphDirection::Both => None,
        }
    }

    fn visit_neighbor_id(
        &self,
        state: &mut NeighborSearchState,
        neighbor_id: &str,
        distance: usize,
        direction: LinkGraphDirection,
    ) {
        if !state.visited.insert(neighbor_id.to_string()) {
            return;
        }
        let Some(doc) = self.docs_by_id.get(neighbor_id) else {
            return;
        };
        state.results.push(LinkGraphNeighbor {
            stem: doc.stem.clone(),
            title: doc.title.clone(),
            path: doc.path.clone(),
            distance,
            direction,
        });
        state.queue.push_back(NeighborQueueEntry {
            doc_id: neighbor_id.to_string(),
            distance,
        });
    }

    /// Find related notes from explicit seed notes and return PPR diagnostics.
    #[must_use]
    pub fn related_from_seeds_with_diagnostics(
        &self,
        seeds: &[String],
        max_distance: usize,
        limit: usize,
        ppr: Option<&LinkGraphRelatedPprOptions>,
    ) -> (
        Vec<LinkGraphNeighbor>,
        Option<LinkGraphRelatedPprDiagnostics>,
    ) {
        let seed_ids = self.resolve_doc_ids(seeds);
        if seed_ids.is_empty() {
            return (Vec::new(), None);
        }
        let Some(computation) = self.related_ppr_compute(&seed_ids, max_distance.max(1), ppr)
        else {
            return (Vec::new(), None);
        };
        (
            self.build_related_neighbors_from_ranked(computation.ranked_doc_ids, limit),
            Some(computation.diagnostics),
        )
    }

    fn build_related_neighbors_from_ranked(
        &self,
        ranked: Vec<(String, usize, f64)>,
        limit: usize,
    ) -> Vec<LinkGraphNeighbor> {
        ranked
            .into_iter()
            .take(limit)
            .filter_map(|(doc_id, distance, _score)| {
                let doc = self.docs_by_id.get(&doc_id)?;
                Some(LinkGraphNeighbor {
                    stem: doc.stem.clone(),
                    title: doc.title.clone(),
                    path: doc.path.clone(),
                    distance,
                    direction: LinkGraphDirection::Both,
                })
            })
            .collect()
    }
}

impl NeighborSearchState {
    fn new(start_id: String) -> Self {
        let mut visited = HashSet::new();
        visited.insert(start_id.clone());
        let mut queue = VecDeque::new();
        queue.push_back(NeighborQueueEntry {
            doc_id: start_id,
            distance: 0,
        });
        Self {
            visited,
            queue,
            results: Vec::new(),
        }
    }

    fn should_skip_entry(&self, distance: usize, max_distance: usize, limit: usize) -> bool {
        distance >= max_distance || self.results.len() >= limit
    }
}

fn active_neighbor_directions(direction: LinkGraphDirection) -> Vec<LinkGraphDirection> {
    match direction {
        LinkGraphDirection::Outgoing => vec![LinkGraphDirection::Outgoing],
        LinkGraphDirection::Incoming => vec![LinkGraphDirection::Incoming],
        LinkGraphDirection::Both => {
            vec![LinkGraphDirection::Outgoing, LinkGraphDirection::Incoming]
        }
    }
}

fn ranked_neighbors(mut results: Vec<LinkGraphNeighbor>, limit: usize) -> Vec<LinkGraphNeighbor> {
    results.sort_by(|a, b| {
        a.distance
            .cmp(&b.distance)
            .then_with(|| a.stem.cmp(&b.stem))
    });
    results.truncate(limit);
    results
}
