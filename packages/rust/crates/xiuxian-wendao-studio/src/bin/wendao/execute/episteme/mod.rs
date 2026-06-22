//! Episteme command execution handlers.

mod bootstrap;
mod cache;
mod external;
mod handler;
mod root;

pub(super) use handler::handle;

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/execute/episteme/mod.rs"]
mod tests;
