//! Agent tracking `orgize` lint helpers.

use orgize::ast::SourcePosition;
use orgize::lint::LintLocation;

pub(super) struct RequiredAgentOrgKeyword {
    pub(super) key: &'static str,
}

pub(super) const REQUIRED_AGENT_ORG_KEYWORDS: &[RequiredAgentOrgKeyword] = &[
    RequiredAgentOrgKeyword { key: "TITLE" },
    RequiredAgentOrgKeyword { key: "AUTHOR" },
    RequiredAgentOrgKeyword { key: "FILETAGS" },
    RequiredAgentOrgKeyword { key: "DATE" },
];

pub(super) fn is_agent_tracking_org_source(keywords: &[(String, String)], source: &str) -> bool {
    org_filetags_include_agent(keywords) || source_contains_agent_tracking_markers(source)
}

fn source_contains_agent_tracking_markers(source: &str) -> bool {
    source.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with(":SDD_KIND:")
            || trimmed.starts_with(":SDD_STATUS:")
            || (trimmed.starts_with('*')
                && (trimmed.contains(":agent:")
                    || trimmed.contains(":sdd:")
                    || trimmed.contains(":execplan:")))
    })
}

pub(super) fn org_keyword_value<'a>(
    keywords: &'a [(String, String)],
    target: &str,
) -> Option<&'a str> {
    keywords
        .iter()
        .find_map(|(key, value)| (key == target).then_some(value.as_str()))
}

pub(super) fn org_filetags_include_agent(keywords: &[(String, String)]) -> bool {
    org_keyword_value(keywords, "FILETAGS").is_some_and(|filetags| {
        filetags
            .split(':')
            .any(|tag| tag.trim().eq_ignore_ascii_case("agent"))
    })
}

pub(super) fn has_org_keyword(keywords: &[(String, String)], target: &str) -> bool {
    keywords.iter().any(|(key, _)| key == target)
}

pub(super) fn agent_org_date_has_seconds(value: &str) -> bool {
    if value.trim() == "YYYY-MM-DD Day HH:MM:SS" {
        return true;
    }

    let mut parts = value.split_whitespace();
    let Some(date) = parts.next() else {
        return false;
    };
    let Some(day) = parts.next() else {
        return false;
    };
    let Some(time) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && org_date_token_has_ymd(date)
        && !day.is_empty()
        && org_time_token_has_seconds(time)
}

fn org_date_token_has_ymd(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

fn org_time_token_has_seconds(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 8
        && bytes[2] == b':'
        && bytes[5] == b':'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 2 | 5) || byte.is_ascii_digit())
}

pub(super) struct OrgKeywordLine {
    pub(super) key: String,
    pub(super) value: String,
    pub(super) start: usize,
    pub(super) end: usize,
}

pub(super) fn leading_org_keyword_lines(source: &str) -> Vec<OrgKeywordLine> {
    let mut keys = Vec::new();
    let mut offset = 0;
    while offset < source.len() {
        let line_start = offset;
        let line_end = line_end(source, line_start);
        offset = line_end;
        let line = &source[line_start..line_end];
        let trimmed = line.trim_start();
        if trimmed.starts_with('*') {
            break;
        }
        let Some((raw_key, value)) = trimmed
            .strip_prefix("#+")
            .and_then(|line| line.split_once(':'))
        else {
            continue;
        };
        if !value.trim().is_empty() {
            keys.push(OrgKeywordLine {
                key: raw_key.trim().to_ascii_uppercase(),
                value: value.trim().to_string(),
                start: line_start,
                end: line_end,
            });
        }
    }
    keys
}

pub(super) fn closed_line_uses_inactive_timestamp(line: &str) -> Option<bool> {
    let (_, rest) = line.split_once("CLOSED:")?;
    Some(rest.trim_start().starts_with('['))
}

pub(super) fn agent_task_todo_keyword(value: &str) -> bool {
    matches!(
        value,
        "TODO" | "DOING" | "NEXT" | "WAITING" | "DONE" | "CANCELLED"
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProgressCookie {
    Zero,
    Partial,
    Complete,
}

pub(super) fn progress_cookie_from_title(title: &str) -> Option<ProgressCookie> {
    title
        .split_whitespace()
        .filter_map(progress_cookie_from_token)
        .max_by_key(|progress| match progress {
            ProgressCookie::Zero => 0,
            ProgressCookie::Partial => 1,
            ProgressCookie::Complete => 2,
        })
}

fn progress_cookie_from_token(token: &str) -> Option<ProgressCookie> {
    let cookie = token.strip_prefix('[')?.strip_suffix(']')?;
    if let Some(percent) = cookie.strip_suffix('%') {
        let percent = percent.parse::<u64>().ok()?;
        return Some(progress_cookie_from_percent(percent));
    }
    let (done, total) = cookie.split_once('/')?;
    let done = done.parse::<u64>().ok()?;
    let total = total.parse::<u64>().ok()?;
    if total == 0 || done == 0 {
        Some(ProgressCookie::Zero)
    } else if done >= total {
        Some(ProgressCookie::Complete)
    } else {
        Some(ProgressCookie::Partial)
    }
}

fn progress_cookie_from_percent(percent: u64) -> ProgressCookie {
    if percent == 0 {
        ProgressCookie::Zero
    } else if percent >= 100 {
        ProgressCookie::Complete
    } else {
        ProgressCookie::Partial
    }
}

pub(super) fn heading_line_end(source: &str, start: usize) -> usize {
    source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset)
}

pub(super) fn lint_location_for_offsets(source: &str, start: usize, end: usize) -> LintLocation {
    let start = start.min(source.len());
    let end = end.min(source.len());
    LintLocation {
        start: source_position_for_offset(source, start),
        end: source_position_for_offset(source, end),
        range_start: start,
        range_end: end,
    }
}

pub(super) fn source_position_for_offset(source: &str, offset: usize) -> SourcePosition {
    let offset = offset.min(source.len());
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let column = source[line_start..offset].chars().count() + 1;
    SourcePosition { line, column }
}

fn line_end(source: &str, start: usize) -> usize {
    source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset + 1)
}
