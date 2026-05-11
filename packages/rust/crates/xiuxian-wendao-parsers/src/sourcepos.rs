//! Source-position conversion helpers for `comrak` ranges.

use comrak::nodes::Sourcepos;

/// Parser-owned byte span in the original source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceByteRange {
    /// Inclusive byte offset for the first character in the span.
    pub start: usize,
    /// Exclusive byte offset after the last character in the span.
    pub end: usize,
}

impl SourceByteRange {
    /// Build a byte range from explicit start and end offsets.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Return this span as the legacy tuple shape used by parser DTOs.
    #[must_use]
    pub(crate) const fn as_tuple(self) -> (usize, usize) {
        (self.start, self.end)
    }
}

impl PartialEq<(usize, usize)> for SourceByteRange {
    fn eq(&self, other: &(usize, usize)) -> bool {
        self.start == other.0 && self.end == other.1
    }
}

/// Parser-owned inclusive line/column span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineColumnSpan {
    /// One-based start line.
    pub start_line: usize,
    /// One-based start column.
    pub start_col: usize,
    /// One-based end line.
    pub end_line: usize,
    /// One-based inclusive end column.
    pub end_col: usize,
}

impl LineColumnSpan {
    /// Build a source-position span from explicit line and column coordinates.
    #[must_use]
    pub const fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SourceLineBounds {
    start: usize,
    end: usize,
}

/// Convert one comrak source-position range into a byte range for the original
/// Markdown input.
#[must_use]
pub fn sourcepos_to_byte_range(text: &str, sourcepos: Sourcepos) -> Option<SourceByteRange> {
    line_col_to_byte_range(
        text,
        LineColumnSpan {
            start_line: sourcepos.start.line,
            start_col: sourcepos.start.column,
            end_line: sourcepos.end.line,
            end_col: sourcepos.end.column,
        },
    )
}

/// Convert one inclusive line/column span into a byte range for the original
/// Markdown input.
#[must_use]
pub fn line_col_to_byte_range(text: &str, span: LineColumnSpan) -> Option<SourceByteRange> {
    let start_line = line_bounds(text, span.start_line)?;
    let end_line = line_bounds(text, span.end_line)?;

    let start_line_text = &text[start_line.start..start_line.end];
    let end_line_text = &text[end_line.start..end_line.end];

    let start_byte = start_line.start + byte_offset_for_column(start_line_text, span.start_col);
    let end_byte =
        end_line.start + byte_offset_for_column(end_line_text, span.end_col.saturating_add(1));

    Some(SourceByteRange::new(start_byte, end_byte))
}

fn line_bounds(text: &str, target_line: usize) -> Option<SourceLineBounds> {
    if target_line == 0 {
        return None;
    }

    text.split_inclusive('\n')
        .scan((1_usize, 0_usize), |(line_number, line_start), line| {
            let start = *line_start;
            let end = start + line.trim_end_matches('\n').len();
            let current = *line_number;
            *line_number += 1;
            *line_start += line.len();
            Some((current, SourceLineBounds { start, end }))
        })
        .find_map(|(line_number, bounds)| (line_number == target_line).then_some(bounds))
        .or_else(|| {
            let line_count = text.lines().count().max(1);
            (line_count == target_line).then_some(SourceLineBounds {
                start: text.len(),
                end: text.len(),
            })
        })
}

fn byte_offset_for_column(line_text: &str, column: usize) -> usize {
    let normalized_column = column.max(1);
    if normalized_column == 1 {
        return 0;
    }

    line_text
        .char_indices()
        .nth(normalized_column - 1)
        .map_or(line_text.len(), |(offset, _)| offset)
}
