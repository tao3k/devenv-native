use crate::link_graph::index::{LinkGraphDirection, LinkGraphIndex};
use std::collections::{HashSet, VecDeque};

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search) fn collect_directional_ids(
        &self,
        seed_id: &str,
        direction: LinkGraphDirection,
        max_distance: usize,
    ) -> HashSet<String> {
        let bounded_distance = max_distance.max(1);
        let mut out: HashSet<String> = HashSet::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();

        visited.insert(seed_id.to_string());
        queue.push_back((seed_id.to_string(), 0));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= bounded_distance {
                continue;
            }
            let next_depth = depth + 1;

            if matches!(
                direction,
                LinkGraphDirection::Outgoing | LinkGraphDirection::Both
            ) && let Some(targets) = self.outgoing.get(&current)
            {
                for target in targets {
                    if target == seed_id {
                        continue;
                    }
                    if visited.insert(target.clone()) {
                        out.insert(target.clone());
                        queue.push_back((target.clone(), next_depth));
                    }
                }
            }

            if matches!(
                direction,
                LinkGraphDirection::Incoming | LinkGraphDirection::Both
            ) && let Some(sources) = self.incoming.get(&current)
            {
                for source in sources {
                    if source == seed_id {
                        continue;
                    }
                    if visited.insert(source.clone()) {
                        out.insert(source.clone());
                        queue.push_back((source.clone(), next_depth));
                    }
                }
            }
        }

        out
    }
}
