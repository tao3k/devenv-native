//! `search::repo_entity::build` owns Wendao search repo entity build behavior.

mod orchestration;
mod plan;
mod types;
mod write;

#[cfg(test)]
#[path = "../../../../tests/unit/search/repo_entity/build/mod.rs"]
mod tests;

pub(crate) use orchestration::publish_repo_entities;
pub(crate) use plan::plan_repo_entity_build;
#[cfg(test)]
pub(crate) use plan::repo_entity_file_fingerprints;
pub(crate) use types::{RepoEntityBuildAction, RepoEntityBuildPlan};
