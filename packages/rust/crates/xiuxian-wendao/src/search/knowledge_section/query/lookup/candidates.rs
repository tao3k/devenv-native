use std::collections::HashMap;

use crate::search::ranking::{RetainedWindow, StreamingRerankTelemetry, trim_ranked_string_map};
use xiuxian_vector_store::EngineRecordBatch;

use super::error::KnowledgeSectionSearchError;
use super::helpers::{
    candidate_path_key, compare_candidates, engine_string_column, nullable_value, score_candidate,
};

const MIN_RETAINED_PATHS: usize = 128;
const RETAINED_PATH_MULTIPLIER: usize = 8;

#[derive(Debug, Clone)]
pub(crate) struct KnowledgeCandidate {
    pub(crate) id: String,
    pub(crate) path: String,
    pub(crate) stem: String,
    pub(crate) score: f64,
}

pub(crate) fn collect_candidates(
    batch: &EngineRecordBatch,
    query_text: &str,
    query_lower: &str,
    best_by_path: &mut HashMap<String, KnowledgeCandidate>,
    window: RetainedWindow,
    telemetry: &mut StreamingRerankTelemetry,
) -> Result<(), KnowledgeSectionSearchError> {
    telemetry.observe_batch(batch.num_rows());
    let id = engine_string_column(batch, "id")?;
    let path = engine_string_column(batch, "path")?;
    let stem = engine_string_column(batch, "stem")?;
    let title = engine_string_column(batch, "title")?;
    let best_section = engine_string_column(batch, "best_section")?;
    let search_text_folded = engine_string_column(batch, "search_text_folded")?;

    for row in 0..batch.num_rows() {
        let score = score_candidate(
            query_text,
            query_lower,
            stem.value(row),
            nullable_value(title, row),
            nullable_value(best_section, row),
            search_text_folded.value(row),
        );
        if score <= 0.0 {
            continue;
        }

        telemetry.observe_match();
        let candidate = KnowledgeCandidate {
            id: id.value(row).to_string(),
            path: path.value(row).to_string(),
            stem: stem.value(row).to_string(),
            score,
        };
        match best_by_path.get(candidate.path.as_str()) {
            Some(existing) if existing.score >= candidate.score => {}
            _ => {
                best_by_path.insert(candidate.path.clone(), candidate);
                telemetry.observe_working_set(best_by_path.len());
                if best_by_path.len() > window.threshold {
                    let before_len = best_by_path.len();
                    trim_ranked_string_map(
                        best_by_path,
                        window.target,
                        compare_candidates,
                        candidate_path_key,
                    );
                    telemetry.observe_trim(before_len, best_by_path.len());
                }
            }
        }
    }

    Ok(())
}

pub(crate) fn retained_window(limit: usize) -> RetainedWindow {
    RetainedWindow::new(limit, RETAINED_PATH_MULTIPLIER, MIN_RETAINED_PATHS)
}
