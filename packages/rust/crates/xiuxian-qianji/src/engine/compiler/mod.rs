//! Compiler feature folder.
//!
//! Start in `pipeline`; dispatch flows through `mechanism_dispatch`.

mod annotation;
mod calibration;
mod formal_audit;
mod graph_assembly;
mod io_mechanisms;
#[cfg(feature = "llm")]
mod llm_client;
#[cfg(feature = "llm")]
mod llm_node;
mod manifest_parser;
mod mechanism_dispatch;
#[path = "../../engine_compiler_pipeline.rs"]
mod pipeline;
mod router;
mod security_scan;
mod stateful_mechanisms;
#[path = "../../engine_compiler_task_mechanisms.rs"]
mod task_mechanisms;
mod task_type;
mod topology_validation;
mod wendao_ingester;
mod wendao_refresh;
#[path = "../../engine_compiler_wendao_sql.rs"]
mod wendao_sql;

pub(in crate::engine) use pipeline::compile_manifest;
