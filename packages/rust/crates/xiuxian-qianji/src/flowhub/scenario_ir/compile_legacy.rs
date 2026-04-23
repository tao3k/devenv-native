use crate::contracts::{FlowhubGraphContract, FlowhubGraphNodeContract};

use super::scenario_ir_model::{
    FlowhubScenarioIr, FlowhubScenarioNodeIr, FlowhubScenarioWorkdirIr,
};

pub(super) fn compile_legacy_scenario_ir(
    resolved_graph_name: &str,
    graph: &FlowhubGraphContract,
) -> FlowhubScenarioIr {
    FlowhubScenarioIr {
        merimind_graph_name: resolved_graph_name.to_string(),
        scenario_id: None,
        description: None,
        declared_topology: Some(graph.topology),
        workdir: graph
            .workdir
            .as_ref()
            .map(|workdir| FlowhubScenarioWorkdirIr {
                note: workdir.note.clone(),
                root: workdir.root.clone(),
                check: workdir.check.clone(),
                target: workdir.target.clone(),
                done_gate_require: workdir
                    .target
                    .as_ref()
                    .map(|target| target.paths.clone())
                    .unwrap_or_default(),
            }),
        nodes: graph.node.iter().map(compose_legacy_node_ir).collect(),
    }
}

fn compose_legacy_node_ir(node: &FlowhubGraphNodeContract) -> FlowhubScenarioNodeIr {
    FlowhubScenarioNodeIr {
        label: node.label.clone(),
        kind: Some(node.kind.clone()),
        role: Some(node.role.clone()),
        agent_action: Some(node.agent_action.clone()),
        checkpoint: None,
        writes: Vec::new(),
        merge_target: Vec::new(),
    }
}
