//! Episteme bootstrap pipeline command boundary.

#[cfg(feature = "episteme-foyer-artifact-cache")]
mod artifact;
mod command;

#[cfg(all(test, feature = "episteme-foyer-artifact-cache"))]
pub(super) use artifact::episteme_bootstrap_artifact_cache_options;
pub(super) use command::run_episteme_bootstrap_pipeline_command;
