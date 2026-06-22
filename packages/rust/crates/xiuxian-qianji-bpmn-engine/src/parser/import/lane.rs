use super::model::{RawLaneMembershipSpec, RawPackageDocument, RawProcessScope};
use crate::bpmn_model_api::{
    BpmnDocumentSnapshot, BpmnLaneSetSnapshot, BpmnLaneSnapshot, BpmnProcessSnapshot,
};

pub(crate) fn attach_lane_memberships(
    raw: &mut RawPackageDocument,
    snapshot: &BpmnDocumentSnapshot,
) {
    for snapshot_process in &snapshot.processes {
        let Some(process_id) = snapshot_process.process_id.as_deref() else {
            continue;
        };
        let Some(raw_process) = raw.processes.iter_mut().find(|process| {
            process.process_id == process_id && process.scope == RawProcessScope::TopLevel
        }) else {
            continue;
        };
        attach_process_lane_memberships(raw_process, snapshot_process);
    }
}

fn attach_process_lane_memberships(
    raw_process: &mut super::model::RawProcess,
    snapshot_process: &BpmnProcessSnapshot,
) {
    for lane_set in &snapshot_process.lane_sets {
        for lane in &lane_set.lanes {
            attach_lane_membership(raw_process, lane_set, lane);
        }
    }
}

fn attach_lane_membership(
    raw_process: &mut super::model::RawProcess,
    lane_set: &BpmnLaneSetSnapshot,
    lane: &BpmnLaneSnapshot,
) {
    for flow_node_ref in &lane.flow_node_refs {
        let Some(node) = raw_process
            .nodes
            .iter_mut()
            .find(|node| node.bpmn_id == *flow_node_ref)
        else {
            continue;
        };
        if node.lane.is_some() {
            continue;
        }
        node.lane = Some(RawLaneMembershipSpec {
            set_id: lane_set.lane_set_id.clone(),
            set_name: lane_set.name.clone(),
            id: lane.lane_id.clone(),
            name: lane.name.clone(),
        });
    }
}
