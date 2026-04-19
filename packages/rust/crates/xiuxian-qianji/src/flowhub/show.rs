use std::path::{Path, PathBuf};

use crate::contracts::FlowhubGraphContract;
use crate::error::QianjiError;
use crate::flowhub::mermaid::parse_mermaid_flowchart;
use crate::markdown::{MarkdownShowSection, render_show_surface};
use serde_json::json;
use xiuxian_qianhuan::EmbeddedManifestationTemplateCatalog;

use super::discover::{
    FlowhubDirKind, FlowhubDiscoveredModule, classify_flowhub_dir, load_flowhub_module_candidate,
    module_candidate_from_dir, module_candidate_from_ref,
};
use super::load::load_flowhub_root_manifest;
use super::scenario_ir::{parse_flowhub_graph_annotations, resolve_flowhub_graph_name};

const FLOWHUB_SCENARIO_CASE_TEMPLATE_NAME: &str = "flowhub_scenario_case.md.j2";
const FLOWHUB_SCENARIO_CASE_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/flowhub_scenario_case.md.j2");
const FLOWHUB_ROOT_MODULE_TEMPLATE_NAME: &str = "flowhub_root_module.md.j2";
const FLOWHUB_ROOT_MODULE_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/flowhub_root_module.md.j2");
const FLOWHUB_MODULE_EXPORTS_TEMPLATE_NAME: &str = "flowhub_module_exports.md.j2";
const FLOWHUB_MODULE_EXPORTS_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/flowhub_module_exports.md.j2");
const FLOWHUB_MODULE_CONTRACT_TEMPLATE_NAME: &str = "flowhub_module_contract.md.j2";
const FLOWHUB_MODULE_CONTRACT_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/flowhub_module_contract.md.j2");

static FLOWHUB_TEMPLATE_CATALOG: EmbeddedManifestationTemplateCatalog =
    EmbeddedManifestationTemplateCatalog::new(
        "Flowhub show template renderer",
        &[
            (
                FLOWHUB_SCENARIO_CASE_TEMPLATE_NAME,
                FLOWHUB_SCENARIO_CASE_TEMPLATE_SOURCE,
            ),
            (
                FLOWHUB_ROOT_MODULE_TEMPLATE_NAME,
                FLOWHUB_ROOT_MODULE_TEMPLATE_SOURCE,
            ),
            (
                FLOWHUB_MODULE_EXPORTS_TEMPLATE_NAME,
                FLOWHUB_MODULE_EXPORTS_TEMPLATE_SOURCE,
            ),
            (
                FLOWHUB_MODULE_CONTRACT_TEMPLATE_NAME,
                FLOWHUB_MODULE_CONTRACT_TEMPLATE_SOURCE,
            ),
        ],
    );

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
                scenario_cases: discover_immediate_scenario_cases(
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
    match show {
        FlowhubShow::Root(show) => render_flowhub_root_show(show),
        FlowhubShow::Module(show) => render_flowhub_module_show(show),
    }
}

fn render_flowhub_root_show(show: &FlowhubRootShow) -> String {
    let sections = show
        .modules
        .iter()
        .map(|module| {
            let lines = render_flowhub_root_module_section_lines(module);
            MarkdownShowSection {
                title: module.module_ref.as_str().into(),
                lines,
            }
        })
        .collect::<Vec<_>>();

    render_show_surface(
        "Flowhub",
        &[
            format!("Location: {}", show.root.display()),
            format!("Modules: {}", show.modules.len()),
        ],
        &sections,
    )
}

fn render_flowhub_module_show(show: &FlowhubModuleShow) -> String {
    let summary = &show.summary;
    let mut sections = vec![
        MarkdownShowSection {
            title: "Exports".into(),
            lines: render_flowhub_module_exports_section_lines(summary),
        },
        MarkdownShowSection {
            title: "Contract".into(),
            lines: render_flowhub_module_contract_section_lines(show),
        },
    ];

    if !summary.child_modules.is_empty() {
        sections.push(MarkdownShowSection {
            title: "Children".into(),
            lines: summary
                .child_modules
                .iter()
                .map(|child| format!("- {child}"))
                .collect(),
        });
    }
    if !show.scenario_cases.is_empty() {
        sections.push(MarkdownShowSection {
            title: "Scenario Cases".into(),
            lines: render_scenario_case_section_lines(&summary.module_ref, &show.scenario_cases),
        });
    }

    render_show_surface(
        "Flowhub Module",
        &[
            format!("Module: {}", summary.module_ref),
            format!("Name: {}", summary.module_name),
            format!("Location: {}", summary.module_dir.display()),
            format!("Kind: {}", module_kind_label(summary.kind)),
        ],
        &sections,
    )
}

