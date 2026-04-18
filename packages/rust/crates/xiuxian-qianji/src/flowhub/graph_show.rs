use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use xiuxian_config_core::resolve_project_root;
use xiuxian_qianhuan::EmbeddedManifestationTemplateCatalog;

use crate::contracts::{FlowhubGraphContract, FlowhubGraphNodeContract, FlowhubGraphTopology};
use crate::error::QianjiError;
use crate::markdown::{MarkdownShowSection, render_show_surface};

use super::discover::{
    FlowhubDiscoveredModule, discover_all_flowhub_module_refs, find_flowhub_root_for_module_dir,
    load_flowhub_module_candidate, module_candidate_from_dir, module_candidate_from_ref,
};
use super::mermaid::{
    MermaidFlowchart, MermaidNodeKind, analyze_mermaid_flowchart_topology,
    declared_graph_node_labels, normalize_graph_node_label, parse_mermaid_flowchart,
    scenario_graph_label_is_allowed,
};

const FLOWHUB_GRAPH_NODE_TEMPLATE_NAME: &str = "flowhub_graph_node_semantics.md.j2";
const FLOWHUB_GRAPH_NODE_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/flowhub_graph_node_semantics.md.j2");

static FLOWHUB_GRAPH_TEMPLATE_CATALOG: EmbeddedManifestationTemplateCatalog =
    EmbeddedManifestationTemplateCatalog::new(
        "Flowhub graph show template renderer",
        &[(
            FLOWHUB_GRAPH_NODE_TEMPLATE_NAME,
            FLOWHUB_GRAPH_NODE_TEMPLATE_SOURCE,
        )],
    );

/// One Flowhub Mermaid graph contract preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphShow {
    /// Mermaid graph file on disk.
    pub graph_path: PathBuf,
    /// Stable graph identity resolved from `[[graph]].name` or the filename
    /// stem fallback.
    pub merimind_graph_name: String,
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
    /// Expected bounded work-surface entries that Codex should materialize.
    pub expected_work_surface: Vec<String>,
    /// Owning module manifest source.
    pub owning_module_manifest_toml: String,
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
        declared_graph,
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
        declared_graph.as_ref(),
    );
    let edges = build_graph_edge_summaries(&flowchart, &nodes_by_id);
    let expected_work_surface = expected_work_surface(&owning_module);

    Ok(FlowhubGraphShow {
        graph_path: graph_path.to_path_buf(),
        merimind_graph_name: flowchart.merimind_graph_name,
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
        expected_work_surface,
        owning_module_manifest_toml,
    })
}

