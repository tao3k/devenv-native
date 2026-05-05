//! Repo-native semantic SSOT command surface.

mod command;
mod run;

pub use command::{
    SemanticCommand, SemanticDescribeReadModelArgs, SemanticReadModelQueryArgs,
    SemanticRefreshProjectionsArgs, SemanticSnapshotReadModelArgs,
};
pub(crate) use run::run_command;
