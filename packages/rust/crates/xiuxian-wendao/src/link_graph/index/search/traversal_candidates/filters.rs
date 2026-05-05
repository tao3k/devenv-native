use crate::link_graph::index::{
    LinkGraphDirection, LinkGraphIndex, LinkGraphLinkFilter, LinkGraphRelatedFilter,
};
use std::collections::HashSet;

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search) fn combine_candidates(
        current: Option<HashSet<String>>,
        incoming: HashSet<String>,
    ) -> HashSet<String> {
        match current {
            None => incoming,
            Some(existing) => existing.intersection(&incoming).cloned().collect(),
        }
    }

    pub(in crate::link_graph::index::search) fn collect_link_filter_candidates(
        &self,
        filter: &LinkGraphLinkFilter,
        direction: LinkGraphDirection,
        universe: &HashSet<String>,
    ) -> HashSet<String> {
        let seed_ids = self.resolve_doc_ids(&filter.seeds);
        let max_distance = if filter.recursive {
            filter.max_distance.unwrap_or(2).max(1)
        } else {
            1
        };
        let mut matches: HashSet<String> = HashSet::new();
        for seed_id in seed_ids {
            matches.extend(self.collect_directional_ids(&seed_id, direction, max_distance));
        }
        if filter.negate {
            universe.difference(&matches).cloned().collect()
        } else {
            matches
        }
    }

    pub(in crate::link_graph::index::search) fn collect_related_filter_candidates(
        &self,
        filter: &LinkGraphRelatedFilter,
    ) -> HashSet<String> {
        let seed_ids = self.resolve_doc_ids(&filter.seeds);
        if seed_ids.is_empty() {
            return HashSet::new();
        }
        let max_distance = filter.max_distance.unwrap_or(2).max(1);
        self.related_ppr_ranked_doc_ids(&seed_ids, max_distance, filter.ppr.as_ref())
            .into_iter()
            .map(|(doc_id, _distance, _score)| doc_id)
            .collect()
    }

    pub(in crate::link_graph::index::search) fn collect_mentioned_by_note_candidates(
        &self,
        seeds: &[String],
    ) -> HashSet<String> {
        let seed_ids = self.resolve_doc_ids(seeds);
        let mut matches: HashSet<String> = HashSet::new();
        for seed_id in seed_ids {
            matches.extend(self.collect_directional_ids(&seed_id, LinkGraphDirection::Outgoing, 1));
        }
        matches
    }
}
