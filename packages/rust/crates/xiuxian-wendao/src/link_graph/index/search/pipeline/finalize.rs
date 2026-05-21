use crate::link_graph::index::search::{
    LinkGraphHit, LinkGraphIndex, LinkGraphSearchOptions, ScoredSearchRow,
    context::SearchExecutionContext, deterministic_random_key, sort_hits,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::{HashMap, HashSet};

const PARALLEL_SEARCH_DOC_THRESHOLD: usize = 256;

impl LinkGraphIndex {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn collect_search_rows(
        &self,
        options: &LinkGraphSearchOptions,
        context: &SearchExecutionContext,
        graph_candidates: Option<&HashSet<String>>,
    ) -> Vec<ScoredSearchRow> {
        let runtime_policy = Self::resolve_search_runtime_policy(options, context);
        if self.docs_by_id.len() >= PARALLEL_SEARCH_DOC_THRESHOLD {
            return self
                .docs_by_id
                .par_iter()
                .flat_map_iter(|(_, doc)| {
                    self.evaluate_doc_rows(doc, options, context, graph_candidates, &runtime_policy)
                })
                .collect();
        }

        self.docs_by_id
            .values()
            .flat_map(|doc| {
                self.evaluate_doc_rows(doc, options, context, graph_candidates, &runtime_policy)
            })
            .collect()
    }

    pub(crate) fn finalize_search_rows(
        &self,
        mut rows: Vec<ScoredSearchRow>,
        options: &LinkGraphSearchOptions,
        bounded: usize,
        doc_boosts: Option<&HashMap<String, f64>>,
    ) -> Vec<LinkGraphHit> {
        let mut boosted_doc_ids: HashSet<String> = HashSet::new();
        if let Some(boosts) = doc_boosts
            && !boosts.is_empty()
        {
            for row in &mut rows {
                let doc_id = self
                    .resolve_doc_id_pub(&row.hit.path)
                    .or_else(|| self.resolve_doc_id_pub(&row.hit.stem));
                let Some(doc_id) = doc_id else {
                    continue;
                };
                let Some(boost) = boosts.get::<str>(doc_id) else {
                    continue;
                };
                if *boost <= 0.0 {
                    continue;
                }
                let bounded_boost = boost.clamp(0.0, 1.0);
                let bounded_score = row.hit.score.clamp(0.0, 1.0);
                row.hit.score =
                    (bounded_score + (1.0 - bounded_score) * bounded_boost).clamp(0.0, 1.0);
                let reason = row.hit.match_reason.get_or_insert_with(String::new);
                if !reason.contains("agentic_provisional") {
                    if !reason.is_empty() {
                        reason.push('+');
                    }
                    reason.push_str("agentic_provisional");
                }
                boosted_doc_ids.insert(doc_id.to_string());
            }

            for (doc_id, boost) in boosts {
                if *boost <= 0.0 || boosted_doc_ids.contains(doc_id.as_str()) {
                    continue;
                }
                let Some(doc) = self.get_doc(doc_id.as_str()) else {
                    continue;
                };
                let bounded_boost = boost.clamp(0.0, 1.0);
                let injected_score = (0.25 + bounded_boost * 0.5).clamp(0.0, 1.0);
                rows.push(ScoredSearchRow {
                    hit: LinkGraphHit {
                        stem: doc.stem.clone(),
                        title: doc.title.clone(),
                        path: doc.path.clone(),
                        doc_type: doc.doc_type.clone(),
                        tags: doc.tags.clone(),
                        score: injected_score,
                        best_section: None,
                        match_reason: Some("agentic_provisional_injection".to_string()),
                    },
                    created_ts: doc.created_ts,
                    modified_ts: doc.modified_ts,
                    word_count: doc.word_count,
                    random_key: deterministic_random_key(&doc.stem, &doc.path),
                });
            }
        }

        sort_hits(&mut rows, &options.sort_terms);
        rows.truncate(bounded);
        rows.into_iter().map(|row| row.hit).collect()
    }
}
