//! `parsers::languages::rust::cargo::dependencies` owns Wendao rust cargo dependencies behavior.

mod api;
mod regex;
mod types;

pub use self::api::parse_cargo_dependencies;
pub use self::types::CargoDependency;

#[cfg(test)]
#[path = "../../../../../../tests/unit/parsers/languages/rust/cargo/dependencies.rs"]
mod tests;
