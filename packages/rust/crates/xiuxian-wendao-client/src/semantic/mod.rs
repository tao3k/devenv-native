//! Repo-native semantic SSOT command surface.

mod command;
mod run;

pub use command::{
    SemanticCheckReadModelSnapshotArgs, SemanticCommand, SemanticDescribeReadModelArgs,
    SemanticPlanReadModelMaterializationArgs, SemanticReadModelQueryArgs,
    SemanticRefreshProjectionsArgs, SemanticSnapshotReadModelArgs,
};
pub(crate) use run::run_command;
