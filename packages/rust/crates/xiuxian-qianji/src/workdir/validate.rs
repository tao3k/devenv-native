use std::collections::BTreeSet;

use crate::contracts::WorkdirManifest;
use crate::error::QianjiError;

pub(crate) fn validate_workdir_manifest(manifest: &WorkdirManifest) -> Result<(), QianjiError> {
    if manifest.version != 1 {
        return Err(QianjiError::Topology(format!(
            "unsupported bounded work-surface manifest version `{}`: expected `1`",
            manifest.version
        )));
    }

    if manifest.plan.name.trim().is_empty() {
        return Err(QianjiError::Topology(
            "bounded work-surface manifest requires a non-empty `plan.name`".to_string(),
        ));
    }

    validate_surface_entries(&manifest.plan.surface)?;
    validate_required_entries(&manifest.check.require)?;
    validate_flowchart_entries(&manifest.check.flowchart, &manifest.plan.surface)?;

    if !manifest
        .plan
        .surface
        .iter()
        .any(|entry| entry == "flowchart.mmd")
    {
        return Err(QianjiError::Topology(
            "`plan.surface` must include `flowchart.mmd`".to_string(),
        ));
    }

    if !manifest
        .check
        .require
        .iter()
        .any(|entry| entry == "flowchart.mmd")
    {
        return Err(QianjiError::Topology(
            "`check.require` must include `flowchart.mmd`".to_string(),
        ));
    }

    Ok(())
}

fn validate_surface_entries(entries: &[String]) -> Result<(), QianjiError> {
    if entries.is_empty() {
        return Err(QianjiError::Topology(
            "bounded work-surface manifest requires at least one `plan.surface` entry".to_string(),
        ));
    }

    let mut seen = BTreeSet::new();
    for entry in entries {
        validate_top_level_surface(entry, "`plan.surface`")?;
        if !seen.insert(entry.as_str()) {
            return Err(QianjiError::Topology(format!(
                "duplicate `plan.surface` entry `{entry}`"
            )));
        }
    }

    Ok(())
}

fn validate_required_entries(entries: &[String]) -> Result<(), QianjiError> {
    if entries.is_empty() {
        return Err(QianjiError::Topology(
            "bounded work-surface manifest requires at least one `check.require` entry".to_string(),
        ));
    }

    let mut seen = BTreeSet::new();
    for entry in entries {
        validate_requirement_path(entry)?;
        if !seen.insert(entry.as_str()) {
            return Err(QianjiError::Topology(format!(
                "duplicate `check.require` entry `{entry}`"
            )));
        }
    }

    Ok(())
}

fn validate_flowchart_entries(entries: &[String], surfaces: &[String]) -> Result<(), QianjiError> {
    if entries.is_empty() {
        return Err(QianjiError::Topology(
            "bounded work-surface manifest requires at least one `check.flowchart` entry"
                .to_string(),
        ));
    }

    let mut seen = BTreeSet::new();
    for entry in entries {
        validate_top_level_surface(entry, "`check.flowchart`")?;
        if !surfaces.iter().any(|surface| surface == entry) {
            return Err(QianjiError::Topology(format!(
                "`check.flowchart` entry `{entry}` must also appear in `plan.surface`"
            )));
        }
        if !seen.insert(entry.as_str()) {
            return Err(QianjiError::Topology(format!(
                "duplicate `check.flowchart` entry `{entry}`"
            )));
        }
    }

    Ok(())
}

fn validate_top_level_surface(entry: &str, field: &str) -> Result<(), QianjiError> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return Err(QianjiError::Topology(format!(
            "{field} contains an empty entry"
        )));
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(QianjiError::Topology(format!(
            "{field} entry `{trimmed}` must stay at the work-surface top level"
        )));
    }
    if contains_glob_metachar(trimmed) {
        return Err(QianjiError::Topology(format!(
            "{field} entry `{trimmed}` must not contain glob metacharacters"
        )));
    }
    if trimmed == "." || trimmed == ".." {
        return Err(QianjiError::Topology(format!(
            "{field} entry `{trimmed}` is not a valid bounded work-surface path"
        )));
    }

    Ok(())
}

fn validate_requirement_path(entry: &str) -> Result<(), QianjiError> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return Err(QianjiError::Topology(
            "`check.require` contains an empty entry".to_string(),
        ));
    }
    if trimmed.starts_with('/') || trimmed.contains('\\') {
        return Err(QianjiError::Topology(format!(
            "`check.require` entry `{trimmed}` must stay inside the bounded work surface"
        )));
    }

    for segment in trimmed.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(QianjiError::Topology(format!(
                "`check.require` entry `{trimmed}` contains an invalid path segment"
            )));
        }
    }

    Ok(())
}

fn contains_glob_metachar(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
}
