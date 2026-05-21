//! Path confinement helpers for Episteme cache materializers.

use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};

pub(crate) fn resolve_run_output_path(
    run_dir: &Path,
    planned_output_path: &str,
    task_label: &str,
) -> Result<PathBuf> {
    let relative_path = validate_relative_path(planned_output_path, task_label, "output path")?;
    let mut components = relative_path.components();
    let first_component = components
        .next()
        .context("episteme cache output path is empty")?;
    if !matches!(first_component, Component::Normal(name) if name == "outputs") {
        anyhow::bail!(
            "episteme cache output path for `{task_label}` must stay under the run outputs directory"
        );
    }
    Ok(run_dir.join(relative_path))
}

pub(crate) fn resolve_existing_corpus_file(
    corpus_root: &Path,
    relative_path: &str,
    task_label: &str,
) -> Result<PathBuf> {
    let relative_path = validate_relative_path(relative_path, task_label, "source path")?;
    let canonical_root = corpus_root.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize episteme corpus root `{}`",
            corpus_root.display()
        )
    })?;
    let candidate = corpus_root.join(relative_path);
    let canonical_candidate = candidate.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize episteme source path `{}`",
            candidate.display()
        )
    })?;
    if !canonical_candidate.starts_with(&canonical_root) {
        anyhow::bail!("episteme source path for `{task_label}` escapes the corpus root");
    }
    Ok(canonical_candidate)
}

fn validate_relative_path<'a>(
    value: &'a str,
    task_label: &str,
    field_name: &str,
) -> Result<&'a Path> {
    let path = Path::new(value);
    if path.as_os_str().is_empty() {
        anyhow::bail!("episteme cache {field_name} for `{task_label}` is empty");
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => anyhow::bail!(
                "episteme cache {field_name} for `{task_label}` must be a clean relative path"
            ),
        }
    }
    Ok(path)
}
