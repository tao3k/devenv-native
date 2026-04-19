use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use xiuxian_config_core::resolve_project_root;

use crate::contracts::{FlowhubGraphContract, FlowhubGraphSurfaceContract, FlowhubGraphTopology};
use crate::error::QianjiError;
use crate::markdown::{MarkdownShowSection, render_show_surface};

use super::discover::{
    FlowhubDiscoveredModule, discover_all_flowhub_module_refs, find_flowhub_root_for_module_dir,
    load_flowhub_module_candidate, module_candidate_from_dir, module_candidate_from_ref,
};
use super::mermaid::{
    MermaidFlowchart, MermaidNodeKind, analyze_mermaid_flowchart_topology, parse_mermaid_flowchart,
    scenario_graph_label_is_allowed,
};
use super::scenario_ir::{
    FlowhubScenarioIr, FlowhubScenarioNodeIr, compile_flowhub_scenario_ir,
    parse_flowhub_graph_annotations, resolve_flowhub_graph_name,
};

/// One Flowhub Mermaid graph contract preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphShow {
    /// Mermaid graph file on disk.
    pub graph_path: PathBuf,
    /// Stable graph identity resolved from `[[graph]].name` or the filename
    /// stem fallback.
    pub merimind_graph_name: String,
    /// Optional scenario id declared in Mermaid annotations.
    pub scenario_id: Option<String>,
    /// Optional scenario description declared in Mermaid annotations.
    pub description: Option<String>,
    /// Resolved topology from petgraph analysis.
    pub topology: FlowhubGraphTopology,
    /// Optional module-owned declared topology.
    pub declared_topology: Option<FlowhubGraphTopology>,
    /// Raw Mermaid source.
    pub mermaid: String,
    /// Owning Flowhub module reference.
    pub owning_module_ref: String,
    /// Flowhub root containing the owning module.
    pub flowhub_root: PathBuf,
    /// Declared Mermaid direction such as `LR`.
    pub direction: String,
    /// Parsed nodes with semantic guidance in declaration order.
    pub nodes: Vec<FlowhubGraphNodeSummary>,
    /// Parsed edges in declaration order.
    pub edges: Vec<FlowhubGraphEdgeSummary>,
    /// Registered Flowhub modules that are missing from the Mermaid graph.
    pub missing_registered_modules: Vec<String>,
    /// Mermaid nodes outside the registered-module set and allowed graph vocabulary.
    pub unknown_graph_nodes: Vec<String>,
    /// Node labels grouped by cyclic SCC when the graph loops.
    pub cyclic_components: Vec<Vec<String>>,
    /// Static Flowhub module-owned contract entries for the graph source.
    pub module_contract_surface: Vec<String>,
    /// Declared bounded check surface for executor guidance.
    pub declared_check_surface: FlowhubGraphCheckSurface,
    /// Owning module manifest source.
    pub owning_module_manifest_toml: String,
}

/// Declared bounded check surface derived from one graph contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphCheckSurface {
    /// Optional note that explains how the localized surface should be used.
    pub note: Option<String>,
    /// Optional localized run root declared by the Flowhub contract.
    pub root: Option<String>,
    /// Raw `check.require` paths or globs declared by Flowhub.
    pub required_paths: Vec<String>,
    /// Raw `check.flowchart` surfaces declared by Flowhub.
    pub flowchart_surfaces: Vec<String>,
    /// Optional persistent canonical target tree for validated merges.
    pub persistent_target_surface_tree: Vec<String>,
    /// Optional declared done-gate paths over the persistent target surface.
    pub done_gate_require: Vec<String>,
}

/// One parsed Flowhub Mermaid node summary with semantic guidance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphNodeSummary {
    /// Stable Mermaid node id.
    pub id: String,
    /// Visible Mermaid label.
    pub label: String,
    /// Contract-owned node semantic kind when declared.
    pub kind: Option<String>,
    /// Stable role description for Codex.
    pub role: String,
    /// Stable agent action guidance for the node.
    pub agent_action: String,
    /// Visible next-node labels in edge order.
    pub next: Vec<String>,
    /// Resolved Flowhub module ref when the node represents a registered module.
    pub module_ref: Option<String>,
    /// Stable module entry export when available.
    pub exports_entry: Option<String>,
    /// Stable module ready export when available.
    pub exports_ready: Option<String>,
}

