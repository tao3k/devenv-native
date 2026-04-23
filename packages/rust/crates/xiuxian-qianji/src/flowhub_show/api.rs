use std::path::{Path, PathBuf};

use crate::error::QianjiError;
use crate::flowhub::discover::{
    FlowhubDirKind, classify_flowhub_dir, load_flowhub_module_candidate, module_candidate_from_dir,
    module_candidate_from_ref,
};
use crate::flowhub::load::load_flowhub_root_manifest;

use super::discover::{load_known_module_names_for_show, module_summary};
use super::render::render_flowhub_show_impl;

/// Flowhub module shape displayed by `qianji show`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowhubModuleKind {
    /// Module owns internal child-module composition.
    Composite,
    /// Module is a qianji.toml-anchored leaf node.
    Leaf,
}

/// Compact summary of one Flowhub module within a root or module render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubModuleSummary {
    /// Hierarchical module reference relative to the Flowhub root.
    pub module_ref: String,
    /// Stable module name declared by the root manifest.
    pub module_name: String,
    /// On-disk module directory.
    pub module_dir: PathBuf,
    /// Whether the module is leaf or composite.
    pub kind: FlowhubModuleKind,
    /// Stable entry export.
    pub exports_entry: String,
    /// Stable ready export.
    pub exports_ready: String,
    /// Qualified child module refs for composite modules.
    pub child_modules: Vec<String>,
    /// Immediate Mermaid scenario-case files owned by this module.
    pub scenario_cases: Vec<FlowhubScenarioCaseSummary>,
}

/// Compact summary of one Mermaid scenario-case owned by a Flowhub node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubScenarioCaseSummary {
    /// On-disk Mermaid filename.
    pub file_name: String,
    /// Stable Mermaid graph identity resolved from `[[graph]].name` or the
    /// owning filename stem.
    pub merimind_graph_name: String,
}

/// Root-level Flowhub library summary rendered by `qianji show`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubRootShow {
    /// Flowhub library root on disk.
    pub root: PathBuf,
    /// Ordered summaries of discovered modules.
    pub modules: Vec<FlowhubModuleSummary>,
}

/// Single-module Flowhub summary rendered by `qianji show`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubModuleShow {
    /// Core module summary.
    pub summary: FlowhubModuleSummary,
    /// Count of registered child graph nodes owned by this module.
    pub registered_child_count: usize,
    /// Count of required contract entries anchored by this module.
    pub required_contract_count: usize,
    /// Immediate Mermaid scenario-case files owned by this module.
    pub scenario_cases: Vec<FlowhubScenarioCaseSummary>,
}

/// First-order Flowhub display surface for either a root or one module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowhubShow {
    /// Flowhub library root summary.
    Root(FlowhubRootShow),
    /// Single Flowhub module summary.
    Module(FlowhubModuleShow),
}

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
