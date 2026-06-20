use chrono::Utc;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::search::{SearchQueryTelemetry, SearchQueryTelemetrySource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetainedWindow {
    pub(crate) target: usize,
    pub(crate) threshold: usize,
}

impl RetainedWindow {
    pub(crate) fn new(limit: usize, multiplier: usize, minimum: usize) -> Self {
        let target = limit.saturating_mul(multiplier).max(minimum);
        let threshold = target.saturating_mul(2);
        Self { target, threshold }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StreamingRerankSource {
    Scan,
}

impl From<StreamingRerankSource> for SearchQueryTelemetrySource {
    fn from(value: StreamingRerankSource) -> Self {
        match value {
            StreamingRerankSource::Scan => Self::Scan,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StreamingRerankTelemetry {
    batch_row_limit: Option<u64>,
    recall_limit_rows: Option<u64>,
    window: RetainedWindow,
    batch_count: u64,
    rows_scanned: u64,
    matched_rows: u64,
    peak_working_set_rows: u64,
    trim_count: u64,
    dropped_candidate_count: u64,
}

impl StreamingRerankTelemetry {
    pub(crate) fn new(
        window: RetainedWindow,
        batch_row_limit: Option<usize>,
        recall_limit_rows: Option<usize>,
    ) -> Self {
        Self {
            batch_row_limit: batch_row_limit.map(usize_to_u64_saturating),
            recall_limit_rows: recall_limit_rows.map(usize_to_u64_saturating),
            window,
            batch_count: 0,
            rows_scanned: 0,
            matched_rows: 0,
            peak_working_set_rows: 0,
            trim_count: 0,
            dropped_candidate_count: 0,
        }
    }

    pub(crate) fn observe_batch(&mut self, row_count: usize) {
        self.batch_count = self.batch_count.saturating_add(1);
        self.rows_scanned = self
            .rows_scanned
            .saturating_add(usize_to_u64_saturating(row_count));
    }

    pub(crate) fn observe_match(&mut self) {
        self.matched_rows = self.matched_rows.saturating_add(1);
    }

    pub(crate) fn observe_rejected_candidate(&mut self) {
        self.dropped_candidate_count = self.dropped_candidate_count.saturating_add(1);
    }

    pub(crate) fn observe_working_set(&mut self, row_count: usize) {
        self.peak_working_set_rows = self
            .peak_working_set_rows
            .max(usize_to_u64_saturating(row_count));
    }

    pub(crate) fn observe_trim(&mut self, before_len: usize, after_len: usize) {
        self.trim_count = self.trim_count.saturating_add(1);
        self.dropped_candidate_count =
            self.dropped_candidate_count
                .saturating_add(usize_to_u64_saturating(
                    before_len.saturating_sub(after_len),
                ));
        self.observe_working_set(after_len);
    }

    pub(crate) fn finish(
        self,
        source: StreamingRerankSource,
        scope: Option<String>,
        result_count: usize,
    ) -> SearchQueryTelemetry {
        SearchQueryTelemetry {
            captured_at: Utc::now().to_rfc3339(),
            scope,
            source: source.into(),
            batch_count: self.batch_count,
            rows_scanned: self.rows_scanned,
            matched_rows: self.matched_rows,
            result_count: usize_to_u64_saturating(result_count),
            batch_row_limit: self.batch_row_limit,
            recall_limit_rows: self.recall_limit_rows,
            working_set_budget_rows: usize_to_u64_saturating(self.window.target),
            trim_threshold_rows: usize_to_u64_saturating(self.window.threshold),
            peak_working_set_rows: self.peak_working_set_rows,
            trim_count: self.trim_count,
            dropped_candidate_count: self.dropped_candidate_count,
        }
    }
}

pub(crate) fn sort_by_rank<T>(items: &mut [T], compare: fn(&T, &T) -> Ordering) {
    items.sort_by(compare);
}

pub(crate) fn trim_ranked_vec<T>(
    items: &mut Vec<T>,
    retain_target: usize,
    compare: fn(&T, &T) -> Ordering,
) {
    sort_by_rank(items, compare);
    items.truncate(retain_target);
}

pub(crate) fn trim_ranked_string_map<T>(
    items: &mut HashMap<String, T>,
    retain_target: usize,
    compare: fn(&T, &T) -> Ordering,
    key_for: fn(&T) -> String,
) {
    let mut retained = items.drain().map(|(_, value)| value).collect::<Vec<_>>();
    sort_by_rank(&mut retained, compare);
    retained.truncate(retain_target);
    items.extend(retained.into_iter().map(|value| (key_for(&value), value)));
}

fn usize_to_u64_saturating(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
#[path = "../../tests/unit/search/ranking.rs"]
mod tests;
