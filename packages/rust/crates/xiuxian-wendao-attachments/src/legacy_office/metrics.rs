//! Legacy Office parser-quality metrics.

use super::LegacyOfficeFormat;

/// Lightweight parser-quality counters for legacy Office projections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyOfficeQualityMetrics {
    /// Plain-text character count.
    pub text_char_count: usize,
    /// Markdown character count.
    pub markdown_char_count: usize,
    /// Plain-text line count after parser normalization.
    pub line_count: usize,
    /// Non-empty plain-text line count after parser normalization.
    pub non_empty_line_count: usize,
    /// Lines that preserve explicit tab-delimited cell boundaries.
    pub tab_delimited_row_count: usize,
    /// Maximum visible columns in the plain-text projection.
    pub max_column_count: usize,
    /// Fenced code blocks in the Markdown projection.
    pub markdown_fenced_block_count: usize,
}

impl LegacyOfficeQualityMetrics {
    /// Returns true when an XLS projection preserved at least one tab boundary.
    #[must_use]
    pub fn has_tabular_boundary_signal(&self, format: LegacyOfficeFormat) -> bool {
        format == LegacyOfficeFormat::Xls && self.tab_delimited_row_count > 0
    }
}

/// Computes parser-quality counters from a legacy Office projection.
#[must_use]
pub fn legacy_office_quality_metrics(
    _format: LegacyOfficeFormat,
    text: &str,
    markdown: &str,
) -> LegacyOfficeQualityMetrics {
    let line_count = text.lines().count();
    let (non_empty_line_count, tab_delimited_row_count, max_column_count) = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let column_count = line.split('\t').count();
            let has_tab = usize::from(line.contains('\t'));
            (has_tab, column_count)
        })
        .fold(
            (0, 0, 0),
            |(non_empty, tab_rows, max_columns), (has_tab, column_count)| {
                (
                    non_empty + 1,
                    tab_rows + has_tab,
                    max_columns.max(column_count),
                )
            },
        );

    LegacyOfficeQualityMetrics {
        text_char_count: text.chars().count(),
        markdown_char_count: markdown.chars().count(),
        line_count,
        non_empty_line_count,
        tab_delimited_row_count,
        max_column_count,
        markdown_fenced_block_count: markdown.matches("```").count() / 2,
    }
}
