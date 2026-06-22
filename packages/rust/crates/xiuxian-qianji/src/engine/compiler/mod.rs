//! Compiler feature folder.
//!
//! Start in `pipeline`; dispatch flows through `mechanism_dispatch`.

mod annotation;
mod calibration;
mod formal_audit;
mod graph_assembly;
mod io_mechanisms;
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
#[cfg(feature = "wendao-integration")]
mod wendao_ingester;
#[cfg(feature = "wendao-integration")]
mod wendao_refresh;
#[cfg(feature = "wendao-integration")]
#[path = "../../engine_compiler_wendao_sql.rs"]
mod wendao_sql;

pub(in crate::engine) use pipeline::compile_manifest;
pub(in crate::engine::compiler) use task_type::TaskType;
