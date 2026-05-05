//! DMN parse driver branch wiring.

mod core;
mod event;
mod state;

pub(crate) use core::{parse_dmn_decision_impl, parse_dmn_decisions_impl};
