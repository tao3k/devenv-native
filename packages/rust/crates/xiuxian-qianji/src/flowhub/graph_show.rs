use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use xiuxian_config_core::resolve_project_root;
use xiuxian_qianhuan::EmbeddedManifestationTemplateCatalog;

use crate::contracts::{FlowhubGraphContract, FlowhubGraphTopology};
use crate::error::QianjiError;
use crate::markdown::{MarkdownShowSection, render_show_surface};

use super::discover::{
    FlowhubDiscoveredModule, find_flowhub_root_for_module_dir, load_flowhub_module_candidate,
    module_candidate_from_dir, module_candidate_from_ref,
};
use super::load::load_flowhub_root_manifest;
use super::mermaid::{
    MermaidFlowchart, MermaidNodeKind, analyze_mermaid_flowchart_topology, parse_mermaid_flowchart,
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
    /// Stable graph kind shown to Codex.
    pub kind: String,
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
    /// Classified node kind.
    pub kind: FlowhubGraphNodeKind,
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

/// Semantic node-kind classification for the graph-contract surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowhubGraphNodeKind {
    /// Upstream scope or lane selector.
    Context,
    /// Constraint projected into writable artifacts.
    Constraint,
    /// Writable bounded artifact surface.
    Artifact,
    /// Contract guard over artifact state.
    Guard,
    /// Validator requirement before completion.
    Validator,
    /// Completion gate.
    Gate,
    /// Process or repair-loop step.
    Process,
    /// Node outside the known v0 graph contract vocabulary.
    Unknown,
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
        declared_topology,
    } = load_flowhub_graph_context(graph_path)?;

    let nodes_by_id = flowchart
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node.label.as_str()))
        .collect::<BTreeMap<_, _>>();
    let unknown_graph_nodes = collect_unknown_graph_nodes(&flowchart);
    let next_by_node_id = build_next_labels_by_node_id(&flowchart.edges, &nodes_by_id);
    let nodes = build_graph_node_summaries(&flowchart, &module_exports, &next_by_node_id);
    let edges = build_graph_edge_summaries(&flowchart, &nodes_by_id);
    let expected_work_surface = expected_work_surface(&owning_module);

    Ok(FlowhubGraphShow {
        graph_path: graph_path.to_path_buf(),
        merimind_graph_name: flowchart.merimind_graph_name,
        kind: "scenario".to_string(),
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
            format!("Kind: {}", show.kind),
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
    declared_topology: Option<FlowhubGraphTopology>,
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
    let root_manifest = load_flowhub_root_manifest(flowhub_root.join("qianji.toml"))?;
    let registered_modules = root_manifest.contract.register;
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
    let declared_topology =
        declared_graph_contract(&owning_module, graph_path).map(|graph| graph.topology);
    let merimind_graph_name = declared_graph_name(&owning_module, graph_path, fallback_graph_name);
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
        declared_topology,
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

fn collect_unknown_graph_nodes(flowchart: &MermaidFlowchart) -> Vec<String> {
    flowchart
        .nodes
        .iter()
        .filter(|node| node.kind != MermaidNodeKind::Module)
        .filter(|node| !scenario_graph_label_is_allowed(node.label.as_str()))
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
            let is_allowed_graph_node = scenario_graph_label_is_allowed(node.label.as_str());
            let kind = classify_graph_node_kind(
                node.label.as_str(),
                module_ref.as_deref(),
                is_allowed_graph_node,
            );

            FlowhubGraphNodeSummary {
                id: node.id.clone(),
                label: node.label.clone(),
                kind,
                role: graph_node_role(node.label.as_str(), kind).to_string(),
                agent_action: graph_node_agent_action(node.label.as_str(), kind).to_string(),
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
                "kind": graph_node_kind_label(node.kind),
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
                lines.push(format!("Kind: {}", graph_node_kind_label(node.kind)));
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
    owning_module: &FlowhubDiscoveredModule,
    graph_path: &Path,
    fallback_graph_name: &str,
) -> String {
    declared_graph_contract(owning_module, graph_path).map_or_else(
        || fallback_graph_name.to_string(),
        |graph| graph.resolved_name_or(fallback_graph_name).to_string(),
    )
}

fn classify_graph_node_kind(
    label: &str,
    module_ref: Option<&str>,
    is_allowed_graph_node: bool,
) -> FlowhubGraphNodeKind {
    let normalized = canonicalize_node_label(label);
    if module_ref.is_none() && !is_allowed_graph_node {
        return FlowhubGraphNodeKind::Unknown;
    }
    match normalized.as_str() {
        "coding" | "rust" => FlowhubGraphNodeKind::Context,
        "style" | "engineering_requirement" | "policy" => FlowhubGraphNodeKind::Constraint,
        "blueprint" | "plan" => FlowhubGraphNodeKind::Artifact,
        "surface_check"
        | "flowchart_alignment"
        | "boundary_check"
        | "drift_check"
        | "boundary_and_drift_check"
        | "status_legality" => FlowhubGraphNodeKind::Guard,
        "validator_gate" | "domain_validators" => FlowhubGraphNodeKind::Validator,
        "done_gate" => FlowhubGraphNodeKind::Gate,
        "codex_write_bounded_surface" | "diagnostics" => FlowhubGraphNodeKind::Process,
        _ if is_exact_http_request_label(label) => FlowhubGraphNodeKind::Process,
        _ if module_ref.is_some() => FlowhubGraphNodeKind::Context,
        _ => FlowhubGraphNodeKind::Unknown,
    }
}

fn graph_node_role(label: &str, kind: FlowhubGraphNodeKind) -> &'static str {
    if kind == FlowhubGraphNodeKind::Unknown {
        return "node is outside the known v0 Flowhub graph contract vocabulary";
    }
    let normalized = canonicalize_node_label(label);
    match normalized.as_str() {
        "coding" => "define the top-level coding lane",
        "rust" => "define the rust specialization lane",
        "style" => "define style requirements for the rust lane",
        "engineering_requirement" => "define engineering requirements for the rust lane",
        "policy" => "define policy requirements for the rust lane",
        "blueprint" => "define the bounded blueprint surface",
        "plan" => "define the bounded execution surface",
        "surface_check" => "require the bounded work surface to exist",
        "flowchart_alignment" => {
            "ensure the flowchart matches the current bounded artifact surface"
        }
        "boundary_check" => "ensure work stays inside the bounded plan surface",
        "drift_check" => "ensure the artifact state has not drifted from the graph contract",
        "boundary_and_drift_check" => {
            "ensure the bounded artifact state remains inside contract boundaries without drift"
        }
        "status_legality" => "ensure status state remains legal for the bounded slice",
        "validator_gate" => "require programmatic validators to pass",
        "domain_validators" => "require domain validators to pass before completion",
        "done_gate" => "allow completion only when required guards and validators pass",
        "codex_write_bounded_surface" => "write the bounded work surface from the graph contract",
        "diagnostics" => "capture blocking diagnostics for bounded-surface repair",
        _ if is_exact_http_request_label(label) => {
            "invoke the exact HTTP request surface carried by this graph node"
        }
        _ => match kind {
            FlowhubGraphNodeKind::Context => "define upstream context for the bounded slice",
            FlowhubGraphNodeKind::Constraint => {
                "define a constraint that must be projected into writable artifacts"
            }
            FlowhubGraphNodeKind::Artifact => "define one bounded writable artifact surface",
            FlowhubGraphNodeKind::Guard => "guard artifact-state correctness",
            FlowhubGraphNodeKind::Validator => "require validator success before completion",
            FlowhubGraphNodeKind::Gate => "gate bounded-slice completion",
            FlowhubGraphNodeKind::Process => "express a repair or execution step in the graph",
            FlowhubGraphNodeKind::Unknown => unreachable!("unknown handled above"),
        },
    }
}

fn graph_node_agent_action(label: &str, kind: FlowhubGraphNodeKind) -> &'static str {
    if kind == FlowhubGraphNodeKind::Unknown {
        return "do not rely on this node until the Flowhub graph contract is corrected";
    }
    let normalized = canonicalize_node_label(label);
    match normalized.as_str() {
        "coding" => "treat as upstream scope, not a writable artifact",
        "rust" => "reflect rust constraints into blueprint and plan surfaces",
        "style" => "encode style requirements into blueprint/ and plan/",
        "engineering_requirement" => "encode engineering requirements into blueprint/ and plan/",
        "policy" => "encode policy requirements into blueprint/ and plan/",
        "blueprint" => "create and populate blueprint/",
        "plan" => "create and populate plan/",
        "surface_check" => "ensure qianji.toml, flowchart.mmd, blueprint/, and plan/ exist",
        "flowchart_alignment" => "keep flowchart.mmd aligned with blueprint and plan",
        "boundary_check" => "do not create or mutate artifacts outside the allowed work surface",
        "drift_check" => "keep blueprint/ and plan/ consistent with the shown graph",
        "boundary_and_drift_check" => {
            "keep blueprint/ and plan/ inside the bounded surface and consistent with the shown graph"
        }
        "status_legality" => "keep bounded-slice status legal before calling the slice complete",
        "validator_gate" => "prepare the artifact state so required validators can succeed",
        "domain_validators" => {
            "prepare the artifact state so required domain validators can succeed"
        }
        "done_gate" => "do not treat the slice as complete before qianji check passes",
        "codex_write_bounded_surface" => {
            "write qianji.toml, flowchart.mmd, blueprint/, and plan/ for the bounded slice"
        }
        "diagnostics" => "use diagnostics to repair the bounded work surface before retrying",
        _ if is_exact_http_request_label(label) => {
            "resolve placeholders and call this HTTP surface exactly as written"
        }
        _ => match kind {
            FlowhubGraphNodeKind::Context => {
                "treat as upstream scope and project it into writable artifacts"
            }
            FlowhubGraphNodeKind::Constraint => "project the constraint into blueprint/ and plan/",
            FlowhubGraphNodeKind::Artifact => "create and populate the writable artifact surface",
            FlowhubGraphNodeKind::Guard => {
                "keep the bounded artifact state aligned with this guard"
            }
            FlowhubGraphNodeKind::Validator => {
                "prepare the bounded artifact state so validators can succeed"
            }
            FlowhubGraphNodeKind::Gate => "do not advance completion until the gate is satisfied",
            FlowhubGraphNodeKind::Process => {
                "follow this process step when materializing or repairing the bounded surface"
            }
            FlowhubGraphNodeKind::Unknown => unreachable!("unknown handled above"),
        },
    }
}

fn is_exact_http_request_label(label: &str) -> bool {
    let Some((method, target)) = label.split_once(' ') else {
        return false;
    };
    matches!(method, "GET" | "POST" | "PUT" | "PATCH" | "DELETE")
        && !target.is_empty()
        && target.starts_with('/')
        && !target.contains(' ')
}

fn graph_node_kind_label(kind: FlowhubGraphNodeKind) -> &'static str {
    match kind {
        FlowhubGraphNodeKind::Context => "context",
        FlowhubGraphNodeKind::Constraint => "constraint",
        FlowhubGraphNodeKind::Artifact => "artifact",
        FlowhubGraphNodeKind::Guard => "guard",
        FlowhubGraphNodeKind::Validator => "validator",
        FlowhubGraphNodeKind::Gate => "gate",
        FlowhubGraphNodeKind::Process => "process",
        FlowhubGraphNodeKind::Unknown => "unknown",
    }
}

fn canonicalize_node_label(label: &str) -> String {
    label
        .chars()
        .map(|character| match character {
            'A'..='Z' => character.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' => character,
            _ => '_',
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}
