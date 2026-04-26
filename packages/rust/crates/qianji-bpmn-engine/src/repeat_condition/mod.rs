//! Shared bounded repeat-condition parsing and evaluation helpers.

mod api;
mod common;
mod gateway;
mod multi_instance;

pub(crate) use api::{
    GatewayConditionError, MultiInstanceCompletionConditionError, MultiInstanceCompletionCounts,
    evaluate_gateway_condition, evaluate_multi_instance_completion_condition,
    is_supported_gateway_condition, is_supported_multi_instance_completion_condition,
};
pub use api::{GatewayConditionSummary, parse_gateway_condition_summary};
