//! BPMN host-work adapter for Qianji-governed LLM activity scheduling.

mod route;
mod types;

pub use route::build_bpmn_host_work_llm_activity_route;
pub use types::{
    BPMN_HOST_WORK_LLM_ACTIVITY_ROUTE_SCHEMA, BpmnHostWorkLlmActivityRouteInput,
    BpmnHostWorkLlmEndpointDecision, BpmnHostWorkLlmRouteDecision,
};
