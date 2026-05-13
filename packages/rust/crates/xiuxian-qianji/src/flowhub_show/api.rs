//! Flowhub show API.
//!
//! This module selects either root-level or module-level Flowhub inspection and
//! delegates formatting to the renderer behind one public `qianji show` seam.

use std::path::Path;

use crate::error::QianjiError;
use crate::flowhub::discover::{
    FlowhubDirKind, classify_flowhub_dir, load_flowhub_module_candidate, module_candidate_from_dir,
    module_candidate_from_ref,
};
use crate::flowhub::load::load_flowhub_root_manifest;

use super::discover::{load_known_module_names_for_show, module_summary};
use super::model::{FlowhubModuleShow, FlowhubRootShow, FlowhubShow};
use super::render::render_flowhub_show_impl;

/// Load and summarize a Flowhub library root or module directory.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the target is not Flowhub-shaped or
/// its manifest cannot be loaded.
pub fn show_flowhub(dir: impl AsRef<Path>) -> Result<FlowhubShow, QianjiError> {
    let dir = dir.as_ref();
    match classify_flowhub_dir(dir)? {
        Some(FlowhubDirKind::Root) => {
            let root_manifest = load_flowhub_root_manifest(dir.join("qianji.toml"))?;
            let modules = root_manifest
                .contract
                .register
                .iter()
                .map(|module_ref| {
                    load_flowhub_module_candidate(&module_candidate_from_ref(dir, module_ref))
                        .and_then(|module| {
                            module_summary(&module, &root_manifest.contract.register)
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(FlowhubShow::Root(FlowhubRootShow {
                root: dir.to_path_buf(),
                modules,
            }))
        }
        Some(FlowhubDirKind::Module) => {
            let candidate = module_candidate_from_dir(dir)?;
            let module = load_flowhub_module_candidate(&candidate)?;
            let registered_child_count = module
                .manifest
                .contract
                .as_ref()
                .map(|contract| contract.register.len())
                .unwrap_or_default();
            let required_contract_count = module
                .manifest
                .contract
                .as_ref()
                .map(|contract| contract.required.len())
                .unwrap_or_default();
            let known_module_names = load_known_module_names_for_show(&module.module_dir)?;
            Ok(FlowhubShow::Module(FlowhubModuleShow {
                scenario_cases: super::discover::discover_immediate_scenario_cases(
                    &module.module_dir,
                    &module.manifest.graph,
                    &known_module_names,
                )?,
                summary: module_summary(&module, &known_module_names)?,
                registered_child_count,
                required_contract_count,
            }))
        }
        None => Err(QianjiError::Topology(format!(
            "`{}` is not a Flowhub root or module directory",
            dir.display()
        ))),
    }
}

/// Render a Flowhub root/module summary into a compact markdown view.
#[must_use]
pub fn render_flowhub_show(show: &FlowhubShow) -> String {
    render_flowhub_show_impl(show)
}
