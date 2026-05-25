//! Flowhub BPMN service-task adapter for the durable `ActivityTask` protocol.

mod completion;
mod schedule;
mod types;

pub use xiuxian_qianji_runtime::{
    FLOWHUB_SERVICE_ACTIVITY_TYPE, FLOWHUB_SERVICE_COMPLETION_METADATA_KEY, FlowhubScenarioIdRef,
    FlowhubServiceActivityScheduleInput, QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeInstantMs,
    build_flowhub_service_activity_schedule_record,
    build_flowhub_service_task_contract_activity_result,
    build_flowhub_service_task_contract_completion_data,
};

pub use completion::{
    build_flowhub_service_task_complete_http_request, build_flowhub_service_task_completion_payload,
};
pub use schedule::build_flowhub_service_activity_schedule_record_from_http_pending_work;
pub use types::FlowhubServiceActivityHttpScheduleInput;
