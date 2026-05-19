//! Orgize-backed client command execution.

mod basic;
mod dispatch;
mod paths;
mod planning;
mod sdd;
mod sparse_tree;

pub(crate) use dispatch::run_command;
