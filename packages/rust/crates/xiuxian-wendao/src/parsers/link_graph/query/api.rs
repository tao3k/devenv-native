//! `parsers::link_graph::query::api` owns Wendao link graph query api behavior.

use super::merge::merge_into_base;
use super::scan::parse_terms;
use super::state::ParsedDirectiveState;
use crate::link_graph::LinkGraphSearchOptions;

/// Parsed query payload used by search pipeline.
#[derive(Debug, Clone)]
pub struct ParsedLinkGraphQuery {
    /// Residual free-text query after directive extraction.
    pub query: String,
    /// Parsed/merged search options.
    pub options: LinkGraphSearchOptions,
    /// Optional limit override parsed from query directives.
    pub limit_override: Option<usize>,
    /// Optional direct-id short-circuit key parsed from `id:<value>`.
    pub direct_id: Option<String>,
}

/// Parse a user query into residual query text + merged options.
#[must_use]
pub fn parse_search_query(
    raw_query: &str,
    mut base: LinkGraphSearchOptions,
) -> ParsedLinkGraphQuery {
    let raw = raw_query.trim();
    if raw.is_empty() {
        return ParsedLinkGraphQuery {
            query: String::new(),
            options: base,
            limit_override: None,
            direct_id: None,
        };
    }

    let mut state = ParsedDirectiveState::default();
    let residual_terms = parse_terms(raw, &mut state);
    merge_into_base(&mut base, &residual_terms, &state);

    ParsedLinkGraphQuery {
        query: residual_terms.join(" ").trim().to_string(),
        options: base,
        limit_override: state.limit_override,
        direct_id: state.direct_id,
    }
}
