//! Public BPMN IR `api` facade.

mod api;
#[path = "ir_node_api/human_task/api.rs"]
mod human_task;
#[path = "ir_node_api/kind/api.rs"]
mod kind;
#[path = "ir_node_api/lane/api.rs"]
mod lane;
#[path = "ir_node_api/node/api.rs"]
mod node;
#[path = "ir_node_api/script/api.rs"]
mod script;
#[path = "ir_node_api/task_io/api.rs"]
mod task_io;

pub use api::{
    BpmnGatewayKind, BpmnHumanTaskAssignmentSpec, BpmnHumanTaskChoiceSpec, BpmnHumanTaskFormSpec,
    BpmnHumanTaskFreeTextSpec, BpmnHumanTaskResourceRoleSpec, BpmnLaneMembershipSpec, BpmnNodeKind,
    BpmnNodeSpec, BpmnScriptTaskSpec, BpmnSubProcessKind, BpmnTaskInputBinding,
    BpmnTaskInputSource, BpmnTaskIoSpec, BpmnTaskOutputBinding,
};
