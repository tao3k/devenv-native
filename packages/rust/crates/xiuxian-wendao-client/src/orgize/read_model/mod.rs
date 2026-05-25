//! `DuckDB`-backed read-model materialization for Orgize agent tasks.

mod archive;
mod filter;
mod json;
mod memory;
mod model;
#[cfg(feature = "performance")]
pub mod perf_support;
mod render;
mod row_view;
mod run;
mod section_lens;
mod settings;
mod store;

pub(crate) use run::{
    run_ogrid_show, run_read_model, run_task_archive, run_task_list, run_task_probe,
    run_task_recover, run_task_report, run_task_sdd,
};
