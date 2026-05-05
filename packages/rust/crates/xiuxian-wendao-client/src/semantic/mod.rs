//! Repo-native semantic SSOT command surface.

mod command;
mod run;

pub use command::{SemanticCommand, SemanticReadModelQueryArgs, SemanticRefreshProjectionsArgs};
pub(crate) use run::run_command;
