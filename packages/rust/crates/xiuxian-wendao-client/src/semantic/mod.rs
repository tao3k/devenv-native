//! Repo-native semantic SSOT command surface.

mod command;
mod run;

pub use command::{
    SemanticCheckReadModelSnapshotArgs, SemanticCommand, SemanticDescribeReadModelArgs,
    SemanticPlanReadModelMaterializationArgs, SemanticPreflightReadModelMaterializationArgs,
    SemanticReadModelQueryArgs, SemanticRefreshProjectionsArgs, SemanticSnapshotReadModelArgs,
};
pub(crate) use run::run_command;
