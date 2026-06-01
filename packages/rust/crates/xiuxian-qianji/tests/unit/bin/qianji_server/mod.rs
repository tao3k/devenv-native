#[cfg(feature = "duckdb")]
mod activity_evidence;
#[cfg(feature = "duckdb")]
mod bpmn_source_admission;
mod config;
mod flowhub;
mod health;
mod internal_auth;
#[cfg(feature = "llm")]
mod llm_worker;
mod parse;
mod support;
