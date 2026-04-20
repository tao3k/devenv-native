use std::cmp::Ordering;

use xiuxian_db_store::EngineRecordBatch;

use crate::gateway::studio::search::score_reference_hit;
use crate::search::ranking::{RetainedWindow, StreamingRerankTelemetry, trim_ranked_vec};
use crate::search::reference_occurrence::ReferenceOccurrenceSearchError;

use super::helpers::{string_column, u64_column};

#[derive(Debug)]
pub(super) struct ReferenceOccurrenceCandidate {
    pub(super) id: String,
    pub(super) score: f64,
    pub(super) path: String,
    pub(super) line: usize,
    pub(super) column: usize,
}

pub(super) fn collect_candidates(
    batch: &EngineRecordBatch,
    query: &str,
    candidates: &mut Vec<ReferenceOccurrenceCandidate>,
    window: RetainedWindow,
    telemetry: &mut StreamingRerankTelemetry,
) -> Result<(), ReferenceOccurrenceSearchError> {
    telemetry.observe_batch(batch.num_rows());
    let id = string_column(batch, "id")?;
    let path = string_column(batch, "path")?;
    let line = u64_column(batch, "line")?;
    let column = u64_column(batch, "column")?;
    let line_text = string_column(batch, "line_text")?;

    for row in 0..batch.num_rows() {
        let score = score_reference_hit(line_text.value(row), query);
        if score <= 0.0 {
            continue;
        }

        telemetry.observe_match();
        candidates.push(ReferenceOccurrenceCandidate {
            id: id.value(row).to_string(),
            score,
            path: path.value(row).to_string(),
            line: usize::try_from(line.value(row)).unwrap_or(usize::MAX),
            column: usize::try_from(column.value(row)).unwrap_or(usize::MAX),
        });
        telemetry.observe_working_set(candidates.len());
        if candidates.len() > window.threshold {
            let before_len = candidates.len();
            trim_ranked_vec(candidates, window.target, compare_candidates);
            telemetry.observe_trim(before_len, candidates.len());
        }
    }

    Ok(())
}

pub(super) fn compare_candidates(
    left: &ReferenceOccurrenceCandidate,
    right: &ReferenceOccurrenceCandidate,
) -> Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.path.cmp(&right.path))
        .then_with(|| left.line.cmp(&right.line))
        .then_with(|| left.column.cmp(&right.column))
}