fn module_summary(
    module: &FlowhubDiscoveredModule,
    known_module_names: &[String],
) -> Result<FlowhubModuleSummary, QianjiError> {
    let child_modules = module
        .manifest
        .contract
        .as_ref()
        .map(|contract| {
            contract
                .register
                .iter()
                .map(|child_ref| {
                    resolve_child_module_ref(&module.module_dir, &module.module_ref, child_ref)
                })
                .collect()
        })
        .unwrap_or_default();
    let scenario_cases = discover_immediate_scenario_cases(
        &module.module_dir,
        &module.manifest.graph,
        known_module_names,
    )?;

    Ok(FlowhubModuleSummary {
        module_ref: module.module_ref.clone(),
        module_name: module.manifest.module.name.clone(),
        module_dir: module.module_dir.clone(),
        kind: if module_owns_child_graphs(module) {
            FlowhubModuleKind::Composite
        } else {
            FlowhubModuleKind::Leaf
        },
        exports_entry: module.manifest.exports.entry.clone(),
        exports_ready: module.manifest.exports.ready.clone(),
        child_modules,
        scenario_cases,
    })
}

fn module_owns_child_graphs(module: &FlowhubDiscoveredModule) -> bool {
    module
        .manifest
        .contract
        .as_ref()
        .is_some_and(|contract| !contract.register.is_empty())
        || module.manifest.template.is_some()
}

fn resolve_child_module_ref(
    parent_module_dir: &Path,
    parent_module_ref: &str,
    child_module_ref: &str,
) -> String {
    if parent_module_dir.join(child_module_ref).is_dir() {
        return format!("{parent_module_ref}/{child_module_ref}");
    }
    child_module_ref.to_string()
}

fn discover_immediate_scenario_cases(
    module_dir: &Path,
    graph_contracts: &[FlowhubGraphContract],
    known_module_names: &[String],
) -> Result<Vec<FlowhubScenarioCaseSummary>, QianjiError> {
    let mut scenario_cases = std::fs::read_dir(module_dir)
        .map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to inspect Flowhub module directory `{}`: {error}",
                module_dir.display()
            ))
        })?
        .map(|entry| {
            let entry = entry.map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to inspect Flowhub module directory `{}`: {error}",
                    module_dir.display()
                ))
            })?;
            Ok(entry.path())
        })
        .collect::<Result<Vec<_>, QianjiError>>()?
        .into_iter()
        .filter(|path| path.is_file())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("mmd"))
        .filter_map(|path| summarize_scenario_case(&path, graph_contracts, known_module_names))
        .collect::<Vec<_>>();
    scenario_cases.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok(scenario_cases)
}

fn summarize_scenario_case(
    path: &Path,
    graph_contracts: &[FlowhubGraphContract],
    known_module_names: &[String],
) -> Option<FlowhubScenarioCaseSummary> {
    let file_name = path.file_name()?.to_str()?.to_string();
    let file_stem = path.file_stem()?.to_str()?.to_string();
    let declared_graph = graph_contracts.iter().find(|graph| graph.path == file_name);
    let merimind_graph_name = std::fs::read_to_string(path)
        .ok()
        .and_then(|source| {
            let annotations = parse_flowhub_graph_annotations(&source).ok().flatten();
            let graph_name =
                resolve_flowhub_graph_name(annotations.as_ref(), declared_graph, file_stem.as_str());
            parse_mermaid_flowchart(&source, graph_name.as_str(), known_module_names).ok()
        })
        .map_or_else(
            || {
                resolve_flowhub_graph_name(None, declared_graph, file_stem.as_str()).to_string()
            },
            |flowchart| flowchart.merimind_graph_name,
        );

    Some(FlowhubScenarioCaseSummary {
        file_name,
        merimind_graph_name,
    })
}

fn render_scenario_case_section_lines(
    module_ref: &str,
    summaries: &[FlowhubScenarioCaseSummary],
) -> Vec<String> {
    let mut lines = Vec::new();
    extend_scenario_case_summary_lines(&mut lines, module_ref, summaries);
    lines
}

