//! Cargo entry point for xiuxian-zhenfa integration tests.

#[path = "integration/client.rs"]
mod client;
#[path = "integration/context_extensions.rs"]
mod context_extensions;
#[path = "integration/contract_validation.rs"]
mod contract_validation;
#[path = "integration/contracts.rs"]
mod contracts;
#[path = "integration/error_mapping.rs"]
mod error_mapping;
#[path = "integration/native_registry.rs"]
mod native_registry;
#[path = "integration/transmuter.rs"]
mod transmuter;
#[path = "integration/xml_lite.rs"]
mod xml_lite;
#[path = "integration/zhenfa_tool_macro.rs"]
mod zhenfa_tool_macro;
