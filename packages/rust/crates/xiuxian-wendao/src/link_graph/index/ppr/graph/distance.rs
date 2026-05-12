use crate::link_graph::index::{LinkGraphDocument, LinkGraphIndex, doc_sort_key};
use std::collections::{HashMap, HashSet, VecDeque};

impl LinkGraphIndex {
    pub(in crate::link_graph::index) fn collect_bidirectional_distance_map(
        &self,
        seed_ids: &HashSet<String>,
        max_distance: usize,
    ) -> HashMap<String, usize> {
        let bounded_distance = max_distance.max(1);
        let mut distances: HashMap<String, usize> = HashMap::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        for seed_id in seed_ids {
            if self.docs_by_id.contains_key(seed_id) {
                distances.insert(seed_id.clone(), 0);
                queue.push_back(seed_id.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            let Some(depth) = distances.get(&current).copied() else {
                continue;
            };
            if depth >= bounded_distance {
                continue;
            }
            let next_depth = depth + 1;

            enqueue_bidirectional_neighbors(
                self.outgoing.get(&current),
                &self.docs_by_id,
                &mut distances,
                &mut queue,
                next_depth,
            );
            enqueue_bidirectional_neighbors(
                self.incoming.get(&current),
                &self.docs_by_id,
                &mut distances,
                &mut queue,
                next_depth,
            );
        }

        distances
    }

    pub(in crate::link_graph::index) fn sort_doc_ids_for_runtime(&self, doc_ids: &mut [String]) {
        doc_ids.sort_by(|left, right| {
            match (self.docs_by_id.get(left), self.docs_by_id.get(right)) {
                (Some(a), Some(b)) => doc_sort_key(a).cmp(&doc_sort_key(b)),
                _ => left.cmp(right),
            }
        });
    }

    pub(in crate::link_graph::index) fn build_graph_nodes_for_related_ppr(
        &self,
        horizon_distances: &HashMap<String, usize>,
        restrict_to_horizon: bool,
    ) -> Vec<String> {
        let mut graph_nodes: Vec<String> = if restrict_to_horizon {
            horizon_distances.keys().cloned().collect()
        } else {
            self.docs_by_id.keys().cloned().collect()
        };
        self.sort_doc_ids_for_runtime(&mut graph_nodes);
        graph_nodes
    }

    pub(in crate::link_graph::index) fn candidate_count_from_horizon(
        horizon_distances: &HashMap<String, usize>,
        seed_ids: &HashSet<String>,
    ) -> usize {
        horizon_distances
            .keys()
            .filter(|doc_id| !seed_ids.contains(*doc_id))
            .count()
    }

    pub(in crate::link_graph::index) fn trim_horizon_candidates(
        &self,
        horizon_distances: &HashMap<String, usize>,
        seed_ids: &HashSet<String>,
        max_candidates: usize,
    ) -> HashMap<String, usize> {
        let mut kept: HashMap<String, usize> = horizon_distances
            .iter()
            .filter(|(doc_id, _)| seed_ids.contains(*doc_id))
            .map(|(doc_id, distance)| (doc_id.clone(), *distance))
            .collect();

        let mut candidates: Vec<(String, usize)> = horizon_distances
            .iter()
            .filter(|(doc_id, _)| !seed_ids.contains(*doc_id))
            .map(|(doc_id, distance)| (doc_id.clone(), *distance))
            .collect();

        candidates.sort_by(|left, right| {
            left.1.cmp(&right.1).then_with(|| {
                match (self.docs_by_id.get(&left.0), self.docs_by_id.get(&right.0)) {
                    (Some(a), Some(b)) => doc_sort_key(a).cmp(&doc_sort_key(b)),
                    _ => left.0.cmp(&right.0),
                }
            })
        });

        for (doc_id, distance) in candidates.into_iter().take(max_candidates.max(1)) {
            kept.insert(doc_id, distance);
        }
        kept
    }
}

fn enqueue_bidirectional_neighbors(
    neighbors: Option<&HashSet<String>>,
    docs_by_id: &HashMap<String, LinkGraphDocument>,
    distances: &mut HashMap<String, usize>,
    queue: &mut VecDeque<String>,
    next_depth: usize,
) {
    let pending_neighbors = neighbors
        .into_iter()
        .flatten()
        .filter(|neighbor| docs_by_id.contains_key(*neighbor))
        .filter(|neighbor| {
            distances
                .get(*neighbor)
                .is_none_or(|existing| next_depth < *existing)
        })
        .cloned()
        .collect::<Vec<_>>();
    pending_neighbors.into_iter().for_each(|neighbor| {
        distances.insert(neighbor.clone(), next_depth);
        queue.push_back(neighbor);
    });
}
