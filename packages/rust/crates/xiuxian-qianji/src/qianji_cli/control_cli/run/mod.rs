//! Routes parsed `qianji control` commands to focused executor leaves.

mod dispatch;
mod error;
mod inventory;
mod lookup;
mod recovery;
mod state;

pub(super) use dispatch::run_control_command_impl;
