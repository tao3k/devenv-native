//! `hmas` owns Wendao hmas behavior.

#[path = "blackboard/mod.rs"]
mod blackboard;
mod protocol;

pub use blackboard::{
    HmasValidationIssue, HmasValidationReport, validate_blackboard_file,
    validate_blackboard_markdown,
};
pub use protocol::{
    HmasConclusionPayload, HmasDigitalThreadPayload, HmasEvidencePayload, HmasRecordKind,
    HmasSourceNode, HmasTaskPayload,
};
