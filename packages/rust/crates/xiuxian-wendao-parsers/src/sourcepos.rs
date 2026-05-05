//! Source-position conversion helpers for `comrak` ranges.

use comrak::nodes::Sourcepos;

/// Convert one comrak source-position range into a byte range for the original
/// Markdown input.
#[must_use]
pub fn sourcepos_to_byte_range(text: &str, sourcepos: Sourcepos) -> Option<(usize, usize)> {
    line_col_to_byte_range(
        text,
        sourcepos.start.line,
        sourcepos.start.column,
        sourcepos.end.line,
        sourcepos.end.column,
    )
}

/// Convert one inclusive line/column span into a byte range for the original
/// Markdown input.
#[must_use]
pub fn line_col_to_byte_range(
    text: &str,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
) -> Option<(usize, usize)> {
    let (start_line_byte, start_line_end) = line_bounds(text, start_line)?;
    let (end_line_byte, end_line_end) = line_bounds(text, end_line)?;

    let start_line_text = &text[start_line_byte..start_line_end];
    let end_line_text = &text[end_line_byte..end_line_end];

    let start_byte = start_line_byte + byte_offset_for_column(start_line_text, start_col);
    let end_byte = end_line_byte + byte_offset_for_column(end_line_text, end_col.saturating_add(1));

    Some((start_byte, end_byte))
}

fn line_bounds(text: &str, target_line: usize) -> Option<(usize, usize)> {
    if target_line == 0 {
        return None;
    }

    let mut current_line = 1;
    let mut line_start = 0;

    for (byte_idx, ch) in text.char_indices() {
        if current_line == target_line && ch == '\n' {
            return Some((line_start, byte_idx));
        }

        if ch == '\n' {
            current_line += 1;
            line_start = byte_idx + ch.len_utf8();
        }
    }

    (current_line == target_line).then_some((line_start, text.len()))
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
