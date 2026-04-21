//! Table-of-contents seam for bounded DMN parse driver and XML leaves.

#[path = "parse/driver.rs"]
mod driver;
#[path = "../dmn_parse_state.rs"]
mod state;
#[path = "../dmn_parse_unary.rs"]
mod unary;
#[path = "parse/xml/mod.rs"]
mod xml;

pub(crate) use driver::{parse_dmn_decision_impl, parse_dmn_decisions_impl};