fn render_flowhub_root_module_section_lines(module: &FlowhubModuleSummary) -> Vec<String> {
    let mut tail_blocks = Vec::new();
    if !module.child_modules.is_empty() {
        tail_blocks.push(format!(
            "Children:\n{}",
            module
                .child_modules
                .iter()
                .map(|child| format!("- {child}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    if !module.scenario_cases.is_empty() {
        tail_blocks.push(format!(
            "Scenario cases:\n{}",
            render_scenario_case_section_lines(&module.module_ref, &module.scenario_cases)
                .join("\n")
        ));
    }

    render_embedded_flowhub_block(
        FLOWHUB_ROOT_MODULE_TEMPLATE_NAME,
        json!({
            "path": module.module_dir.display().to_string(),
            "kind": module_kind_label(module.kind),
            "exports_entry": module.exports_entry,
            "exports_ready": module.exports_ready,
            "tail_block": if tail_blocks.is_empty() {
                String::new()
            } else {
                format!("\n{}", tail_blocks.join("\n"))
            },
        }),
    )
    .unwrap_or_else(|error| {
        log::warn!(
            "failed to render Flowhub root module section through qianhuan; falling back to inline format: {error}"
        );
        render_flowhub_root_module_section_lines_inline(module)
    })
}

fn render_flowhub_root_module_section_lines_inline(module: &FlowhubModuleSummary) -> Vec<String> {
    let mut lines = vec![
        format!("Path: {}", module.module_dir.display()),
        format!("Kind: {}", module_kind_label(module.kind)),
        format!(
            "Exports: {} -> {}",
            module.exports_entry, module.exports_ready
        ),
    ];
    if !module.child_modules.is_empty() {
        lines.push("Children:".to_string());
        lines.extend(
            module
                .child_modules
                .iter()
                .map(|child| format!("- {child}")),
        );
    }
    if !module.scenario_cases.is_empty() {
        lines.push("Scenario cases:".to_string());
        extend_scenario_case_summary_lines(&mut lines, &module.module_ref, &module.scenario_cases);
    }
    lines
}

fn render_flowhub_module_exports_section_lines(summary: &FlowhubModuleSummary) -> Vec<String> {
    render_embedded_flowhub_block(
        FLOWHUB_MODULE_EXPORTS_TEMPLATE_NAME,
        json!({
            "entry_export": summary.exports_entry,
            "ready_export": summary.exports_ready,
        }),
    )
    .unwrap_or_else(|error| {
        log::warn!(
            "failed to render Flowhub exports section through qianhuan; falling back to inline format: {error}"
        );
        vec![
            format!("Entry export: {}", summary.exports_entry),
            format!("Ready export: {}", summary.exports_ready),
        ]
    })
}

fn render_flowhub_module_contract_section_lines(show: &FlowhubModuleShow) -> Vec<String> {
    render_embedded_flowhub_block(
        FLOWHUB_MODULE_CONTRACT_TEMPLATE_NAME,
        json!({
            "registered_children": show.registered_child_count,
            "required_contract_entries": show.required_contract_count,
        }),
    )
    .unwrap_or_else(|error| {
        log::warn!(
            "failed to render Flowhub contract section through qianhuan; falling back to inline format: {error}"
        );
        vec![
            format!("Registered children: {}", show.registered_child_count),
            format!("Required contract entries: {}", show.required_contract_count),
        ]
    })
}

fn extend_scenario_case_summary_lines(
    lines: &mut Vec<String>,
    module_ref: &str,
    summaries: &[FlowhubScenarioCaseSummary],
) {
    for (index, summary) in summaries.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        match render_scenario_case_summary_block(module_ref, summary) {
            Ok(rendered) => lines.extend(rendered.lines().map(ToOwned::to_owned)),
            Err(error) => {
                log::warn!(
                    "failed to render Flowhub scenario-case markdown through qianhuan; falling back to inline format: {error}"
                );
                lines.push(format!("Graph name: {}", summary.merimind_graph_name));
                lines.push(format!("Path: ./{module_ref}/{}", summary.file_name));
            }
        }
    }
}

fn render_scenario_case_summary_block(
    module_ref: &str,
    summary: &FlowhubScenarioCaseSummary,
) -> Result<String, QianjiError> {
    render_embedded_flowhub_block(
        FLOWHUB_SCENARIO_CASE_TEMPLATE_NAME,
        json!({
            "merimind_graph_name": summary.merimind_graph_name,
            "path": format!("./{module_ref}/{}", summary.file_name),
        }),
    )
    .map(|lines| lines.join("\n"))
    .map_err(|error| {
        QianjiError::Execution(format!(
            "failed to render Flowhub scenario case `{}`: {error}",
            summary.file_name
        ))
    })
}

fn render_embedded_flowhub_block(
    template_name: &str,
    payload: serde_json::Value,
) -> Result<Vec<String>, String> {
    FLOWHUB_TEMPLATE_CATALOG.render_lines(template_name, payload)
}

fn load_known_module_names_for_show(module_dir: &Path) -> Result<Vec<String>, QianjiError> {
    let Some(root_dir) = module_dir.parent() else {
        return Ok(Vec::new());
    };
    let root_manifest_path = root_dir.join("qianji.toml");
    if !root_manifest_path.is_file() {
        return Ok(Vec::new());
    }

    match load_flowhub_root_manifest(&root_manifest_path) {
        Ok(manifest) => Ok(manifest.contract.register),
        Err(error) => Err(QianjiError::Topology(format!(
            "Failed to load Flowhub root manifest `{}` while summarizing scenario cases: {error}",
            root_manifest_path.display()
        ))),
    }
}

fn module_kind_label(kind: FlowhubModuleKind) -> &'static str {
    match kind {
        FlowhubModuleKind::Composite => "composite",
        FlowhubModuleKind::Leaf => "node",
    }
}

#[cfg(test)]
#[path = "../../tests/unit/flowhub/show.rs"]
mod tests;