/// One extracted graph edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphEdgeSummary {
    /// Edge source label.
    pub from_label: String,
    /// Edge destination label.
    pub to_label: String,
}

/// Load and summarize one Flowhub Mermaid graph file.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the target is not a Mermaid file
/// owned by a Flowhub module or when the Flowhub manifests cannot be loaded.
pub fn show_flowhub_graph(graph_path: impl AsRef<Path>) -> Result<FlowhubGraphShow, QianjiError> {
    let graph_path = graph_path.as_ref();
    validate_graph_path(graph_path)?;
    let LoadedFlowhubGraphContext {
        owning_module,
        flowhub_root,
        module_exports,
        owning_module_manifest_toml,
        source,
        flowchart,
        topology,
        cyclic_components,
        scenario_ir,
        declared_topology,
        allowed_graph_node_labels,
    } = load_flowhub_graph_context(graph_path)?;

    let nodes_by_id = flowchart
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.label.as_str()))
        .collect::<BTreeMap<_, _>>();
    let unknown_graph_nodes = collect_unknown_graph_nodes(&flowchart, &allowed_graph_node_labels);
    let next_by_node_id = build_next_labels_by_node_id(&flowchart.edges, &nodes_by_id);
    let nodes = build_graph_node_summaries(
        &flowchart,
        &module_exports,
        &next_by_node_id,
        scenario_ir.as_ref(),
    );
    let edges = build_graph_edge_summaries(&flowchart, &nodes_by_id);
    let module_contract_surface = module_contract_surface(&owning_module);
    let declared_check_surface = declared_check_surface(scenario_ir.as_ref());

    Ok(FlowhubGraphShow {
        graph_path: graph_path.to_path_buf(),
        merimind_graph_name: flowchart.merimind_graph_name,
        scenario_id: scenario_ir
            .as_ref()
            .and_then(|graph| graph.scenario_id.clone()),
        description: scenario_ir
            .as_ref()
            .and_then(|graph| graph.description.clone()),
        topology,
        declared_topology,
        mermaid: source,
        owning_module_ref: owning_module.module_ref,
        flowhub_root,
        direction: flowchart.direction,
        nodes,
        edges,
        missing_registered_modules: Vec::new(),
        unknown_graph_nodes,
        cyclic_components,
        module_contract_surface,
        declared_check_surface,
        owning_module_manifest_toml,
    })
}

pub(crate) fn load_flowhub_graph_runtime_contract(
    graph_path: &Path,
) -> Result<(MermaidFlowchart, Option<FlowhubScenarioIr>), QianjiError> {
    let LoadedFlowhubGraphContext {
        flowchart,
        scenario_ir,
        ..
    } = load_flowhub_graph_context(graph_path)?;
    Ok((flowchart, scenario_ir))
}

