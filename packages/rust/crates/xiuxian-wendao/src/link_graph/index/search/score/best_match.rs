use crate::link_graph::index::{LinkGraphIndex, SectionCandidate, SectionMatch};

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search) fn best_section_match(
        candidates: &[SectionCandidate],
    ) -> Option<SectionMatch> {
        let best = candidates.first()?;
        Some(SectionMatch {
            score: best.score,
            heading_path: if best.heading_path.trim().is_empty() {
                None
            } else {
                Some(best.heading_path.clone())
            },
            reason: best.reason,
        })
    }
}
