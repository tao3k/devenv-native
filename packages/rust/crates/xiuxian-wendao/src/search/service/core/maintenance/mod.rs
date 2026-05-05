mod helpers;
mod local;
mod queue;
#[path = "repo/mod.rs"]
mod repo;
mod worker;

#[cfg(test)]
#[path = "../../../../../tests/unit/search/service/core/maintenance/mod.rs"]
mod tests;

#[cfg(test)]
pub(crate) use helpers::REPO_MAINTENANCE_SHUTDOWN_MESSAGE;
