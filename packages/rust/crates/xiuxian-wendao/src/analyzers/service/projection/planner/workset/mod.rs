//! `analyzers::service::projection::planner::workset` owns Wendao projection planner workset behavior.

mod balance;
mod groups;
mod math;
mod orchestration;
mod strategy;

pub use orchestration::{
    build_docs_planner_workset, docs_planner_workset_from_config,
    docs_planner_workset_from_config_with_registry,
};

#[cfg(test)]
pub(crate) use math::quota_band;
#[cfg(test)]
pub(crate) use strategy::build_docs_planner_workset_strategy;
