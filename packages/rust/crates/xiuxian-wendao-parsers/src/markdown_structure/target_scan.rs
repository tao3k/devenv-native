use crate::targets::{MarkdownTargetOccurrence, MarkdownTargetOccurrenceKind};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
struct FenceStart {
    marker: char,
    width: usize,
}

#[derive(Clone, Copy)]
struct FenceMarker<'a> {
    marker: char,
    width: usize,
    trailing: &'a str,
}

#[derive(Clone, Copy)]
struct ScannedTarget<'a> {
    kind: MarkdownTargetOccurrenceKind,
    target: &'a str,
    start: usize,
    end: usize,
}

pub(super) fn extend_loose_markdown_targets(
    targets: &mut Vec<MarkdownTargetOccurrence>,
    body: &str,
) {
    let mut seen_ranges = targets
        .iter()
        .map(|target| target.byte_range)
        .collect::<HashSet<_>>();
    let mut open_fence: Option<FenceStart> = None;
    let mut line_start = 0usize;

    for (line_number, raw_line) in (1usize..).zip(body.split_inclusive('\n')) {
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
        if let Some(candidate) = parse_fence_marker(line) {
            match open_fence {
                Some(start)
                    if candidate.marker == start.marker
                        && candidate.width >= start.width
                        && candidate.trailing.trim().is_empty() =>
                {
                    open_fence = None;
                }
                None => {
                    open_fence = Some(FenceStart {
                        marker: candidate.marker,
                        width: candidate.width,
                    });
                }
                Some(_) => {}
            }
        } else if open_fence.is_none() {
            scan_line_for_markdown_targets(
                line,
                line_start,
                line_number,
                &mut seen_ranges,
                targets,
            );
        }

        line_start += raw_line.len();
    }
}

fn scan_line_for_markdown_targets(
    line: &str,
    line_start: usize,
    line_number: usize,
    seen_ranges: &mut HashSet<(usize, usize)>,
    targets: &mut Vec<MarkdownTargetOccurrence>,
) {
    let mut cursor = 0usize;
    let mut inline_code_width: Option<usize> = None;
    let bytes = line.as_bytes();

    while cursor < bytes.len() {
        if bytes[cursor] == b'`' {
            let tick_start = cursor;
            while cursor < bytes.len() && bytes[cursor] == b'`' {
                cursor += 1;
            }
            let tick_width = cursor - tick_start;
            match inline_code_width {
                Some(active_width) if active_width == tick_width => inline_code_width = None,
                Some(_) => {}
                None => inline_code_width = Some(tick_width),
            }
            continue;
        }

        if inline_code_width.is_none()
            && let Some(scanned) = scan_markdown_target_at(line, cursor)
        {
            let byte_range = (line_start + scanned.start, line_start + scanned.end);
            if seen_ranges.insert(byte_range) {
                targets.push(MarkdownTargetOccurrence::new(
                    scanned.kind,
                    scanned.target.trim().to_string(),
                    line[scanned.start..scanned.end].to_string(),
                    byte_range,
                    (line_number, line_number),
                ));
            }
            cursor = scanned.end;
            continue;
        }

        cursor += line[cursor..].chars().next().map_or(1, char::len_utf8);
    }
}

fn scan_markdown_target_at(line: &str, start: usize) -> Option<ScannedTarget<'_>> {
    let bytes = line.as_bytes();
    let (kind, label_start) =
        if bytes.get(start) == Some(&b'!') && bytes.get(start + 1) == Some(&b'[') {
            (MarkdownTargetOccurrenceKind::MarkdownImage, start + 2)
        } else if bytes.get(start) == Some(&b'[') {
            (MarkdownTargetOccurrenceKind::MarkdownLink, start + 1)
        } else {
            return None;
        };

    let label_end = find_matching_bracket(line, label_start)?;
    if bytes.get(label_end + 1) != Some(&b'(') {
        return None;
    }
    let target_start = label_end + 2;
    let target_end = find_matching_paren(line, target_start)?;
    Some(ScannedTarget {
        kind,
        target: &line[target_start..target_end],
        start,
        end: target_end + 1,
    })
}

fn find_matching_bracket(line: &str, start: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut depth = 1usize;
    let mut cursor = start;

    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => {
                cursor += 2;
                continue;
            }
            b'[' => depth += 1,
            b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => {}
        }
        cursor += 1;
    }

    None
}

fn find_matching_paren(line: &str, start: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut depth = 1usize;
    let mut cursor = start;

    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => {
                cursor += 2;
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => {}
        }
        cursor += 1;
    }

    None
}

fn parse_fence_marker(line: &str) -> Option<FenceMarker<'_>> {
    let trimmed = line
        .strip_prefix("   ")
        .or_else(|| line.strip_prefix("  "))
        .or_else(|| line.strip_prefix(' '));
    let candidate = trimmed.unwrap_or(line);
    let first = candidate.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let width = candidate.chars().take_while(|ch| *ch == first).count();
    if width < 3 {
        return None;
    }
    Some(FenceMarker {
        marker: first,
        width,
        trailing: &candidate[width..],
    })
}
