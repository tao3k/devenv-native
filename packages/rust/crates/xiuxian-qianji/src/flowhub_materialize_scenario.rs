use std::fs;
use std::path::{Path, PathBuf};

use crate::error::QianjiError;
use crate::{
    check_flowhub, check_workdir, load_flowhub_scenario_manifest, render_flowhub_check_markdown,
    render_workdir_check_markdown, resolve_flowhub_scenario_modules,
};

use super::materialize_copy::copy_template_dir;
use super::materialize_root::render_root_manifest;
use super::materialize_safety::ensure_output_dir_is_safe;
use crate::flowhub::{derive_flowchart_aliases, render_flowchart};

/// Summary of one generated bounded work surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializedWorkdir {
    /// Stable scenario/plan name.
    pub plan_name: String,
    /// Materialized work-surface root.
    pub output_dir: PathBuf,
    /// Ordered visible top-level surfaces excluding `flowchart.mmd`.
    pub visible_aliases: Vec<String>,
}

/// Materialize a Flowhub scenario manifest into one compact bounded work
/// surface.
///
/// This helper only materializes visible template-bearing leaf nodes plus the
/// compact root contract. The real `qianji-flowhub/` root is now qianji.toml-
/// only; template-bearing Flowhub trees remain test-only fixture inputs for
/// this helper. It does not compile runtime graphs or expose a new CLI verb
/// yet.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the scenario cannot be loaded,
/// selected modules fail Flowhub validation, the output directory is unsafe to
/// use, template copying fails, or the generated work surface does not pass
/// the existing compact workdir validation contract.
pub fn materialize_flowhub_scenario_workdir(
    flowhub_root: impl AsRef<Path>,
    scenario_manifest_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<MaterializedWorkdir, QianjiError> {
    let flowhub_root = flowhub_root.as_ref();
    let output_dir = output_dir.as_ref();
    let manifest = load_flowhub_scenario_manifest(scenario_manifest_path)?;
    let resolved_modules = resolve_flowhub_scenario_modules(flowhub_root, &manifest)?;

    ensure_output_dir_is_safe(output_dir)?;

    let visible_modules = resolved_modules
        .iter()
        .filter(|module| module.manifest.template.is_none())
        .collect::<Vec<_>>();
    if visible_modules.is_empty() {
        return Err(QianjiError::Topology(format!(
            "Flowhub scenario `{}` does not expose any template-bearing leaf nodes that can materialize into a bounded work surface",
            manifest.planning.name
        )));
    }

    for module in &resolved_modules {
        let report = check_flowhub(&module.module_dir)?;
        if !report.is_valid() {
            return Err(QianjiError::Topology(format!(
                "Flowhub module `{}` failed validation before materialization:\n{}",
                module.module_ref,
                render_flowhub_check_markdown(&report)
            )));
        }
    }

    fs::create_dir_all(output_dir).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to create materialized workdir `{}`: {error}",
            output_dir.display()
        ))
    })?;

    let visible_aliases = visible_modules
        .iter()
        .map(|module| module.alias.clone())
        .collect::<Vec<_>>();
    for module in &visible_modules {
        copy_template_dir(
            &module.module_dir.join("template"),
            &output_dir.join(&module.alias),
        )?;
    }

    let flowchart_aliases = derive_flowchart_aliases(&manifest, &visible_aliases);
    let root_manifest = render_root_manifest(
        &manifest.planning.name,
        &visible_aliases,
        &flowchart_aliases,
    )?;
    fs::write(output_dir.join("qianji.toml"), root_manifest).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to write materialized root manifest `{}`: {error}",
            output_dir.join("qianji.toml").display()
        ))
    })?;

    let flowchart = render_flowchart(&manifest, &visible_aliases, &flowchart_aliases);
    fs::write(output_dir.join("flowchart.mmd"), flowchart).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to write materialized flowchart `{}`: {error}",
            output_dir.join("flowchart.mmd").display()
        ))
    })?;

    let report = check_workdir(output_dir)?;
    if !report.is_valid() {
        return Err(QianjiError::Topology(format!(
            "Generated work surface `{}` failed validation:\n{}",
            output_dir.display(),
            render_workdir_check_markdown(&report)
        )));
    }

    Ok(MaterializedWorkdir {
        plan_name: manifest.planning.name,
        output_dir: output_dir.to_path_buf(),
        visible_aliases,
    })
}