/// Render one Flowhub Mermaid graph contract preview into markdown.
#[must_use]
pub fn render_flowhub_graph_show(show: &FlowhubGraphShow) -> String {
    render_show_surface(
        "Graph",
        &graph_show_metadata_lines(show),
        &graph_show_sections(show),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleExports {
    entry: String,
    ready: String,
}

#[derive(Debug)]
struct LoadedFlowhubGraphContext {
    owning_module: FlowhubDiscoveredModule,
    flowhub_root: PathBuf,
    module_exports: BTreeMap<String, ModuleExports>,
    owning_module_manifest_toml: String,
    source: String,
    flowchart: MermaidFlowchart,
    topology: FlowhubGraphTopology,
    cyclic_components: Vec<Vec<String>>,
    scenario_ir: Option<FlowhubScenarioIr>,
    declared_topology: Option<FlowhubGraphTopology>,
    allowed_graph_node_labels: BTreeSet<String>,
}

fn validate_graph_path(graph_path: &Path) -> Result<(), QianjiError> {
    if !graph_path.is_file() {
        return Err(QianjiError::Topology(format!(
            "`{}` is not a Mermaid graph file",
            graph_path.display()
        )));
    }
    if graph_path
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("mmd")
    {
        return Err(QianjiError::Topology(format!(
            "`{}` is not a `.mmd` graph file",
            graph_path.display()
        )));
    }
    Ok(())
}

fn load_flowhub_graph_context(graph_path: &Path) -> Result<LoadedFlowhubGraphContext, QianjiError> {
    let module_dir = graph_path.parent().ok_or_else(|| {
        QianjiError::Topology(format!(
            "Flowhub Mermaid graph `{}` has no parent module directory",
            graph_path.display()
        ))
    })?;
    let module_candidate = module_candidate_from_dir(module_dir)?;
    let owning_module = load_flowhub_module_candidate(&module_candidate)?;
    let flowhub_root = find_flowhub_root_for_module_dir(module_dir)?;
    let registered_modules = discover_all_flowhub_module_refs(&flowhub_root)?;
    let module_exports = load_registered_module_exports(&flowhub_root, &registered_modules)?;
    let owning_module_manifest_toml =
        fs::read_to_string(&owning_module.manifest_path).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read Flowhub module manifest `{}`: {error}",
                owning_module.manifest_path.display()
            ))
        })?;
    let source = fs::read_to_string(graph_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub Mermaid graph `{}`: {error}",
            graph_path.display()
        ))
    })?;
    let fallback_graph_name = graph_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| {
            QianjiError::Topology(format!(
                "Failed to derive Mermaid graph name from `{}`",
                graph_path.display()
            ))
        })?;
    let declared_graph = declared_graph_contract(&owning_module, graph_path).cloned();
    let annotations = parse_flowhub_graph_annotations(&source).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to parse Flowhub Mermaid annotations from `{}`: {error}",
            graph_path.display()
        ))
    })?;
    let merimind_graph_name = resolve_flowhub_graph_name(
        annotations.as_ref(),
        declared_graph.as_ref(),
        fallback_graph_name,
    );
    let flowchart =
        parse_mermaid_flowchart(&source, merimind_graph_name.as_str(), &registered_modules)
            .map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to parse Flowhub Mermaid graph `{}`: {error}",
                    graph_path.display()
                ))
            })?;
    let scenario_ir = compile_flowhub_scenario_ir(
        graph_path,
        merimind_graph_name.as_str(),
        &flowchart,
        annotations.as_ref(),
        declared_graph.as_ref(),
    )?;
    let topology_analysis = analyze_mermaid_flowchart_topology(&flowchart);
    let declared_topology = scenario_ir
        .as_ref()
        .and_then(|graph| graph.declared_topology);
    let allowed_graph_node_labels = scenario_ir
        .as_ref()
        .map_or_else(BTreeSet::new, FlowhubScenarioIr::allowed_graph_node_labels);

    Ok(LoadedFlowhubGraphContext {
        owning_module,
        flowhub_root,
        module_exports,
        owning_module_manifest_toml,
        source,
        flowchart,
        topology: topology_analysis.topology,
        cyclic_components: topology_analysis.cyclic_components,
        scenario_ir,
        declared_topology,
        allowed_graph_node_labels,
    })
}

fn load_registered_module_exports(
    flowhub_root: &Path,
    registered_modules: &[String],
) -> Result<BTreeMap<String, ModuleExports>, QianjiError> {
    registered_modules
        .iter()
        .map(|module_ref| {
            let module = load_flowhub_module_candidate(&module_candidate_from_ref(
                flowhub_root,
                module_ref,
            ))?;
            Ok((
                module_ref.clone(),
                ModuleExports {
                    entry: module.manifest.exports.entry,
                    ready: module.manifest.exports.ready,
                },
            ))
        })
        .collect()
}

