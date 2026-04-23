use std::fs;
use std::path::{Path, PathBuf};

use crate::error::QianjiError;
use crate::markdown::{MarkdownShowSection, render_show_surface};

use super::load::load_workdir_manifest;

/// One visible top-level surface from a bounded workdir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirVisibleSurface {
    /// Surface key from `plan.surface`.
    pub surface: String,
    /// Resolved path under the bounded workdir.
    pub path: PathBuf,
    /// Detected surface kind on disk.
    pub kind: WorkdirVisibleSurfaceKind,
    /// Sorted top-level entries when the surface is a directory.
    pub entries: Vec<String>,
}

/// Concrete on-disk state of one visible top-level surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkdirVisibleSurfaceKind {
    /// Surface path is a regular file.
    File,
    /// Surface path is a directory.
    Directory,
    /// Surface path does not exist yet.
    Missing,
    /// Surface path exists but is neither a regular file nor directory.
    Other,
}

/// First-order display surface for a bounded workdir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirShow {
    /// Stable plan name from the manifest.
    pub plan_name: String,
    /// Bounded workdir root.
    pub workdir: PathBuf,
    /// Ordered top-level visible surfaces.
    pub surfaces: Vec<WorkdirVisibleSurface>,
}

/// Load and summarize the first-order visible surface of a bounded workdir.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the root manifest cannot be loaded
/// or one of the visible surfaces cannot be inspected.
pub fn show_workdir(workdir: impl AsRef<Path>) -> Result<WorkdirShow, QianjiError> {
    let workdir = workdir.as_ref();
    let manifest = load_workdir_manifest(workdir.join("qianji.toml"))?;

    let mut surfaces = Vec::with_capacity(manifest.plan.surface.len());
    for surface in &manifest.plan.surface {
        let surface_path = workdir.join(surface);
        let (kind, entries) = inspect_surface(&surface_path)?;
        surfaces.push(WorkdirVisibleSurface {
            surface: surface.clone(),
            path: surface_path,
            kind,
            entries,
        });
    }

    Ok(WorkdirShow {
        plan_name: manifest.plan.name,
        workdir: workdir.to_path_buf(),
        surfaces,
    })
}

/// Render a bounded workdir summary into a compact markdown view.
#[must_use]
pub fn render_workdir_show(show: &WorkdirShow) -> String {
    let sections = show
        .surfaces
        .iter()
        .map(|surface| {
            let mut lines = vec![
                format!("Path: {}", surface.path.display()),
                format!("Status: {}", surface_kind_label(surface.kind)),
            ];
            if surface.kind == WorkdirVisibleSurfaceKind::Directory {
                if surface.entries.is_empty() {
                    lines.push("Entries: (empty)".to_string());
                } else {
                    lines.push("Entries:".to_string());
                    lines.extend(surface.entries.iter().map(|entry| format!("- {entry}")));
                }
            }
            MarkdownShowSection {
                title: surface.surface.as_str().into(),
                lines,
            }
        })
        .collect::<Vec<_>>();

    render_show_surface(
        "Work Surface",
        &[
            format!("Plan: {}", show.plan_name),
            format!("Location: {}", show.workdir.display()),
        ],
        &sections,
    )
}

fn inspect_surface(
    surface_path: &Path,
) -> Result<(WorkdirVisibleSurfaceKind, Vec<String>), QianjiError> {
    if !surface_path.exists() {
        return Ok((WorkdirVisibleSurfaceKind::Missing, Vec::new()));
    }

    if surface_path.is_file() {
        return Ok((WorkdirVisibleSurfaceKind::File, Vec::new()));
    }

    if surface_path.is_dir() {
        let mut entries = fs::read_dir(surface_path)
            .map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to inspect bounded work-surface directory `{}`: {error}",
                    surface_path.display()
                ))
            })?
            .map(|entry| {
                let entry = entry.map_err(|error| {
                    QianjiError::Topology(format!(
                        "Failed to inspect bounded work-surface directory `{}`: {error}",
                        surface_path.display()
                    ))
                })?;
                let file_type = entry.file_type().map_err(|error| {
                    QianjiError::Topology(format!(
                        "Failed to inspect bounded work-surface entry `{}`: {error}",
                        entry.path().display()
                    ))
                })?;
                let mut name = entry.file_name().to_string_lossy().into_owned();
                if file_type.is_dir() {
                    name.push('/');
                }
                Ok(name)
            })
            .collect::<Result<Vec<_>, QianjiError>>()?;
        entries.sort();
        return Ok((WorkdirVisibleSurfaceKind::Directory, entries));
    }

    Ok((WorkdirVisibleSurfaceKind::Other, Vec::new()))
}

fn surface_kind_label(kind: WorkdirVisibleSurfaceKind) -> &'static str {
    match kind {
        WorkdirVisibleSurfaceKind::File => "file",
        WorkdirVisibleSurfaceKind::Directory => "directory",
        WorkdirVisibleSurfaceKind::Missing => "missing",
        WorkdirVisibleSurfaceKind::Other => "other",
    }
}
