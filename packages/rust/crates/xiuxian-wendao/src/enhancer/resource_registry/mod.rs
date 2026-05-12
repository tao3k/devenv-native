//! `enhancer::resource_registry` owns Wendao enhancer resource registry behavior.

mod registry;
mod scan;
mod semantic;
mod types;

pub use types::{WendaoResourceLinkTarget, WendaoResourceRegistry};

#[cfg(test)]
#[path = "../../../tests/unit/enhancer/resource_registry/mod.rs"]
mod tests;
