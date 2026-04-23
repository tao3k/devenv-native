use std::path::{Path, PathBuf};

use crate::contracts::{FlowhubModuleManifest, FlowhubScenarioManifest};
use crate::error::QianjiError;

use super::discover::find_flowhub_root_for_module_dir;
use super::load::load_flowhub_module_manifest;

/// Flowhub module resolved from a scenario `template.use` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFlowhubModule {
    /// Alias assigned by the scenario.
    pub alias: String,
    /// Resolved hierarchical module reference.
    pub module_ref: String,
    /// Stable module name declared by the manifest.
    pub module_name: String,
    /// Resolved module directory under the Flowhub root or parent module.
    pub module_dir: PathBuf,
    /// Resolved module-root manifest path.
    pub manifest_path: PathBuf,
    /// Parsed module-root manifest contract.
    pub manifest: FlowhubModuleManifest,
}

/// Resolve scenario `template.use` entries against a Flowhub root.
///
/// This stage verifies that each requested hierarchical module reference
/// exists, exposes a readable root `qianji.toml`, and declares a `module.name`
/// matching the last segment of the module reference.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when a requested module cannot be found,
/// read, or parsed.
pub fn resolve_flowhub_scenario_modules(
    flowhub_root: impl AsRef<std::path::Path>,
    manifest: &FlowhubScenarioManifest,
) -> Result<Vec<ResolvedFlowhubModule>, QianjiError> {
    resolve_template_use_entries(
        flowhub_root.as_ref(),
        None,
        &manifest.template.use_entries,
        "Flowhub scenario manifest",
    )
}

/// Resolve child modules declared by a composite module-root `[template]`.
///
/// Composite child refs are interpreted relative to the parent module
/// directory, while the returned `module_ref` is qualified back into the full
/// Flowhub hierarchy.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when a child module cannot be found,
/// read, or parsed.
pub fn resolve_flowhub_module_children(
    parent: &ResolvedFlowhubModule,
) -> Result<Vec<ResolvedFlowhubModule>, QianjiError> {
    let Some(template) = &parent.manifest.template else {
        return Ok(Vec::new());
    };

    resolve_template_use_entries(
        &parent.module_dir,
        Some(parent.module_ref.as_str()),
        &template.use_entries,
        "Flowhub composite module manifest",
    )
}

fn resolve_template_use_entries(
    base_dir: &Path,
    parent_module_ref: Option<&str>,
    use_entries: &[crate::contracts::TemplateUseSpec],
    context: &str,
) -> Result<Vec<ResolvedFlowhubModule>, QianjiError> {
    let mut resolved_modules = Vec::with_capacity(use_entries.len());
    let flowhub_root = match parent_module_ref {
        Some(_) => Some(find_flowhub_root_for_module_dir(base_dir)?),
        None => None,
    };

    for use_entry in use_entries {
        let (module_dir, module_ref) = resolve_module_dir_and_ref(
            base_dir,
            flowhub_root.as_deref(),
            parent_module_ref,
            &use_entry.module_ref,
        );
        if !module_dir.is_dir() {
            return Err(QianjiError::Topology(format!(
                "{context} could not resolve module reference `{module_ref}` under `{}`",
                base_dir.display()
            )));
        }

        let manifest_path = module_dir.join("qianji.toml");
        let module_manifest = load_flowhub_module_manifest(&manifest_path)?;
        let expected_module_name = last_module_segment(&module_ref)?;
        if module_manifest.module.name != expected_module_name {
            return Err(QianjiError::Topology(format!(
                "Flowhub module reference `{module_ref}` resolves to manifest `{}` with mismatched `module.name = \"{}\"`",
                manifest_path.display(),
                module_manifest.module.name
            )));
        }

        resolved_modules.push(ResolvedFlowhubModule {
            alias: use_entry.alias.clone(),
            module_ref,
            module_name: module_manifest.module.name.clone(),
            module_dir,
            manifest_path,
            manifest: module_manifest,
        });
    }

    Ok(resolved_modules)
}

fn resolve_module_dir_and_ref(
    base_dir: &Path,
    flowhub_root: Option<&Path>,
    parent_module_ref: Option<&str>,
    scoped_module_ref: &str,
) -> (PathBuf, String) {
    let relative_dir = base_dir.join(scoped_module_ref);
    if relative_dir.is_dir() {
        return (
            relative_dir,
            qualify_relative_module_ref(parent_module_ref, scoped_module_ref),
        );
    }

    if let Some(flowhub_root) = flowhub_root {
        let absolute_dir = flowhub_root.join(scoped_module_ref);
        if absolute_dir.is_dir() {
            return (absolute_dir, scoped_module_ref.to_string());
        }
    }

    (
        relative_dir,
        qualify_relative_module_ref(parent_module_ref, scoped_module_ref),
    )
}

fn qualify_relative_module_ref(parent_module_ref: Option<&str>, scoped_module_ref: &str) -> String {
    match parent_module_ref {
        Some(parent_module_ref) => format!("{parent_module_ref}/{scoped_module_ref}"),
        None => scoped_module_ref.to_string(),
    }
}

fn last_module_segment(module_ref: &str) -> Result<&str, QianjiError> {
    module_ref.rsplit('/').next().ok_or_else(|| {
        QianjiError::Topology(format!(
            "Flowhub module reference `{module_ref}` does not contain a valid module segment"
        ))
    })
}
