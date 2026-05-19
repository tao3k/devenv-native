//! Org task archive writeback support.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::ClientContext;

use super::model::{AgentOrgTaskListRow, ResolvedReadModelSettings};
use super::row_view::property_value;
use super::settings::resolve_config_path_value;

pub(super) fn apply_archive_plan(
    rows: &[&AgentOrgTaskListRow],
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) -> Result<()> {
    let mut rows_by_source = BTreeMap::<PathBuf, Vec<&AgentOrgTaskListRow>>::new();
    for row in rows {
        rows_by_source
            .entry(PathBuf::from(&row.source_path))
            .or_default()
            .push(*row);
    }

    let mut appends = BTreeMap::<PathBuf, Vec<String>>::new();
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
    }

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
            archive_file_header()
        };
        if !archive_content.ends_with('\n') {
            archive_content.push('\n');
        }
        for subtree in subtrees {
            archive_content.push('\n');
            archive_content.push_str(subtree.trim_end_matches('\n'));
            archive_content.push('\n');
        }
        fs::write(&target, archive_content)
            .with_context(|| format!("failed to write `{}`", target.display()))?;
    }

    Ok(())
}

pub(super) fn archive_target_for_row(
    row: &AgentOrgTaskListRow,
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) -> PathBuf {
    property_value(row, "ARCHIVE_TARGET").map_or_else(
        || {
            settings
                .cache_home
                .join("agent")
                .join("org")
                .join("archives")
                .join("agent_tasks.org")
        },
        |value| resolve_config_path_value(value, context.root(), settings.cache_home.as_path()),
    )
}

fn archive_file_header() -> String {
    concat!(
        "#+TITLE: Agent Org Archive\n",
        "#+AUTHOR: CyberXiuXian Artisan workshop\n",
        "#+FILETAGS: :ARCHIVE:\n"
    )
    .to_string()
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
