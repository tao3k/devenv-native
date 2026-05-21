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
                push_directional_neighbors(
                    targets,
                    seed_id,
                    &mut visited,
                    &mut out,
                    &mut queue,
                    next_depth,
                );
            }

            if matches!(
                direction,
                LinkGraphDirection::Incoming | LinkGraphDirection::Both
            ) && let Some(sources) = self.incoming.get(&current)
            {
                push_directional_neighbors(
                    sources,
                    seed_id,
                    &mut visited,
                    &mut out,
                    &mut queue,
                    next_depth,
                );
            }
        }

        out
    }
}

fn push_directional_neighbors(
    neighbors: &HashSet<String>,
    seed_id: &str,
    visited: &mut HashSet<String>,
    out: &mut HashSet<String>,
    queue: &mut VecDeque<(String, usize)>,
    next_depth: usize,
) {
    neighbors
        .iter()
        .filter(|neighbor| neighbor.as_str() != seed_id)
        .filter(|neighbor| visited.insert((*neighbor).clone()))
        .cloned()
        .for_each(|neighbor| {
            out.insert(neighbor.clone());
            queue.push_back((neighbor, next_depth));
        });
}
