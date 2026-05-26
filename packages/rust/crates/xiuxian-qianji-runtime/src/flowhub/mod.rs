//! Flowhub runtime adapters.

mod service_activity;
mod service_completion;
mod worker;

pub use service_activity::{
    FLOWHUB_SERVICE_ACTIVITY_METADATA_KEY, FLOWHUB_SERVICE_ACTIVITY_SCHEMA,
    FLOWHUB_SERVICE_ACTIVITY_TYPE, FlowhubScenarioIdRef, FlowhubServiceActivityScheduleInput,
    QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeInstantMs,
    build_flowhub_service_activity_schedule_record,
};
pub use service_completion::{
    FLOWHUB_SERVICE_COMPLETION_METADATA_KEY, FLOWHUB_SERVICE_COMPLETION_SCHEMA,
    FlowhubServiceTaskCompletion, QianjiRuntimeBpmnActivityId, QianjiRuntimeBpmnProcessId,
    QianjiRuntimeBpmnTokenId, build_flowhub_service_task_completion,
    build_flowhub_service_task_contract_activity_result,
    build_flowhub_service_task_contract_completion_data, flowhub_service_task_bpmn_source_path,
};
pub use worker::{
    FLOWHUB_SERVICE_WORKER_RUN_SCHEMA, FlowhubServiceWorkerLoopOutput,
    FlowhubServiceWorkerLoopRequest, FlowhubServiceWorkerStepOutput,
    run_flowhub_service_worker_completion_loop,
};
