//! Directory-oriented CLI feature folder.
//!
//! Start with `api`; it is the single visible entry seam for this folder.

mod api;
mod output;
mod parse;
mod run;
mod types;

#[cfg(test)]
pub(crate) use api::{DirCliCommand, ShowCliTarget, run_dir_command};
pub(crate) use api::{handle_dir_command, parse_dir_command};
