//! Repo-native semantic SSOT command surface.

mod command;
mod run;

pub use command::{SemanticCommand, SemanticRefreshProjectionsArgs};
pub(crate) use run::run_command;
