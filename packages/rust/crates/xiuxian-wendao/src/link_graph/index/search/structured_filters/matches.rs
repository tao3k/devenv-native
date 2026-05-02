use crate::link_graph::index::search::context::SearchExecutionContext;
use crate::link_graph::index::{LinkGraphDocument, LinkGraphIndex, LinkGraphSearchOptions};

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search) fn matches_temporal_filters(
        doc: &LinkGraphDocument,
        options: &LinkGraphSearchOptions,
    ) -> bool {
        if let Some(created_after) = options.created_after
            && doc.created_ts.is_none_or(|ts| ts < created_after)
        {
            return false;
        }
        if let Some(created_before) = options.created_before
            && doc.created_ts.is_none_or(|ts| ts > created_before)
        {
            return false;
        }
        if let Some(modified_after) = options.modified_after
            && doc.modified_ts.is_none_or(|ts| ts < modified_after)
        {
            return false;
        }
        if let Some(modified_before) = options.modified_before
            && doc.modified_ts.is_none_or(|ts| ts > modified_before)
        {
            return false;
        }
        true
    }

    pub(in crate::link_graph::index::search) fn matches_structured_filters(
        &self,
        doc: &LinkGraphDocument,
        options: &LinkGraphSearchOptions,
        context: &SearchExecutionContext,
    ) -> bool {
        if !Self::matches_path_filters(doc, &context.include_paths, &context.exclude_paths) {
            return false;
        }

        if !Self::matches_tag_filters(
            doc,
            options,
            &context.tag_all,
            &context.tag_any,
            &context.tag_not,
        ) {
            return false;
        }

        if !self.matches_graph_state_filters(doc, options, &context.mention_filters) {
            return false;
        }

        true
    }
}
