#[path = "../../integration/support/valkey.rs"]
mod valkey_support;

mod adapter;
mod control;
#[cfg(feature = "duckdb")]
mod data_store;
mod http;
mod runtime;
mod runtime_identity;
mod runtime_lease;
mod runtime_selector;
