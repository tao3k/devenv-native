//! `analyzers::projection::builder` owns Wendao analyzers projection builder behavior.

mod anchors;
mod assemble;
mod helpers;
mod kinds;
mod sources;

pub use assemble::build_projection_inputs;

#[cfg(test)]
#[path = "../../../../tests/unit/analyzers/projection/builder/mod.rs"]
mod tests;
