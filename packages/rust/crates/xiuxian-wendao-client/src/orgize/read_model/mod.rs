//! `DuckDB`-backed read-model materialization for Orgize agent tasks.

mod archive;
mod filter;
mod json;
mod model;
#[cfg(feature = "performance")]
pub mod perf_support;
mod render;
mod row_view;
mod run;
mod settings;
mod store;

pub(crate) use run::{run_read_model, run_task_archive, run_task_list, run_task_report};
