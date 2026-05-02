use crate::link_graph::index::{
    DEFAULT_MIN_SECTION_WORDS, DEFAULT_PER_DOC_SECTION_CAP, LinkGraphIndex, LinkGraphScope,
    LinkGraphSearchFilters, LinkGraphSearchOptions,
};

impl LinkGraphIndex {
    fn has_tag_filters(filters: &LinkGraphSearchFilters) -> bool {
        filters.tags.as_ref().is_some_and(|tags| {
            !tags.all.is_empty() || !tags.any.is_empty() || !tags.not_tags.is_empty()
        })
    }

    pub(in crate::link_graph::index::search) fn effective_scope(
        filters: &LinkGraphSearchFilters,
    ) -> LinkGraphScope {
        filters.scope.unwrap_or(LinkGraphScope::DocOnly)
    }

    pub(in crate::link_graph::index::search) fn effective_per_doc_section_cap(
        filters: &LinkGraphSearchFilters,
        scope: LinkGraphScope,
    ) -> usize {
        if let Some(cap) = filters.per_doc_section_cap {
            return cap.max(1);
        }
        if matches!(scope, LinkGraphScope::SectionOnly | LinkGraphScope::Mixed) {
            return DEFAULT_PER_DOC_SECTION_CAP;
        }
        1
    }

    pub(in crate::link_graph::index::search) fn effective_min_section_words(
        filters: &LinkGraphSearchFilters,
        scope: LinkGraphScope,
    ) -> usize {
        if let Some(min_words) = filters.min_section_words {
            return min_words;
        }
        if matches!(scope, LinkGraphScope::SectionOnly | LinkGraphScope::Mixed) {
            return DEFAULT_MIN_SECTION_WORDS;
        }
        0
    }

    pub(in crate::link_graph::index::search) fn effective_max_heading_level(
        filters: &LinkGraphSearchFilters,
    ) -> usize {
        filters.max_heading_level.unwrap_or(6).clamp(1, 6)
    }

    pub(in crate::link_graph::index::search) fn has_non_query_filters(
        options: &LinkGraphSearchOptions,
    ) -> bool {
        let filters = &options.filters;
        !filters.include_paths.is_empty()
            || !filters.exclude_paths.is_empty()
            || Self::has_tag_filters(filters)
            || Self::has_link_filter(filters.link_to.as_ref())
            || Self::has_link_filter(filters.linked_by.as_ref())
            || Self::has_related_filter(filters.related.as_ref())
            || !filters.mentions_of.is_empty()
            || !filters.mentioned_by_notes.is_empty()
            || filters.orphan
            || filters.tagless
            || filters.missing_backlink
            || filters.scope.is_some()
            || filters.max_heading_level.is_some()
            || filters.max_tree_hops.is_some()
            || filters.collapse_to_doc.is_some()
            || !filters.edge_types.is_empty()
            || filters.per_doc_section_cap.is_some()
            || filters.min_section_words.is_some()
            || options.created_after.is_some()
            || options.created_before.is_some()
            || options.modified_after.is_some()
            || options.modified_before.is_some()
    }
}
