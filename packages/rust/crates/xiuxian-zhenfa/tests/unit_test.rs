//! Cargo entry point for xiuxian-zhenfa unit tests.

#[path = "unit/dependency_boundary.rs"]
mod dependency_boundary;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[cfg(feature = "gateway")]
#[path = "unit/notification.rs"]
mod notification;
#[path = "unit/signal_registry.rs"]
mod signal_registry;
