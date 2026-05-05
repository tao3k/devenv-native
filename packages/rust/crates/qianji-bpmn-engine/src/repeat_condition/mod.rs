//! Shared bounded repeat-condition parsing and evaluation helpers.

mod api;
mod gateway;
mod multi_instance;
mod operand;

pub(crate) use api::{
    GatewayConditionError, GatewayConditionSummary, MultiInstanceCompletionConditionError,
    MultiInstanceCompletionCounts, evaluate_gateway_condition,
    evaluate_multi_instance_completion_condition, is_supported_gateway_condition,
    is_supported_multi_instance_completion_condition, parse_gateway_condition_summary,
};
