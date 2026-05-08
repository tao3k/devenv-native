use crate::link_graph::index::search::context::{SearchExecutionContext, SearchRuntimePolicy};
use crate::link_graph::index::{
    LinkGraphIndex, SectionCandidate, section_tree_distance, token_match_ratio,
};
use std::cmp::Ordering;

impl LinkGraphIndex {
    pub(in crate::link_graph::index::search) fn section_candidates(
        &self,
        doc_id: &str,
        context: &SearchExecutionContext,
        runtime_policy: &SearchRuntimePolicy,
    ) -> Vec<SectionCandidate> {
        let Some(sections) = self.sections_by_doc.get(doc_id) else {
            return Vec::new();
        };
        let query = context.clean_query.as_str();
        let query_tokens = &context.query_tokens;

        let mut candidates: Vec<SectionCandidate> = Vec::new();
        for section in sections {
            if section.heading_level > 0 && section.heading_level > runtime_policy.max_heading_level
            {
                continue;
            }

            if runtime_policy.min_section_words > 0
                && section.section_text.split_whitespace().count()
                    < runtime_policy.min_section_words
            {
                continue;
            }

            let heading = if context.case_sensitive {
                section.heading_path.as_str()
            } else {
                section.heading_path_lower.as_str()
            };
            let body = if context.case_sensitive {
                section.section_text.as_str()
            } else {
                section.section_text_lower.as_str()
            };

            let mut score = if query.is_empty() { 1.0 } else { 0.0 };
            let mut reason = "section_filtered";

            if !query.is_empty() {
                if !heading.is_empty() && heading == query {
                    score = 1.0;
                    reason = "section_heading_exact";
                } else if !heading.is_empty() && heading.contains(query) {
                    score = 0.92;
                    reason = "section_heading_contains";
                } else if body.contains(query) {
                    score = 0.72;
                    reason = "section_body_contains";
                }

                if !query_tokens.is_empty() {
                    let heading_ratio = token_match_ratio(heading, query_tokens);
                    let body_ratio = token_match_ratio(body, query_tokens);
                    let token_score = (heading_ratio * 0.75 + body_ratio * 0.40).clamp(0.0, 1.0);
                    if token_score > score {
                        score = token_score;
                        reason = if heading_ratio > 0.0 {
                            "section_heading_token"
                        } else {
                            "section_body_token"
                        };
                    }
                }

                if score > 0.0 {
                    let heading_depth_boost = usize_to_f64_saturating(
                        6usize.saturating_sub(section.heading_level.min(6)),
                    ) * 0.01;
                    score = (score + heading_depth_boost).clamp(0.0, 1.0);
                }
            }

            if score <= 0.0 {
                continue;
            }
            candidates.push(SectionCandidate {
                heading_path: section.heading_path.clone(),
                score,
                reason,
            });
        }

        candidates.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
                .then(left.heading_path.cmp(&right.heading_path))
        });

        if let Some(max_hops) = runtime_policy.max_tree_hops
            && !query.is_empty()
            && let Some(seed_heading) = candidates.first().map(|row| row.heading_path.clone())
        {
            candidates.retain(|candidate| {
                section_tree_distance(seed_heading.as_str(), candidate.heading_path.as_str())
                    <= max_hops
            });
        }

        candidates
    }
}

fn usize_to_f64_saturating(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}
