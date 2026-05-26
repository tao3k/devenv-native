//! JSON trace projection for the `SearchStrategyFlow` Flight service path.

mod budget;
mod decode;
mod policy;
mod projection;
mod route;
mod timing;
mod validation;

pub(crate) use projection::{
    SearchStrategyFlowServiceTraceRequest, search_strategy_flow_service_trace_json,
};
pub(crate) use timing::{
    SearchStrategyFlowTimingMeasurements, search_strategy_flow_trace_with_materialization_timing,
};

#[cfg(test)]
pub(crate) use policy::search_strategy_flow_performance_policy_json;
#[cfg(test)]
pub(crate) use route::frontier_route_bucket;
#[cfg(test)]
pub(crate) use timing::search_strategy_flow_timing_breakdown_json;
