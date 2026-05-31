use std::collections::HashMap;

use super::model::RecallCandidate;
use crate::orgize::read_model::model::AgentOrgTaskListRow;
use crate::orgize::read_model::section_lens::TaskSectionLens;

#[derive(Debug, Clone)]
pub(super) struct TemporalRecallContext {
    pub(super) oldest_modified_unix_ms: u64,
    pub(super) newest_modified_unix_ms: u64,
    pub(super) source_line_spans: HashMap<String, (u64, u64)>,
}

impl TemporalRecallContext {
    pub(super) fn from_candidates(candidates: &[RecallCandidate<'_>]) -> Self {
        let modified_times = candidates
            .iter()
            .map(|candidate| candidate.row.source_modified_unix_ms)
            .filter(|modified| *modified > 0)
            .collect::<Vec<_>>();
        let mut source_line_spans = HashMap::<String, (u64, u64)>::new();
        for candidate in candidates {
            let entry = source_line_spans
                .entry(candidate.row.source_path.clone())
                .or_insert((candidate.row.source_line, candidate.row.source_line));
            entry.0 = entry.0.min(candidate.row.source_line);
            entry.1 = entry.1.max(candidate.row.source_line);
        }
        Self {
            oldest_modified_unix_ms: modified_times.iter().min().copied().unwrap_or_default(),
            newest_modified_unix_ms: modified_times.iter().max().copied().unwrap_or_default(),
            source_line_spans,
        }
    }

    pub(super) fn modified_recency_bonus(&self, row: &AgentOrgTaskListRow) -> f32 {
        let mut bonus = if let Some(relative) = self.modified_relative_position(row) {
            0.02 + (0.06 * relative)
        } else {
            absolute_modified_recency_bonus(self.modified_age_ms(row))
        };
        if let Some(line_position) = self.source_line_relative_position(row) {
            bonus += 0.035 * line_position;
        }
        bonus
    }

    pub(super) fn source_line_relative_position(&self, row: &AgentOrgTaskListRow) -> Option<f32> {
        let (min_line, max_line) = self
            .source_line_spans
            .get(row.source_path.as_str())
            .copied()?;
        if min_line == max_line {
            return None;
        }
        let span = max_line.saturating_sub(min_line);
        if span == 0 {
            return None;
        }
        relative_position(row.source_line.saturating_sub(min_line), span)
    }

    pub(super) fn modified_relative_position(&self, row: &AgentOrgTaskListRow) -> Option<f32> {
        if self.oldest_modified_unix_ms == 0
            || self.newest_modified_unix_ms == 0
            || self.oldest_modified_unix_ms == self.newest_modified_unix_ms
            || row.source_modified_unix_ms == 0
        {
            return None;
        }
        let span = self
            .newest_modified_unix_ms
            .saturating_sub(self.oldest_modified_unix_ms);
        if span == 0 {
            return None;
        }
        let position = row
            .source_modified_unix_ms
            .saturating_sub(self.oldest_modified_unix_ms);
        relative_position(position, span)
    }

    pub(super) fn modified_age_ms(&self, row: &AgentOrgTaskListRow) -> Option<u64> {
        if self.newest_modified_unix_ms == 0 || row.source_modified_unix_ms == 0 {
            return None;
        }
        Some(
            self.newest_modified_unix_ms
                .saturating_sub(row.source_modified_unix_ms),
        )
    }
}

pub(super) fn absolute_modified_recency_bonus(age: Option<u64>) -> f32 {
    match age {
        Some(age) if age <= minutes(5) => 0.08,
        Some(age) if age <= hours(1) => 0.06,
        Some(age) if age <= hours(6) => 0.045,
        Some(age) if age <= hours(24) => 0.03,
        Some(age) if age <= days(7) => 0.015,
        None | Some(_) => 0.0,
    }
}

pub(super) fn relative_position(position: u64, span: u64) -> Option<f32> {
    if span == 0 {
        return None;
    }
    let scaled = position.saturating_mul(1_000).checked_div(span)?;
    let clamped = u16::try_from(scaled.min(1_000)).ok()?;
    Some(f32::from(clamped) / 1_000.0)
}

pub(super) const fn minutes(value: u64) -> u64 {
    value * 60 * 1_000
}

pub(super) const fn hours(value: u64) -> u64 {
    minutes(value * 60)
}

pub(super) const fn days(value: u64) -> u64 {
    hours(value * 24)
}

pub(super) fn task_row_section_lens(row: &AgentOrgTaskListRow) -> Option<TaskSectionLens> {
    let source = std::fs::read_to_string(row.source_path.as_str()).ok()?;
    let start = usize::try_from(row.source_range_start).ok()?;
    let end = usize::try_from(row.source_range_end).ok()?;
    let section = source.get(start..end)?;
    Some(TaskSectionLens::from_section(section))
}
