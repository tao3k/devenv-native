//! Generic BPMN host-work `ActivityTask` adapters.
//!
//! Flowhub service tasks keep their scenario-specific adapter. This module is
//! the generic server-side evidence bridge for native BPMN host work completed
//! through qianji-server HTTP routes.

mod completion;

pub(crate) use completion::bpmn_host_work_completion_from_payload;
pub use completion::build_bpmn_host_work_activity_result;
pub use xiuxian_qianji_runtime::{
    BPMN_HOST_WORK_ACTIVITY_METADATA_KEY, BPMN_HOST_WORK_ACTIVITY_SCHEMA,
    BPMN_HOST_WORK_ACTIVITY_TYPE, BPMN_HOST_WORK_COMPLETION_METADATA_KEY,
    BPMN_HOST_WORK_COMPLETION_SCHEMA, BpmnHostWorkActivityScheduleInput,
    QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeInstantMs,
    build_bpmn_host_work_activity_schedule_record,
};