fn collect_unknown_graph_nodes(
    flowchart: &MermaidFlowchart,
    allowed_graph_node_labels: &BTreeSet<String>,
) -> Vec<String> {
    flowchart
        .nodes
        .iter()
        .filter(|node| node.kind != MermaidNodeKind::Module)
        .filter(|node| {
            !scenario_graph_label_is_allowed(node.label.as_str(), allowed_graph_node_labels)
        })
        .map(|node| node.label.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn build_next_labels_by_node_id<'a>(
    edges: &'a [super::mermaid::MermaidEdge],
    nodes_by_id: &BTreeMap<&'a str, &'a str>,
) -> BTreeMap<&'a str, Vec<String>> {
    let mut next_by_node_id = BTreeMap::<&str, Vec<String>>::new();

    for edge in edges {
        let next_label = nodes_by_id
            .get(edge.to.as_str())
            .copied()
            .unwrap_or(edge.to.as_str())
            .to_string();
        let entry = next_by_node_id.entry(edge.from.as_str()).or_default();
        if !entry.contains(&next_label) {
            entry.push(next_label);
        }
    }

    next_by_node_id
}

fn build_graph_node_summaries(
    flowchart: &MermaidFlowchart,
    module_exports: &BTreeMap<String, ModuleExports>,
    next_by_node_id: &BTreeMap<&str, Vec<String>>,
    scenario_ir: Option<&FlowhubScenarioIr>,
) -> Vec<FlowhubGraphNodeSummary> {
    flowchart
        .nodes
        .iter()
        .map(|node| {
            let module_ref = match node.kind {
                MermaidNodeKind::Module => Some(node.label.clone()),
                MermaidNodeKind::Scenario => None,
            };
            let exports = module_ref
                .as_deref()
                .and_then(|module_name| module_exports.get(module_name));
            let node_contract =
                scenario_ir.and_then(|graph| graph.node_contract(node.label.as_str()));
            let (kind, role, agent_action) =
                graph_node_semantics(module_ref.as_deref(), node.label.as_str(), node_contract);

            FlowhubGraphNodeSummary {
                id: node.id.clone(),
                label: node.label.clone(),
                kind,
                role,
                agent_action,
                next: next_by_node_id
                    .get(node.id.as_str())
                    .cloned()
                    .unwrap_or_default(),
                module_ref,
                exports_entry: exports.map(|value| value.entry.clone()),
                exports_ready: exports.map(|value| value.ready.clone()),
            }
        })
        .collect()
}

fn build_graph_edge_summaries(
    flowchart: &MermaidFlowchart,
    nodes_by_id: &BTreeMap<&str, &str>,
) -> Vec<FlowhubGraphEdgeSummary> {
    flowchart
        .edges
        .iter()
        .map(|edge| FlowhubGraphEdgeSummary {
            from_label: nodes_by_id
                .get(edge.from.as_str())
                .copied()
                .unwrap_or(edge.from.as_str())
                .to_string(),
            to_label: nodes_by_id
                .get(edge.to.as_str())
                .copied()
                .unwrap_or(edge.to.as_str())
                .to_string(),
        })
        .collect()
}

fn render_mermaid_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
    let mut lines = vec!["```mermaid".to_string()];
    lines.extend(show.mermaid.lines().map(ToString::to_string));
    lines.push("```".to_string());
    lines
}

fn render_declared_topology_line(topology: Option<FlowhubGraphTopology>) -> String {
    match topology {
        Some(value) => format!("Declared topology: {}", value.as_str()),
        None => "Declared topology: (none)".to_string(),
    }
}

fn render_execution_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
    render_execution_summary_lines(show)
}

pub(crate) fn graph_show_metadata_lines(show: &FlowhubGraphShow) -> Vec<String> {
    vec![
        format!("Name: {}", show.merimind_graph_name),
        format!("Path: {}", display_graph_path(&show.graph_path)),
        format!("Owning module: {}", show.owning_module_ref),
        format!("Direction: {}", show.direction),
        format!("Topology: {}", show.topology.as_str()),
        render_declared_topology_line(show.declared_topology),
    ]
}

