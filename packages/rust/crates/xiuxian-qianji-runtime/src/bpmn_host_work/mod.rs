//! Generic BPMN host-work runtime adapters.
//!
//! This module converts engine-owned pending host-work facts into durable
//! Qianji control-plane activity schedule/result records without depending on
//! qianji-server or HTTP payload types.

mod completion;
mod evidence;
mod identity;
mod schedule;

pub use completion::{
    BPMN_HOST_WORK_COMPLETION_METADATA_KEY, BPMN_HOST_WORK_COMPLETION_SCHEMA,
    BpmnHostWorkCompletion, BpmnHostWorkCompletionKind, build_bpmn_host_work_activity_result,
};
pub use evidence::{
    BPMN_HOST_WORK_EVIDENCE_RUN_SCHEMA, BPMN_HOST_WORK_FAILURE_METADATA_KEY,
    BPMN_HOST_WORK_FAILURE_SCHEMA, BpmnHostWorkActivityEvidenceInput,
    BpmnHostWorkCompletionActivityEvidenceInput, BpmnHostWorkFailure,
    BpmnHostWorkFailureActivityEvidenceInput, ensure_bpmn_host_work_activity_evidence_run,
    record_bpmn_host_work_completion_activity_evidence,
    record_bpmn_host_work_failure_activity_evidence,
};
pub use identity::{
    BpmnHostWorkIdentity, find_matching_bpmn_host_work, pending_bpmn_host_work_matches_identity,
};
pub use schedule::{
    BPMN_HOST_WORK_ACTIVITY_METADATA_KEY, BPMN_HOST_WORK_ACTIVITY_SCHEMA,
    BPMN_HOST_WORK_ACTIVITY_TYPE, BpmnHostWorkActivityScheduleInput,
    build_bpmn_host_work_activity_schedule_record,
};
