use crate::link_graph::index::{
    LinkGraphHit, LinkGraphIndex, LinkGraphScope, LinkGraphSearchOptions,
};
use crate::parsers::link_graph::query::{ParsedLinkGraphQuery, parse_search_query};
use std::collections::HashMap;

impl LinkGraphIndex {
    /// Parse query directives/options once and execute the resulting search plan.
    #[must_use]
    pub fn search_planned(
        &self,
        query: &str,
        limit: usize,
        base_options: LinkGraphSearchOptions,
    ) -> (ParsedLinkGraphQuery, Vec<LinkGraphHit>) {
        let parsed = parse_search_query(query, base_options);
        let effective_limit = parsed.limit_override.unwrap_or(limit);
        let rows = self.execute_search(&parsed.query, effective_limit, &parsed.options);
        (parsed, rows)
    }

    /// Execute query plan with explicit matching and sorting options.
    #[must_use]
    pub fn execute_search(
        &self,
        query: &str,
        limit: usize,
        options: &LinkGraphSearchOptions,
    ) -> Vec<LinkGraphHit> {
        self.execute_search_with_doc_boosts(query, limit, options, None)
    }

    /// Execute query plan with explicit matching/sorting options and
    /// optional agentic provisional doc-score boosts.
    #[must_use]
    pub(crate) fn execute_search_with_doc_boosts(
        &self,
        query: &str,
        limit: usize,
        options: &LinkGraphSearchOptions,
        doc_boosts: Option<&HashMap<String, f64>>,
    ) -> Vec<LinkGraphHit> {
        let Some(context) = Self::prepare_execution_context(query, limit, options) else {
            return Vec::new();
        };

        let graph_candidates = self.graph_filter_candidates(options);
        if context.raw_query.is_empty()
            && graph_candidates.is_none()
            && !Self::has_non_query_filters(options)
        {
            return Vec::new();
        }

        let scope = LinkGraphIndex::effective_scope(&options.filters);
        let structural_edges_enabled = LinkGraphIndex::allows_structural_edges(&options.filters);
        if matches!(scope, LinkGraphScope::SectionOnly) && !structural_edges_enabled {
            return Vec::new();
        }

        let rows = self.collect_search_rows(options, &context, graph_candidates.as_ref());
        self.finalize_search_rows(rows, options, context.bounded, doc_boosts)
    }
}