pub(crate) fn graph_show_sections(show: &FlowhubGraphShow) -> Vec<MarkdownShowSection<'_>> {
    let mut sections = vec![
        MarkdownShowSection {
            title: "Execution".into(),
            lines: render_execution_section_lines(show),
        },
        MarkdownShowSection {
            title: "Nodes".into(),
            lines: render_node_section_lines(show),
        },
        MarkdownShowSection {
            title: "Check Surface".into(),
            lines: render_check_surface_section_lines(show),
        },
    ];

    if !show
        .declared_check_surface
        .persistent_target_surface_tree
        .is_empty()
    {
        sections.push(MarkdownShowSection {
            title: "Persistent Target Surface".into(),
            lines: render_persistent_target_surface_section_lines(show),
        });
    }

    if !show.declared_check_surface.done_gate_require.is_empty() {
        sections.push(MarkdownShowSection {
            title: "Done Gate".into(),
            lines: render_done_gate_section_lines(show),
        });
    }
    sections.push(MarkdownShowSection {
        title: "Mermaid".into(),
        lines: render_mermaid_section_lines(show),
    });

    sections
}

fn module_contract_surface(
    owning_module: &super::discover::FlowhubDiscoveredModule,
) -> Vec<String> {
    let mut entries = vec!["qianji.toml".to_string()];
    if let Some(contract) = &owning_module.manifest.contract {
        entries.extend(contract.required.iter().cloned());
    }
    entries
}

fn render_execution_summary_lines(show: &FlowhubGraphShow) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(note) = &show.declared_check_surface.note {
        lines.push(format!("- {note}"));
    }

    let entry_nodes = entry_node_labels(show);
    if !entry_nodes.is_empty() {
        lines.push(format!("- Start at {}.", render_label_list(&entry_nodes)));
    }

    let terminal_nodes = terminal_node_labels(show);
    if !terminal_nodes.is_empty() {
        lines.push(format!(
            "- Complete at {}.",
            render_label_list(&terminal_nodes)
        ));
    }

    if !show.cyclic_components.is_empty() {
        let loop_lines = show
            .cyclic_components
            .iter()
            .map(|component| render_label_list(component))
            .collect::<Vec<_>>()
            .join("; ");
        lines.push(format!("- Retry or loop components: {loop_lines}."));
    }

    lines
}

fn render_node_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
    if show.nodes.is_empty() {
        return vec!["- none".to_string()];
    }

    let mut lines = show
        .nodes
        .iter()
        .map(render_execution_node_line)
        .collect::<Vec<_>>();
    if !show.unknown_graph_nodes.is_empty() {
        lines.push(format!(
            "- Undeclared graph nodes: {}.",
            render_label_list(&show.unknown_graph_nodes)
        ));
    }
    lines
}

fn entry_node_labels(show: &FlowhubGraphShow) -> Vec<String> {
    let targets = show
        .edges
        .iter()
        .map(|edge| edge.to_label.as_str())
        .collect::<BTreeSet<_>>();

    show.nodes
        .iter()
        .filter(|node| !targets.contains(node.label.as_str()))
        .map(|node| node.label.clone())
        .collect()
}

fn terminal_node_labels(show: &FlowhubGraphShow) -> Vec<String> {
    show.nodes
        .iter()
        .filter(|node| node.next.is_empty())
        .map(|node| node.label.clone())
        .collect()
}

fn render_execution_node_line(node: &FlowhubGraphNodeSummary) -> String {
    let mut prefix = format!("- `{}`", node.label);
    if let Some(kind) = &node.kind {
        let _ = write!(prefix, " [`{kind}`]");
    }

    let mut detail = format!("{} Action: {}", node.role, node.agent_action);
    let _ = write!(detail, ". Next: {}", render_label_list(&node.next));
    if let Some(entry) = &node.exports_entry {
        let _ = write!(detail, ". Entry: `{entry}`");
    }
    if let Some(ready) = &node.exports_ready {
        let _ = write!(detail, ". Ready: `{ready}`");
    }

    format!("{prefix}: {detail}")
}

