#[cfg(all(feature = "duckdb", feature = "valkey"))]
mod activity_evidence;
#[cfg(all(feature = "duckdb", feature = "valkey"))]
mod bpmn_source_admission;
mod config;
#[cfg(feature = "valkey")]
mod flowhub;
mod health;
mod internal_auth;
#[cfg(feature = "qianji-full")]
mod llm_worker;
mod parse;
mod support;
