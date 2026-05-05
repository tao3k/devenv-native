use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::models::{
    LinkGraphDirection, LinkGraphNeighbor, LinkGraphRelatedPprDiagnostics,
    LinkGraphRelatedPprOptions,
};
use std::collections::{HashSet, VecDeque};

impl LinkGraphIndex {
    /// Return the neighbor count for a note.
    #[must_use]
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
    pub fn neighbors(
        &self,
        stem_or_id: &str,
        direction: LinkGraphDirection,
        max_distance: usize,
        limit: usize,
    ) -> Vec<LinkGraphNeighbor> {
        let Some(start_id) = self
            .resolve_doc_id(stem_or_id)
            .map(std::string::ToString::to_string)
        else {
            return Vec::new();
        };

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut results = Vec::new();

        visited.insert(start_id.clone());
        queue.push_back((start_id.clone(), 0, LinkGraphDirection::Both));

        while let Some((current_id, distance, _)) = queue.pop_front() {
            if distance >= max_distance || results.len() >= limit {
                continue;
            }

            let next_distance = distance + 1;

            if (direction == LinkGraphDirection::Outgoing || direction == LinkGraphDirection::Both)
                && let Some(neighbors) = self.outgoing.get(&current_id)
            {
                for neighbor_id in neighbors {
                    if !visited.insert(neighbor_id.clone()) {
                        continue;
                    }
                    if let Some(doc) = self.docs_by_id.get(neighbor_id) {
                        results.push(LinkGraphNeighbor {
                            stem: doc.stem.clone(),
                            title: doc.title.clone(),
                            path: doc.path.clone(),
                            distance: next_distance,
                            direction: LinkGraphDirection::Outgoing,
                        });
                        queue.push_back((
                            neighbor_id.clone(),
                            next_distance,
                            LinkGraphDirection::Outgoing,
                        ));
                    }
                }
            }

            if (direction == LinkGraphDirection::Incoming || direction == LinkGraphDirection::Both)
                && let Some(neighbors) = self.incoming.get(&current_id)
            {
                for neighbor_id in neighbors {
                    if !visited.insert(neighbor_id.clone()) {
                        continue;
                    }
                    if let Some(doc) = self.docs_by_id.get(neighbor_id) {
                        results.push(LinkGraphNeighbor {
                            stem: doc.stem.clone(),
                            title: doc.title.clone(),
                            path: doc.path.clone(),
                            distance: next_distance,
                            direction: LinkGraphDirection::Incoming,
                        });
                        queue.push_back((
                            neighbor_id.clone(),
                            next_distance,
                            LinkGraphDirection::Incoming,
                        ));
                    }
                }
            }
        }

        results.sort_by(|a, b| {
            a.distance
                .cmp(&b.distance)
                .then_with(|| a.stem.cmp(&b.stem))
        });
        results.truncate(limit);
        results
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
