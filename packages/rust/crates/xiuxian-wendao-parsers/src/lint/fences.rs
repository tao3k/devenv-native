use super::types::{MarkdownSyntaxLintCode, MarkdownSyntaxLintIssue};

#[derive(Clone, Copy, Debug)]
struct FenceStart {
    marker: char,
    width: usize,
    line: usize,
}

#[derive(Clone, Copy)]
pub(super) struct FenceMarker<'a> {
    pub(super) marker: char,
    pub(super) width: usize,
    pub(super) trailing: &'a str,
}

pub(super) fn lint_fences(
    body: &str,
    line_offset: usize,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    let mut open_fence: Option<FenceStart> = None;
    for (index, raw_line) in body.lines().enumerate() {
        let line_number = line_offset + index;
        let line = raw_line.trim_end_matches('\r');
        let Some(candidate) = parse_fence_marker(line) else {
            continue;
        };
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
                    line: line_number,
                });
            }
            Some(_) => {}
        }
    }

    if let Some(start) = open_fence {
        issues.push(MarkdownSyntaxLintIssue {
            code: MarkdownSyntaxLintCode::UnclosedFence,
            message: format!(
                "fenced code block starting with `{}` is never closed",
                start.marker.to_string().repeat(start.width)
            ),
            line: start.line,
            column: 1,
        });
    }
}

pub(super) fn parse_fence_marker(line: &str) -> Option<FenceMarker<'_>> {
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