fn render_check_surface_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
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

fn render_persistent_target_surface_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
    let mut lines = vec![
        "- Merge validated staging artifacts into this canonical surface after `qianji check` passes."
            .to_string(),
    ];
    lines.extend(render_text_block(
        &show.declared_check_surface.persistent_target_surface_tree,
    ));
    lines
}

fn render_done_gate_section_lines(show: &FlowhubGraphShow) -> Vec<String> {
    let mut lines =
        vec!["- Completion remains blocked until these declared paths are satisfied:".to_string()];
    lines.extend(render_text_block(
        &show.declared_check_surface.done_gate_require,
    ));
    lines
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

fn render_label_list(values: &[String]) -> String {
    if values.is_empty() {
        return "`none`".to_string();
    }

    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_file_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn declared_check_surface(scenario_ir: Option<&FlowhubScenarioIr>) -> FlowhubGraphCheckSurface {
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

fn declared_graph_contract<'a>(
    owning_module: &'a FlowhubDiscoveredModule,
    graph_path: &Path,
) -> Option<&'a FlowhubGraphContract> {
    let file_name = graph_path.file_name()?.to_str()?;
    owning_module
        .manifest
        .graph
        .iter()
        .find(|graph| graph.path == file_name)
}

fn graph_node_semantics(
    module_ref: Option<&str>,
    label: &str,
    node_contract: Option<&FlowhubScenarioNodeIr>,
) -> (Option<String>, String, String) {
    if let Some(node_contract) = node_contract {
        return (
            node_contract.kind.clone(),
            node_contract
                .role
                .clone()
                .unwrap_or_else(|| inferred_graph_node_role(label, node_contract)),
            node_contract
                .agent_action
                .clone()
                .unwrap_or_else(|| inferred_graph_node_action(node_contract)),
        );
    }

    if module_ref.is_some() {
        return (
            None,
            "registered Flowhub module is present in the Mermaid graph without a declared graph-node contract"
                .to_string(),
            "add a matching `[[graph.node]]` entry before relying on semantic guidance for this module node"
                .to_string(),
        );
    }

    (
        None,
        "node is outside the declared Flowhub graph contract vocabulary".to_string(),
        "do not rely on this node until the Flowhub graph contract is corrected".to_string(),
    )
}

fn inferred_graph_node_role(label: &str, node_contract: &FlowhubScenarioNodeIr) -> String {
    if node_contract.kind.as_deref() == Some("gate") || label == "done gate" {
        return "allow completion only when the declared graph-step contracts are satisfied"
            .to_string();
    }
    if label == "diagnostics" {
        return "capture blocking diagnostics for bounded-surface repair".to_string();
    }
    if !node_contract.merge_target.is_empty() {
        return "materialize localized outputs that can be merged into the persistent target surface"
            .to_string();
    }
    if !node_contract.writes.is_empty() || node_contract.checkpoint.is_some() {
        return "materialize localized bounded-work artifacts for this graph step".to_string();
    }
    "follow the declared graph-step contract".to_string()
}

fn inferred_graph_node_action(node_contract: &FlowhubScenarioNodeIr) -> String {
    let mut parts = Vec::new();
    if let Some(checkpoint) = &node_contract.checkpoint {
        parts.push(format!("write checkpoint `{checkpoint}`"));
    }
    if !node_contract.writes.is_empty() {
        parts.push(format!(
            "write localized artifacts {}",
            render_file_list(&node_contract.writes)
        ));
    }
    if !node_contract.merge_target.is_empty() {
        parts.push(format!(
            "prepare canonical merge targets {}",
            render_file_list(&node_contract.merge_target)
        ));
    }

    if parts.is_empty() {
        "follow the declared node contract".to_string()
    } else {
        parts.join("; ")
    }
}
