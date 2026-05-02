use crate::link_graph::agentic::LinkGraphSuggestedLink;
use crate::link_graph::index::{LinkGraphIndex, normalize_with_case, tokenize};
use std::collections::HashMap;

const AGENTIC_PROVISIONAL_MIN_BOOST: f64 = 0.08;
const AGENTIC_PROVISIONAL_MAX_BOOST: f64 = 0.35;
const AGENTIC_PROVISIONAL_CONFIDENCE_SCALE: f64 = 0.27;

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search::plan) fn build_provisional_doc_boosts(
        &self,
        query: &str,
        case_sensitive: bool,
        rows: &[LinkGraphSuggestedLink],
    ) -> HashMap<String, f64> {
        if rows.is_empty() {
            return HashMap::new();
        }
        let clean_query = normalize_with_case(query.trim(), case_sensitive);
        let query_tokens = tokenize(query, case_sensitive);
        let mut boosts: HashMap<String, f64> = HashMap::new();

        for row in rows {
            if !Self::suggested_link_matches_query(row, &clean_query, &query_tokens, case_sensitive)
            {
                continue;
            }
            let boost = Self::suggested_link_boost(row.confidence);
            for endpoint in [&row.source_id, &row.target_id] {
                let Some(doc_id) = self.resolve_doc_id(endpoint) else {
                    continue;
                };
                let entry = boosts.entry(doc_id.to_string()).or_insert(0.0);
                *entry =
                    (*entry + (1.0 - *entry) * boost).clamp(0.0, AGENTIC_PROVISIONAL_MAX_BOOST);
            }
        }
        boosts
    }

    fn suggested_link_matches_query(
        row: &LinkGraphSuggestedLink,
        clean_query: &str,
        query_tokens: &[String],
        case_sensitive: bool,
    ) -> bool {
        let haystack = normalize_with_case(
            &format!(
                "{} {} {} {}",
                row.source_id, row.target_id, row.relation, row.evidence
            ),
            case_sensitive,
        );
        if clean_query.is_empty() {
            return true;
        }
        if haystack.contains(clean_query) {
            return true;
        }
        query_tokens
            .iter()
            .any(|token| !token.is_empty() && haystack.contains(token))
    }

    fn suggested_link_boost(confidence: f64) -> f64 {
        let bounded_confidence = confidence.clamp(0.0, 1.0);
        (AGENTIC_PROVISIONAL_MIN_BOOST + bounded_confidence * AGENTIC_PROVISIONAL_CONFIDENCE_SCALE)
            .clamp(AGENTIC_PROVISIONAL_MIN_BOOST, AGENTIC_PROVISIONAL_MAX_BOOST)
    }
}
