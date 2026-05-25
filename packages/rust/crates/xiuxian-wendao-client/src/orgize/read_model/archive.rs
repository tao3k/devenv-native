//! Org task archive writeback support.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::ClientContext;

use super::model::{AgentOrgTaskListRow, ResolvedReadModelSettings};
use super::row_view::property_value;
use super::settings::resolve_config_path_value;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct ArchiveApplyReport {
    pub(super) rows: usize,
    pub(super) sources_updated: Vec<PathBuf>,
    pub(super) targets_updated: Vec<PathBuf>,
}

pub(super) fn apply_archive_plan(
    rows: &[&AgentOrgTaskListRow],
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) -> Result<ArchiveApplyReport> {
    let mut rows_by_source = BTreeMap::<PathBuf, Vec<&AgentOrgTaskListRow>>::new();
    for row in rows {
        rows_by_source
            .entry(PathBuf::from(&row.source_path))
            .or_default()
            .push(*row);
    }

    let mut appends = BTreeMap::<PathBuf, Vec<String>>::new();
    let mut sources_updated = BTreeSet::<PathBuf>::new();
    for (source_path, mut source_rows) in rows_by_source {
        source_rows.sort_by_key(|row| std::cmp::Reverse(row.source_range_start));
        let original = fs::read_to_string(&source_path)
            .with_context(|| format!("failed to read `{}`", source_path.display()))?;
        let mut updated = original.clone();
        for row in source_rows {
            let start = usize::try_from(row.source_range_start)
                .with_context(|| "archive source range start overflowed usize")?;
            let end = usize::try_from(row.source_range_end)
                .with_context(|| "archive source range end overflowed usize")?;
            if start >= end || end > original.len() {
                anyhow::bail!(
                    "invalid archive range {}..{} for `{}`",
                    start,
                    end,
                    source_path.display()
                );
            }
            let target = archive_target_for_row(row, settings, context);
            if target == source_path {
                anyhow::bail!(
                    "archive target for `{}` resolves to the source file",
                    source_path.display()
                );
            }
            let subtree = original[start..end].trim_end_matches('\n').to_string();
            appends
                .entry(target)
                .or_default()
                .push(mark_subtree_archived(&subtree));
            updated.replace_range(start..end, "");
        }
        fs::write(&source_path, updated)
            .with_context(|| format!("failed to write `{}`", source_path.display()))?;
        sources_updated.insert(source_path);
    }

    let mut targets_updated = BTreeSet::<PathBuf>::new();
    for (target, subtrees) in appends {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create archive directory `{}`", parent.display())
            })?;
        }
        let mut archive_content = if target.is_file() {
            fs::read_to_string(&target)
                .with_context(|| format!("failed to read `{}`", target.display()))?
        } else {
            String::new()
        };
        if !archive_content.is_empty() && !archive_content.ends_with('\n') {
            archive_content.push('\n');
        }
        for subtree in subtrees {
            if !archive_content.is_empty() {
                archive_content.push('\n');
            }
            archive_content.push_str(subtree.trim_end_matches('\n'));
            archive_content.push('\n');
        }
        fs::write(&target, archive_content)
            .with_context(|| format!("failed to write `{}`", target.display()))?;
        targets_updated.insert(target);
    }

    Ok(ArchiveApplyReport {
        rows: rows.len(),
        sources_updated: sources_updated.into_iter().collect(),
        targets_updated: targets_updated.into_iter().collect(),
    })
}

pub(super) fn archive_target_for_row(
    row: &AgentOrgTaskListRow,
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) -> PathBuf {
    property_value(row, "ARCHIVE_TARGET")
        .filter(|value| !archive_target_is_deprecated_bucket(value))
        .map_or_else(
            || default_archive_target_for_row(row, settings),
            |value| resolve_config_path_value(value, context.root(), settings.cache_home.as_path()),
        )
}

fn default_archive_target_for_row(
    row: &AgentOrgTaskListRow,
    settings: &ResolvedReadModelSettings,
) -> PathBuf {
    settings
        .cache_home
        .join("agent")
        .join("org")
        .join("archives")
        .join(
            Path::new(row.source_path.as_str())
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| format!("{}.org", archive_slug(row.title.as_str()))),
        )
}

fn archive_target_is_deprecated_bucket(value: &str) -> bool {
    Path::new(value.trim())
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name == "agent_tasks.org"
                || name.strip_suffix(".org").is_some_and(|stem| {
                    stem.len() == 4 && stem.chars().all(|ch| ch.is_ascii_digit())
                })
        })
}

fn archive_slug(title: &str) -> String {
    let mut slug = String::new();
    let mut previous_separator = false;
    for character in title.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_separator = false;
        } else if !previous_separator {
            slug.push('_');
            previous_separator = true;
        }
    }
    let slug = slug.trim_matches('_');
    if slug.is_empty() {
        "agent_task".to_string()
    } else {
        slug.to_string()
    }
}

fn mark_subtree_archived(subtree: &str) -> String {
    let Some((heading, rest)) = subtree.split_once('\n') else {
        return mark_heading_archived(subtree);
    };
    format!("{}\n{}", mark_heading_archived(heading), rest)
}

fn mark_heading_archived(heading: &str) -> String {
    let trimmed = heading.trim_end();
    if trimmed.contains(":ARCHIVE:") {
        return trimmed.to_string();
    }
    if trimmed.ends_with(':') && trimmed.rfind(" :").is_some() {
        let without_final_colon = &trimmed[..trimmed.len() - 1];
        format!("{without_final_colon}:ARCHIVE:")
    } else {
        format!("{trimmed} :ARCHIVE:")
    }
}
