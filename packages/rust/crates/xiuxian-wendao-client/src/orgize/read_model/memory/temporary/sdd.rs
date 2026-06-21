use std::path::{Path, PathBuf};

use super::evidence::push_evidence_facet;
use super::model::{OrgEvidenceFacet, OrgEvidenceFacetKind, RecallCandidate};
use crate::orgize::read_model::model::AgentOrgTaskListRow;
use crate::orgize::read_model::row_view::property_value;

const SDD_EVIDENCE_MAX_LINES: usize = 96;
const SDD_EVIDENCE_MAX_HEADINGS: usize = 16;
const SDD_EVIDENCE_MAX_PROPERTIES: usize = 24;

pub(super) fn push_candidate_sdd_facets(
    facets: &mut Vec<OrgEvidenceFacet>,
    candidate: &RecallCandidate<'_>,
) {
    let Some(path) = task_sdd_evidence_path(candidate.row) else {
        return;
    };
    let Ok(source) = std::fs::read_to_string(path.as_path()) else {
        return;
    };

    source.lines().take(SDD_EVIDENCE_MAX_LINES).fold(
        (0usize, 0usize),
        |(headings, properties), line| {
            let trimmed = line.trim();
            if let Some(title) = org_keyword_value_from_line(trimmed, "TITLE") {
                push_evidence_facet(
                    facets,
                    OrgEvidenceFacetKind::Graph,
                    "sdd-title",
                    Some(title),
                );
                return (headings, properties);
            }
            if headings < SDD_EVIDENCE_MAX_HEADINGS
                && let Some(heading) = org_heading_title_for_evidence(trimmed)
            {
                push_evidence_facet(
                    facets,
                    OrgEvidenceFacetKind::Graph,
                    "sdd-heading",
                    Some(heading.as_str()),
                );
                return (headings + 1, properties);
            }
            if properties < SDD_EVIDENCE_MAX_PROPERTIES
                && let Some((key, value)) = org_property_from_line(trimmed)
                && sdd_property_is_evidence(key)
            {
                let label = format!("sdd-{}", key.to_ascii_lowercase().replace('_', "-"));
                push_evidence_facet(
                    facets,
                    OrgEvidenceFacetKind::Graph,
                    label.as_str(),
                    Some(value),
                );
                return (headings, properties + 1);
            }
            (headings, properties)
        },
    );
}

pub(super) fn task_sdd_evidence_path(row: &AgentOrgTaskListRow) -> Option<PathBuf> {
    let raw = property_value(row, "SDD")?.trim();
    if raw.is_empty()
        || raw == "none"
        || raw.starts_with('<')
        || raw.starts_with("id:")
        || raw.starts_with("http://")
        || raw.starts_with("https://")
    {
        return None;
    }

    let root = task_source_project_root(row);
    let expanded = expand_task_sdd_path(raw, root.as_path());
    let path = PathBuf::from(expanded);
    let path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    path.is_file().then_some(path)
}

pub(super) fn task_source_project_root(row: &AgentOrgTaskListRow) -> PathBuf {
    let source = Path::new(row.source_path.as_str());
    let source_text = source.to_string_lossy();
    if let Some(index) = source_text.find("/.cache/agent/org/") {
        return PathBuf::from(&source_text[..index]);
    }
    if let Some(rest) = source_text.strip_prefix(".cache/agent/org/")
        && rest != source_text
    {
        return PathBuf::from(".");
    }
    source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

pub(super) fn expand_task_sdd_path(raw: &str, root: &Path) -> String {
    let cache_home = xiuxian_db_store::state::project_cache_root_from_config(
        xiuxian_db_store::state::ProjectCacheRootConfig {
            project_root: Some(root.to_path_buf()),
            cache_home: None,
            project_namespace: None,
        },
    );
    let replacements = [
        ("${PRJ_CACHE_HOME}", cache_home.as_path()),
        ("$PRJ_CACHE_HOME", cache_home.as_path()),
        ("${PRJ_ROOT}", root),
        ("$PRJ_ROOT", root),
    ];
    let mut expanded = raw.trim().to_string();
    for (token, path) in replacements {
        let path = path.to_string_lossy();
        if expanded == token {
            expanded = path.into_owned();
        } else if let Some(rest) = expanded.strip_prefix(&format!("{token}/")) {
            expanded = format!("{path}/{rest}");
        }
    }
    expanded
}

pub(super) fn org_keyword_value_from_line<'a>(trimmed: &'a str, key: &str) -> Option<&'a str> {
    let (raw_key, value) = trimmed.strip_prefix("#+")?.split_once(':')?;
    raw_key
        .trim()
        .eq_ignore_ascii_case(key)
        .then_some(value.trim())
        .filter(|value| !value.is_empty())
}

pub(super) fn org_property_from_line(trimmed: &str) -> Option<(&str, &str)> {
    let rest = trimmed.strip_prefix(':')?;
    let (key, value) = rest.split_once(':')?;
    let key = key.trim();
    let value = value.trim();
    (!key.is_empty() && !value.is_empty()).then_some((key, value))
}

pub(super) fn sdd_property_is_evidence(key: &str) -> bool {
    matches!(
        key,
        "ID" | "SDD_KIND" | "SDD_STATUS" | "SDD_PARENT" | "SDD_RATIONALE" | "SDD_DECISION"
    )
}

pub(super) fn org_heading_title_for_evidence(trimmed: &str) -> Option<String> {
    let level = trimmed.bytes().take_while(|byte| *byte == b'*').count();
    if level == 0
        || !trimmed
            .as_bytes()
            .get(level)
            .is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }
    let mut title = trimmed[level..].trim();
    for keyword in ["TODO", "DOING", "NEXT", "WAITING", "DONE", "CANCELLED"] {
        if let Some(rest) = title.strip_prefix(keyword) {
            title = rest.trim_start();
            break;
        }
    }
    if title.starts_with("[#") && title.get(3..4) == Some("]") {
        title = title[4..].trim_start();
    }
    let tagless = strip_org_heading_tags(title).trim();
    (!tagless.is_empty()).then(|| tagless.to_string())
}

pub(super) fn strip_org_heading_tags(title: &str) -> &str {
    let Some((before, after)) = title.rsplit_once(' ') else {
        return title;
    };
    if after.starts_with(':')
        && after.ends_with(':')
        && after.trim_matches(':').split(':').all(|tag| {
            !tag.is_empty()
                && tag
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' || ch == '#')
        })
    {
        before
    } else {
        title
    }
}
