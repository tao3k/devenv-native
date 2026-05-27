use crate::telemetry::unix_millis_now;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::HostBridgeError;

use super::api::{
    BusinessRuleHandler, ClockHandler, EventPollHandler, ManualHandler, ScriptHandler, SendHandler,
    ServiceHandler, TaskHandler, UserHandler,
};

pub(super) fn default_clock_handler() -> ClockHandler {
    Arc::new(unix_millis_now)
}

pub(super) fn unsupported_send_handler(operation: &'static str) -> SendHandler {
    Arc::new(move |_request| Box::pin(async move { Err(unsupported(operation)) }))
}

pub(super) fn unsupported_task_handler(operation: &'static str) -> TaskHandler {
    Arc::new(move |_request| Box::pin(async move { Err(unsupported(operation)) }))
}

pub(super) fn unsupported_service_handler(operation: &'static str) -> ServiceHandler {
    Arc::new(move |_request| Box::pin(async move { Err(unsupported(operation)) }))
}

pub(super) fn unsupported_script_handler(operation: &'static str) -> ScriptHandler {
    Arc::new(move |_request| Box::pin(async move { Err(unsupported(operation)) }))
}

pub(super) fn unsupported_user_handler(operation: &'static str) -> UserHandler {
    Arc::new(move |_request| Box::pin(async move { Err(unsupported(operation)) }))
}

pub(super) fn unsupported_manual_handler(operation: &'static str) -> ManualHandler {
    Arc::new(move |_request| Box::pin(async move { Err(unsupported(operation)) }))
}

pub(super) fn unsupported_business_rule_handler(operation: &'static str) -> BusinessRuleHandler {
    Arc::new(move |_request| Box::pin(async move { Err(unsupported(operation)) }))
}

pub(super) fn unsupported_event_poll_handler(operation: &'static str) -> EventPollHandler {
    Arc::new(move |_request| Box::pin(async move { Err(unsupported(operation)) }))
}

fn unsupported(operation: &'static str) -> HostBridgeError {
    HostBridgeError::UnsupportedOperation { operation }
}
