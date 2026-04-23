//! Start in `api`; `defaults` owns unsupported-handler fallbacks and
//! `bridge_impl` owns the `BpmnHostBridge` trait implementation.

#[path = "bpmn_adapter_bridge_api.rs"]
pub(crate) mod api;
#[path = "bpmn_adapter_bridge_impl.rs"]
mod bridge_impl;
#[path = "bpmn_adapter_bridge_defaults.rs"]
mod defaults;
