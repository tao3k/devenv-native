//! `analyzers::config` owns Wendao analyzers config behavior.

mod load;
mod parse;
#[cfg(test)]
#[path = "../../../tests/unit/analyzers/config/mod.rs"]
mod tests;
mod toml;
mod types;

pub use load::load_repo_intelligence_config;
pub use types::{
    RegisteredRepository, RepoIntelligenceConfig, RepositoryPluginConfig, RepositoryRef,
    RepositoryRefreshPolicy,
};
