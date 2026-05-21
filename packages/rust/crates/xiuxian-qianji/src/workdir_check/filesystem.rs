use std::fs;
use std::path::Path;

use globset::Glob;
use walkdir::WalkDir;

use crate::error::QianjiError;

use super::model::WorkdirDiagnostic;
use super::render::follow_up_surfaces_for_requirement;

pub(super) fn load_optional_flowchart(
    flowchart_path: &Path,
) -> Result<Option<String>, QianjiError> {
    if !flowchart_path.is_file() {
        return Ok(None);
    }

    fs::read_to_string(flowchart_path)
        .map(Some)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read bounded work-surface flowchart `{}`: {error}",
                flowchart_path.display()
            ))
        })
}

pub(super) fn validate_required_paths(
    workdir: &Path,
    required_paths: &[String],
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) -> Result<(), QianjiError> {
    for requirement in required_paths {
        if is_glob_pattern(requirement) {
            emit_missing_glob_diagnostic(workdir, requirement, diagnostics)?;
        } else if !workdir.join(requirement).exists() {
            diagnostics.push(missing_required_path_diagnostic(workdir, requirement));
        }
    }

    Ok(())
}

pub(super) fn count_glob_matches(workdir: &Path, pattern: &str) -> Result<usize, QianjiError> {
    let matcher = Glob::new(pattern)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "invalid `check.require` glob pattern `{pattern}`: {error}"
            ))
        })?
        .compile_matcher();

    WalkDir::new(workdir)
        .into_iter()
        .try_fold(0_usize, |match_count, entry| {
            let entry = entry.map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to walk bounded work surface `{}`: {error}",
                    workdir.display()
                ))
            })?;
            if entry.path() == workdir {
                return Ok(match_count);
            }
            let relative = entry.path().strip_prefix(workdir).map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to relativize bounded work-surface path `{}` against `{}`: {error}",
                    entry.path().display(),
                    workdir.display()
                ))
            })?;
            let normalized = relative.to_string_lossy().replace('\\', "/");
            Ok(match_count + usize::from(matcher.is_match(normalized.as_str())))
        })
}

pub(super) fn is_glob_pattern(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
}

fn emit_missing_glob_diagnostic(
    workdir: &Path,
    requirement: &str,
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) -> Result<(), QianjiError> {
    if count_glob_matches(workdir, requirement)? == 0 {
        diagnostics.push(WorkdirDiagnostic {
            title: "Missing required glob matches".to_string(),
            location: workdir.to_path_buf(),
            problem: format!(
                "bounded work-surface contract requires at least one match for `{requirement}`, but none were found"
            ),
            why_it_blocks: "the bounded surface is structurally incomplete".to_string(),
            fix: format!("create at least one file matching `{requirement}` or relax `check.require`"),
            follow_up_surfaces: follow_up_surfaces_for_requirement(requirement),
        });
    }

    Ok(())
}

fn missing_required_path_diagnostic(workdir: &Path, requirement: &str) -> WorkdirDiagnostic {
    WorkdirDiagnostic {
        title: "Missing required path".to_string(),
        location: workdir.join(requirement),
        problem: format!(
            "bounded work-surface contract requires `{requirement}`, but the path is absent"
        ),
        why_it_blocks: "Codex cannot rely on the declared bounded surface".to_string(),
        fix: format!("create `{requirement}` or relax `check.require`"),
        follow_up_surfaces: follow_up_surfaces_for_requirement(requirement),
    }
}
