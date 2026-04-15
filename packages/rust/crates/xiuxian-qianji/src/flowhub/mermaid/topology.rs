use std::collections::{BTreeMap, BTreeSet};

use petgraph::Directed;
use petgraph::algo::kosaraju_scc;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use petgraph::visit::{Dfs, EdgeRef, IntoEdgeReferences};

use crate::contracts::FlowhubGraphTopology;

use super::MermaidFlowchart;

/// Petgraph-backed topology summary for one Flowhub Mermaid graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MermaidTopologyAnalysis {
    /// Resolved graph classification.
    pub(crate) topology: FlowhubGraphTopology,
    /// Node labels grouped by cyclic SCC in stable sorted order.
    pub(crate) cyclic_components: Vec<Vec<String>>,
}

type MermaidLabelGraph = StableGraph<String, (), Directed>;
type MermaidComponentGraph = StableGraph<(), (), Directed>;
type MermaidCyclicComponents = Vec<(Vec<String>, bool)>;

pub(crate) fn analyze_mermaid_flowchart_topology(
    flowchart: &MermaidFlowchart,
) -> MermaidTopologyAnalysis {
    let (_graph, component_graph, cyclic_components) = build_topology_state(flowchart);
    let cyclic_component_indexes = cyclic_components
        .iter()
        .enumerate()
        .filter_map(|(index, (_, is_cyclic))| (*is_cyclic).then_some(index))
        .collect::<Vec<_>>();

    if cyclic_component_indexes.is_empty() {
        return MermaidTopologyAnalysis {
            topology: FlowhubGraphTopology::Dag,
            cyclic_components: Vec::new(),
        };
    }

    let component_nodes = component_graph.node_indices().collect::<Vec<_>>();
    let component_index_by_node = component_nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (*node, index))
        .collect::<BTreeMap<_, _>>();
    let exit_components = component_nodes
        .iter()
        .enumerate()
        .filter(|(index, node)| {
            !cyclic_components[*index].1
                && component_graph
                    .neighbors_directed(**node, petgraph::Direction::Outgoing)
                    .next()
                    .is_none()
        })
        .map(|(index, _)| index)
        .collect::<BTreeSet<_>>();

    let every_cycle_has_exit = cyclic_component_indexes.iter().copied().all(|index| {
        let mut dfs = Dfs::new(&component_graph, component_nodes[index]);
        while let Some(component_node) = dfs.next(&component_graph) {
            let reachable_index = component_index_by_node[&component_node];
            if reachable_index != index && exit_components.contains(&reachable_index) {
                return true;
            }
        }
        false
    });

    MermaidTopologyAnalysis {
        topology: if every_cycle_has_exit {
            FlowhubGraphTopology::BoundedLoop
        } else {
            FlowhubGraphTopology::OpenLoop
        },
        cyclic_components: cyclic_components
            .into_iter()
            .filter_map(|(labels, is_cyclic)| is_cyclic.then_some(labels))
            .collect(),
    }
}

fn build_topology_state(
    flowchart: &MermaidFlowchart,
) -> (
    MermaidLabelGraph,
    MermaidComponentGraph,
    MermaidCyclicComponents,
) {
    let mut graph = MermaidLabelGraph::new();
    let mut node_indices = BTreeMap::<&str, NodeIndex>::new();
    for node in &flowchart.nodes {
        let index = graph.add_node(node.label.clone());
        node_indices.insert(node.id.as_str(), index);
    }
    for edge in &flowchart.edges {
        if let (Some(from), Some(to)) = (
            node_indices.get(edge.from.as_str()),
            node_indices.get(edge.to.as_str()),
        ) {
            graph.add_edge(*from, *to, ());
        }
    }

    let sccs = kosaraju_scc(&graph);
    let mut component_graph = MermaidComponentGraph::new();
    let component_nodes = sccs
        .iter()
        .map(|_| component_graph.add_node(()))
        .collect::<Vec<_>>();
    let mut component_by_node = BTreeMap::<NodeIndex, usize>::new();
    let mut cyclic_components = Vec::with_capacity(sccs.len());

    for (component_index, component) in sccs.iter().enumerate() {
        for node_index in component {
            component_by_node.insert(*node_index, component_index);
        }
        let mut labels = component
            .iter()
            .map(|node_index| graph[*node_index].clone())
            .collect::<Vec<_>>();
        labels.sort();
        labels.dedup();
        cyclic_components.push((labels, component_is_cyclic(&graph, component)));
    }

    let mut added_edges = BTreeSet::new();
    for edge in graph.edge_references() {
        let from_component = component_by_node[&edge.source()];
        let to_component = component_by_node[&edge.target()];
        if from_component == to_component || !added_edges.insert((from_component, to_component)) {
            continue;
        }
        component_graph.add_edge(
            component_nodes[from_component],
            component_nodes[to_component],
            (),
        );
    }

    (graph, component_graph, cyclic_components)
}

fn component_is_cyclic(graph: &MermaidLabelGraph, component: &[NodeIndex]) -> bool {
    if component.len() > 1 {
        return true;
    }

    let node_index = component[0];
    graph
        .edges_directed(node_index, petgraph::Direction::Outgoing)
        .any(|edge| edge.target() == node_index)
}

#[cfg(test)]
#[path = "../../../tests/unit/flowhub/mermaid/topology.rs"]
mod tests;