/// Render one Flowhub Mermaid graph contract preview into markdown.
#[must_use]
pub fn render_flowhub_graph_show(show: &FlowhubGraphShow) -> String {
    let sections = vec![
        MarkdownShowSection {
            title: "Mermaid".into(),
            lines: render_mermaid_section_lines(show),
        },
        MarkdownShowSection {
            title: "Nodes".into(),
            lines: render_node_section_lines(&show.nodes),
        },
        MarkdownShowSection {
            title: "Module contract".into(),
            lines: render_expected_work_surface_lines(show),
        },
        MarkdownShowSection {
            title: "Owning qianji.toml".into(),
            lines: render_owning_module_manifest_lines(show),
        },
    ];

    render_show_surface(
        "Graph",
        &[
            format!("Name: {}", show.merimind_graph_name),
            format!("Path: {}", display_graph_path(&show.graph_path)),
            format!("Topology: {}", show.topology.as_str()),
            render_declared_topology_line(show.declared_topology),
        ],
        &sections,
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
    declared_graph: Option<FlowhubGraphContract>,
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
    let declared_topology = declared_graph.as_ref().map(|graph| graph.topology);
    let merimind_graph_name = declared_graph_name(declared_graph.as_ref(), fallback_graph_name);
    let allowed_graph_node_labels = declared_graph_node_labels(declared_graph.as_ref());
    let flowchart =
        parse_mermaid_flowchart(&source, merimind_graph_name.as_str(), &registered_modules)
            .map_err(|error| {
                QianjiError::Topology(format!(
                    "Failed to parse Flowhub Mermaid graph `{}`: {error}",
                    graph_path.display()
                ))
            })?;
    let topology_analysis = analyze_mermaid_flowchart_topology(&flowchart);

    Ok(LoadedFlowhubGraphContext {
        owning_module,
        flowhub_root,
        module_exports,
        owning_module_manifest_toml,
        source,
        flowchart,
        topology: topology_analysis.topology,
        cyclic_components: topology_analysis.cyclic_components,
        declared_graph,
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
    declared_graph: Option<&FlowhubGraphContract>,
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
            let node_contract = declared_graph
                .and_then(|graph| declared_graph_node_contract(graph, node.label.as_str()));
            let (kind, role, agent_action) =
                graph_node_semantics(module_ref.as_deref(), node_contract);

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

fn render_node_section_lines(nodes: &[FlowhubGraphNodeSummary]) -> Vec<String> {
    if nodes.is_empty() {
        return vec!["- none".to_string()];
    }

    let mut lines = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        match render_embedded_graph_block(
            FLOWHUB_GRAPH_NODE_TEMPLATE_NAME,
            json!({
            "label": node.label,
                "kind": node.kind,
                "role": node.role,
                "agent_action": node.agent_action,
                "next": render_next_labels(&node.next),
            }),
        ) {
            Ok(rendered) => lines.extend(rendered),
            Err(error) => {
                log::warn!(
                    "failed to render Flowhub graph node block through qianhuan; falling back to inline format: {error}"
                );
                lines.push(format!("### {}", node.label));
                if let Some(kind) = &node.kind {
                    lines.push(format!("Kind: {kind}"));
                }
                lines.push(format!("Role: {}", node.role));
                lines.push(format!("Agent action: {}", node.agent_action));
                lines.push(format!("Next: {}", render_next_labels(&node.next)));
            }
        }
    }
    lines
}

fn render_expected_work_surface_lines(show: &FlowhubGraphShow) -> Vec<String> {
    show.expected_work_surface
        .iter()
        .map(|entry| format!("- {entry}"))
        .collect()
}

fn render_owning_module_manifest_lines(show: &FlowhubGraphShow) -> Vec<String> {
    let mut lines = vec!["```toml".to_string()];
    lines.extend(
        show.owning_module_manifest_toml
            .lines()
            .map(ToString::to_string),
    );
    lines.push("```".to_string());
    lines
}

fn expected_work_surface(owning_module: &super::discover::FlowhubDiscoveredModule) -> Vec<String> {
    let mut entries = vec!["qianji.toml".to_string()];
    if let Some(contract) = &owning_module.manifest.contract {
        entries.extend(contract.required.iter().cloned());
    }
    entries
}

fn render_embedded_graph_block(
    template_name: &str,
    payload: serde_json::Value,
) -> Result<Vec<String>, String> {
    FLOWHUB_GRAPH_TEMPLATE_CATALOG.render_lines(template_name, payload)
}

fn render_next_labels(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn display_graph_path(path: &Path) -> String {
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

fn declared_graph_name(
    declared_graph: Option<&FlowhubGraphContract>,
    fallback_graph_name: &str,
) -> String {
    declared_graph.map_or_else(
        || fallback_graph_name.to_string(),
        |graph| graph.resolved_name_or(fallback_graph_name).to_string(),
    )
}

fn declared_graph_node_contract<'a>(
    declared_graph: &'a FlowhubGraphContract,
    label: &str,
) -> Option<&'a FlowhubGraphNodeContract> {
    let normalized_label = normalize_graph_node_label(label);
    declared_graph
        .node
        .iter()
        .find(|node| normalize_graph_node_label(node.label.as_str()) == normalized_label)
}

fn graph_node_semantics(
    module_ref: Option<&str>,
    node_contract: Option<&FlowhubGraphNodeContract>,
) -> (Option<String>, String, String) {
    if let Some(node_contract) = node_contract {
        return (
            Some(node_contract.kind.clone()),
            node_contract.role.clone(),
            node_contract.agent_action.clone(),
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
