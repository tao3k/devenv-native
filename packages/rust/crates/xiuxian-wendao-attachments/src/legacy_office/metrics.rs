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
    let mut line_count = 0;
    let mut non_empty_line_count = 0;
    let mut tab_delimited_row_count = 0;
    let mut max_column_count = 0;

    for line in text.lines() {
        line_count += 1;
        if line.trim().is_empty() {
            continue;
        }
        non_empty_line_count += 1;
        let column_count = line.split('\t').count();
        max_column_count = max_column_count.max(column_count);
        if line.contains('\t') {
            tab_delimited_row_count += 1;
        }
    }

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

#[cfg(test)]
mod tests {
    use super::{LegacyOfficeFormat, legacy_office_quality_metrics};

    #[test]
    fn xls_metrics_preserve_tabular_boundary_signal() {
        let metrics = legacy_office_quality_metrics(
            LegacyOfficeFormat::Xls,
            "name\tvalue\nalpha\t42\nnotes",
            "# rates\n\n```tsv\nname\tvalue\nalpha\t42\nnotes\n```\n",
        );

        assert_eq!(metrics.line_count, 3);
        assert_eq!(metrics.non_empty_line_count, 3);
        assert_eq!(metrics.tab_delimited_row_count, 2);
        assert_eq!(metrics.max_column_count, 2);
        assert_eq!(metrics.markdown_fenced_block_count, 1);
        assert!(metrics.has_tabular_boundary_signal(LegacyOfficeFormat::Xls));
    }
}
