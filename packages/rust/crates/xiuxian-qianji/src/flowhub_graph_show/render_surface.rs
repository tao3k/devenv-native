use std::collections::BTreeSet;
use std::path::Path;

use xiuxian_config_core::resolve_project_root;

use crate::contracts::FlowhubGraphSurfaceContract;
use crate::flowhub::FlowhubScenarioIr;

use super::api::{FlowhubGraphCheckSurface, FlowhubGraphShow};

pub(super) fn render_check_surface_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
    let mut lines = Vec::new();
    if !show.module_contract_surface.is_empty() {
        lines.push(format!(
            "- Flowhub source surface: {}.",
            render_file_list(&show.module_contract_surface)
        ));
    }

    let Some(root) = &show.declared_check_surface.root else {
        lines.push(
            "- No declared bounded check surface. Add `[graph.workdir]` or Mermaid `%% qianji.scenario.*` workdir metadata before relying on `qianji check` guidance."
                .to_string(),
        );
        return lines;
    };

    lines.push(format!("- Run root: `{root}`."));
    if !show.declared_check_surface.required_paths.is_empty() {
        lines.push("- `qianji check` requires the following declared paths and globs:".to_string());
        lines.extend(render_text_block(
            &show.declared_check_surface.required_paths,
        ));
    }
    if !show.declared_check_surface.flowchart_surfaces.is_empty() {
        lines.push(format!(
            "- `qianji check` keeps these surfaces visible in `flowchart.mmd`: {}.",
            render_file_list(&show.declared_check_surface.flowchart_surfaces)
        ));
    }

    lines
}

pub(super) fn render_persistent_target_surface_section_lines(
    show: &FlowhubGraphShow,
) -> Vec<String> {
    let mut lines = vec![
        "- Merge validated staging artifacts into this canonical surface after `qianji check` passes."
            .to_string(),
    ];
    lines.extend(render_text_block(
        &show.declared_check_surface.persistent_target_surface_tree,
    ));
    lines
}

pub(super) fn render_done_gate_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
    let mut lines =
        vec!["- Completion remains blocked until these declared paths are satisfied:".to_string()];
    lines.extend(render_text_block(
        &show.declared_check_surface.done_gate_require,
    ));
    lines
}

pub(super) fn declared_check_surface(
    scenario_ir: Option<&FlowhubScenarioIr>,
) -> FlowhubGraphCheckSurface {
    let Some(workdir) = scenario_ir.and_then(|graph| graph.workdir.as_ref()) else {
        return FlowhubGraphCheckSurface {
            note: Some(
                "This graph does not yet declare `[graph.workdir]`, so `show --graph` can only render the source/module contract until Flowhub declares a bounded check surface."
                    .to_string(),
            ),
            root: None,
            required_paths: Vec::new(),
            flowchart_surfaces: Vec::new(),
            persistent_target_surface_tree: Vec::new(),
            done_gate_require: Vec::new(),
        };
    };

    FlowhubGraphCheckSurface {
        note: workdir.note.clone(),
        root: Some(workdir.root.clone()),
        required_paths: workdir.check.require.clone(),
        flowchart_surfaces: workdir.check.flowchart.clone(),
        persistent_target_surface_tree: workdir
            .target
            .as_ref()
            .map_or_else(Vec::new, render_surface_contract_tree),
        done_gate_require: workdir.done_gate_require.clone(),
    }
}

pub(crate) fn render_label_list(values: &[String]) -> String {
    if values.is_empty() {
        return "`none`".to_string();
    }

    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn render_file_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn display_graph_path(path: &Path) -> String {
    if path.is_absolute() {
        if let Some(project_root) = resolve_project_root()
            && let Ok(relative) = path.strip_prefix(&project_root)
        {
            return format!("./{}", relative.display());
        }
        if let Ok(current_dir) = std::env::current_dir()
            && let Ok(relative) = path.strip_prefix(&current_dir)
        {
            return format!("./{}", relative.display());
        }
        return path.display().to_string();
    }

    let rendered = path.display().to_string();
    if rendered.starts_with("./") || rendered.starts_with("../") {
        rendered
    } else {
        format!("./{rendered}")
    }
}

fn render_text_block(lines: &[String]) -> Vec<String> {
    if lines.is_empty() {
        return vec!["- none".to_string()];
    }

    let mut block = vec!["```text".to_string()];
    block.extend(lines.iter().cloned());
    block.push("```".to_string());
    block
}

fn render_surface_contract_tree(surface: &FlowhubGraphSurfaceContract) -> Vec<String> {
    render_surface_tree(surface.root.as_str(), surface.paths.as_slice())
}

fn render_surface_tree(root: &str, paths: &[String]) -> Vec<String> {
    let directory_hints = paths
        .iter()
        .filter_map(|path| {
            let trimmed = path.trim();
            if trimmed.ends_with('/') {
                Some(trimmed.trim_end_matches('/').to_string())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    render_surface_tree_with_directory_hints(root, paths, &directory_hints)
}

fn render_surface_tree_with_directory_hints(
    root: &str,
    paths: &[String],
    directory_hints: &BTreeSet<String>,
) -> Vec<String> {
    let mut lines = vec![format!("{root}/")];
    let mut seen = BTreeSet::new();

    for path in paths {
        let trimmed = path.trim();
        let normalized = trimmed.trim_end_matches('/');
        if normalized.is_empty() {
            continue;
        }

        let segments = normalized.split('/').collect::<Vec<_>>();
        let mut prefix = String::new();
        for (index, segment) in segments.iter().enumerate() {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(segment);

            let is_directory_line =
                index + 1 < segments.len() || directory_hints.contains(prefix.as_str());
            let key = if is_directory_line {
                format!("{prefix}/")
            } else {
                prefix.clone()
            };
            if seen.insert(key) {
                lines.push(format!(
                    "{}{}{}",
                    "  ".repeat(index + 1),
                    segment,
                    if is_directory_line { "/" } else { "" }
                ));
            }
        }
    }

    lines
}
