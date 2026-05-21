use std::path::Path;

use regex::Regex;

use crate::error::QianjiError;

use super::model::WorkdirDiagnostic;
use super::render::{follow_up_surfaces_for_flowchart, render_label_list};
use super::runtime::WorkdirStepAwareContext;

pub(super) fn validate_flowchart_alignment(
    workdir: &Path,
    flowchart_path: &Path,
    flowchart: Option<&str>,
    flowchart_entries: &[String],
    step_aware: Option<&WorkdirStepAwareContext>,
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) -> Result<(), QianjiError> {
    let Some(flowchart) = flowchart else {
        diagnostics.push(missing_flowchart_companion_diagnostic(
            flowchart_path,
            flowchart_entries,
        ));
        return Ok(());
    };

    if step_aware.is_none() {
        validate_declared_flowchart_surfaces(
            flowchart,
            flowchart_path,
            flowchart_entries,
            diagnostics,
        )?;
    }
    validate_allowed_next_alignment(workdir, step_aware, diagnostics);

    Ok(())
}

fn validate_declared_flowchart_surfaces(
    flowchart: &str,
    flowchart_path: &Path,
    flowchart_entries: &[String],
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) -> Result<(), QianjiError> {
    for surface in flowchart_entries {
        if !flowchart_contains_surface(flowchart, surface)? {
            diagnostics.push(missing_flowchart_surface_diagnostic(
                flowchart_path,
                surface,
                flowchart_entries,
            ));
        }
    }

    for pair in flowchart_entries.windows(2) {
        let from = &pair[0];
        let to = &pair[1];
        if !flowchart_contains_backbone(flowchart, from, to)? {
            diagnostics.push(missing_flowchart_backbone_diagnostic(
                flowchart_path,
                from,
                to,
                flowchart_entries,
            ));
        }
    }

    Ok(())
}

fn validate_allowed_next_alignment(
    workdir: &Path,
    step_aware: Option<&WorkdirStepAwareContext>,
    diagnostics: &mut Vec<WorkdirDiagnostic>,
) {
    let Some(allowed_next) =
        step_aware.and_then(|context| context.allowed_next_validation.as_ref())
    else {
        return;
    };
    if allowed_next.expected_next_labels == allowed_next.actual_next_labels {
        return;
    }

    diagnostics.push(WorkdirDiagnostic {
        title: "Allowed-next drift".to_string(),
        location: workdir.join("state/allowed_next.json"),
        problem: format!(
            "`state/allowed_next.json` declares {}, but the current node `{}` allows {}",
            render_label_list(allowed_next.actual_next_labels.as_slice()),
            allowed_next.current_node_label,
            render_label_list(allowed_next.expected_next_labels.as_slice()),
        ),
        why_it_blocks:
            "the localized run state no longer matches the declared graph transition boundary"
                .to_string(),
        fix: format!(
            "rewrite `state/allowed_next.json` so it matches the current node `{}` adjacency",
            allowed_next.current_node_label
        ),
        follow_up_surfaces: Vec::new(),
    });
}

fn missing_flowchart_companion_diagnostic(
    flowchart_path: &Path,
    flowchart_entries: &[String],
) -> WorkdirDiagnostic {
    WorkdirDiagnostic {
        title: "Missing flowchart companion".to_string(),
        location: flowchart_path.to_path_buf(),
        problem:
            "`flowchart.mmd` is required for flowchart alignment checks, but the file is absent"
                .to_string(),
        why_it_blocks: "the bounded work surface has no direct graph companion".to_string(),
        fix: "create `flowchart.mmd` at the work-surface root".to_string(),
        follow_up_surfaces: follow_up_surfaces_for_flowchart(flowchart_entries),
    }
}

fn missing_flowchart_surface_diagnostic(
    flowchart_path: &Path,
    surface: &str,
    flowchart_entries: &[String],
) -> WorkdirDiagnostic {
    WorkdirDiagnostic {
        title: "Missing flowchart surface".to_string(),
        location: flowchart_path.to_path_buf(),
        problem: format!(
            "`flowchart.mmd` does not visibly contain the principal surface `{surface}`"
        ),
        why_it_blocks: "the graph companion no longer aligns with the bounded work surface"
            .to_string(),
        fix: format!("add a visible `{surface}` node or label to `flowchart.mmd`"),
        follow_up_surfaces: follow_up_surfaces_for_flowchart(flowchart_entries),
    }
}

fn missing_flowchart_backbone_diagnostic(
    flowchart_path: &Path,
    from: &str,
    to: &str,
    flowchart_entries: &[String],
) -> WorkdirDiagnostic {
    WorkdirDiagnostic {
        title: "Missing flowchart backbone".to_string(),
        location: flowchart_path.to_path_buf(),
        problem: format!("`flowchart.mmd` does not visibly express the backbone `{from} --> {to}`"),
        why_it_blocks: "Codex cannot trust the visible backbone direction of the bounded work"
            .to_string(),
        fix: format!("add a visible `{from} --> {to}` relation to `flowchart.mmd`"),
        follow_up_surfaces: follow_up_surfaces_for_flowchart(flowchart_entries),
    }
}

fn flowchart_contains_surface(flowchart: &str, surface: &str) -> Result<bool, QianjiError> {
    let regex = surface_regex(surface)?;
    Ok(regex.is_match(flowchart))
}

fn flowchart_contains_backbone(flowchart: &str, from: &str, to: &str) -> Result<bool, QianjiError> {
    let from_regex = surface_regex(from)?;
    let to_regex = surface_regex(to)?;

    for line in flowchart.lines().filter(|line| line.contains("-->")) {
        let Some(arrow_index) = line.find("-->") else {
            continue;
        };
        let from_match = from_regex
            .find_iter(line)
            .find(|capture| capture.start() < arrow_index);
        let to_match = to_regex
            .find_iter(line)
            .find(|capture| capture.start() > arrow_index);
        if from_match.is_some() && to_match.is_some() {
            return Ok(true);
        }
    }

    Ok(false)
}

fn surface_regex(surface: &str) -> Result<Regex, QianjiError> {
    Regex::new(&format!(
        r"(^|[^A-Za-z0-9_-]){}([^A-Za-z0-9_-]|$)",
        regex::escape(surface)
    ))
    .map_err(|error| {
        QianjiError::Topology(format!(
            "failed to build flowchart surface matcher for `{surface}`: {error}"
        ))
    })
}
