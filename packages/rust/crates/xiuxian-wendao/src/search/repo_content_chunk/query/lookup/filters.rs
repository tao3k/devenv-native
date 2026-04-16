use std::collections::HashSet;

use crate::gateway::studio::types::SearchHit;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RepoContentChunkSearchFilters {
    pub(crate) path_prefixes: HashSet<String>,
    pub(crate) filename_filters: HashSet<String>,
    pub(crate) title_filters: HashSet<String>,
    pub(crate) tag_filters: HashSet<String>,
}

impl RepoContentChunkSearchFilters {
    pub(crate) fn retain_matching_hits(&self, hits: &mut Vec<SearchHit>) {
        if self.tag_filters.is_empty() {
            return;
        }

        hits.retain(|hit| self.matches_hit(hit));
    }

    fn matches_hit(&self, hit: &SearchHit) -> bool {
        self.matches_tag_filters(hit)
    }

    fn matches_tag_filters(&self, hit: &SearchHit) -> bool {
        if self.tag_filters.is_empty() {
            return true;
        }

        let normalized_tags = hit
            .tags
            .iter()
            .map(|tag| tag.to_ascii_lowercase())
            .collect::<HashSet<_>>();
        self.tag_filters
            .iter()
            .map(|filter| filter.to_ascii_lowercase())
            .any(|filter| normalized_tags.contains(&filter))
    }
}
