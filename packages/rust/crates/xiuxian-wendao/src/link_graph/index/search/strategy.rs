use crate::link_graph::index::{
    LinkGraphDocument, LinkGraphIndex, LinkGraphMatchStrategy, LinkGraphScope,
    LinkGraphSearchOptions, SECTION_AGGREGATION_BETA, SectionCandidate, SectionMatch,
    WEIGHT_FTS_LEXICAL, WEIGHT_FTS_PATH, WEIGHT_FTS_SECTION, WEIGHT_PATH_FUZZY_PATH,
    WEIGHT_PATH_FUZZY_SECTION, score_document, score_document_exact, score_document_regex,
};
use regex::Regex;

pub(super) struct StrategyScoreInputs<'a> {
    pub(super) raw_query: &'a str,
    pub(super) clean_query: &'a str,
    pub(super) query_tokens: &'a [String],
    pub(super) scope: LinkGraphScope,
    pub(super) collapse_to_doc: bool,
    pub(super) section_candidates: &'a [SectionCandidate],
    pub(super) section_match: Option<&'a SectionMatch>,
    pub(super) section_score: f64,
    pub(super) path_score: f64,
    pub(super) semantic_edges_enabled: bool,
    pub(super) regex: Option<&'a Regex>,
}

impl LinkGraphIndex {
    pub(super) fn score_doc_for_strategy(
        &self,
        doc: &LinkGraphDocument,
        options: &LinkGraphSearchOptions,
        inputs: &StrategyScoreInputs<'_>,
    ) -> (f64, String) {
        let (mut doc_score, mut doc_reason) = match options.match_strategy {
            LinkGraphMatchStrategy::Fts if !inputs.raw_query.is_empty() => {
                let lexical = score_document(
                    doc,
                    inputs.clean_query,
                    inputs.query_tokens,
                    options.case_sensitive,
                );
                let blended = (lexical * WEIGHT_FTS_LEXICAL
                    + inputs.section_score * WEIGHT_FTS_SECTION
                    + inputs.path_score * WEIGHT_FTS_PATH)
                    .max(lexical);
                let reason = if let Some(section) = inputs.section_match {
                    format!("fts+{}", section.reason)
                } else {
                    "fts".to_string()
                };
                (blended, reason)
            }
            LinkGraphMatchStrategy::PathFuzzy if !inputs.raw_query.is_empty() => {
                let base = inputs.path_score.max(inputs.section_score);
                let blended = if base > 0.0 {
                    (inputs.path_score * WEIGHT_PATH_FUZZY_PATH
                        + inputs.section_score * WEIGHT_PATH_FUZZY_SECTION)
                        .max(base)
                } else {
                    0.0
                };
                let reason = if let Some(section) = inputs.section_match {
                    format!("path_fuzzy+{}", section.reason)
                } else {
                    "path_fuzzy".to_string()
                };
                (blended, reason)
            }
            LinkGraphMatchStrategy::Exact if !inputs.raw_query.is_empty() => (
                score_document_exact(doc, inputs.clean_query, options.case_sensitive),
                "exact".to_string(),
            ),
            LinkGraphMatchStrategy::Re if !inputs.raw_query.is_empty() => (
                inputs
                    .regex
                    .map_or(0.0, |compiled| score_document_regex(doc, compiled)),
                "regex".to_string(),
            ),
            _ => (1.0, "filtered".to_string()),
        };

        if matches!(inputs.scope, LinkGraphScope::Mixed)
            && inputs.collapse_to_doc
            && !inputs.section_candidates.is_empty()
        {
            let max_section = inputs
                .section_candidates
                .first()
                .map_or(0.0, |row| row.score);
            let section_tail_sum = inputs
                .section_candidates
                .iter()
                .skip(1)
                .map(|row| row.score)
                .sum::<f64>();
            let aggregated =
                (max_section + SECTION_AGGREGATION_BETA * section_tail_sum).clamp(0.0, 1.0);
            if aggregated > doc_score {
                doc_score = aggregated;
                doc_reason.push_str("+section_agg");
            }
        }

        if inputs.semantic_edges_enabled
            && !inputs.raw_query.is_empty()
            && matches!(
                options.match_strategy,
                LinkGraphMatchStrategy::Fts | LinkGraphMatchStrategy::PathFuzzy
            )
            && doc_score > 0.0
        {
            let boosted = self.apply_graph_rank_boost(&doc.id, doc_score);
            if boosted > doc_score {
                doc_reason.push_str("+graph_rank");
            }
            doc_score = boosted;
        }

        (doc_score, doc_reason)
    }
}
