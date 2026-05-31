//! Crate-owned deterministic ontology bootstrap pipeline.

mod engine;
mod types;

pub use engine::run_episteme_ontology_bootstrap_pipeline;
#[cfg(feature = "foyer-artifact-cache")]
pub use engine::{
    admit_episteme_ontology_bootstrap_artifact_cache_options,
    read_through_episteme_ontology_bootstrap_artifacts,
    restore_episteme_ontology_bootstrap_pipeline_artifacts,
    run_episteme_ontology_bootstrap_pipeline_with_artifact_cache,
};

#[cfg(feature = "foyer-artifact-cache")]
pub use types::{
    EpistemeOntologyBootstrapArtifactCacheOptions,
    EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome,
    EpistemeOntologyBootstrapArtifactCacheReadThroughReport,
    EpistemeOntologyBootstrapArtifactCacheReport,
    EpistemeOntologyBootstrapArtifactCacheRestoreMiss,
    EpistemeOntologyBootstrapArtifactCacheRestoreReport,
    EpistemeOntologyBootstrapArtifactCacheStage,
};
pub use types::{
    EpistemeOntologyBootstrapPipelineReport, EpistemeOntologyBootstrapPipelineRequest,
    EpistemeOntologyBootstrapPipelineSafetyFlags,
};
