//! Immutable BPMN package and process specification types.

mod edge;
mod event;
mod index;
mod node;
mod process;
mod repeat;

pub use edge::BpmnEdgeSpec;
pub use event::{BpmnEventKind, BpmnEventSpec, BpmnTimerKind, BpmnTimerSpec};
pub use index::{BpmnIndexRange, BpmnNodeIndex};
pub use node::{BpmnGatewayKind, BpmnNodeKind, BpmnNodeSpec};
pub use process::{BpmnPackage, BpmnProcessSpec, ProcessKey};
pub use repeat::{BpmnRepeatSpec, BpmnSequentialMultiInstanceSpec, BpmnStandardLoopSpec};
