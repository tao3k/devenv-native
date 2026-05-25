//! Crate-owned deterministic ontology bootstrap pipeline.

mod types;

use std::path::Path;
#[cfg(feature = "artifact-cache")]
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::load_episteme_runtime_config;
#[cfg(feature = "artifact-cache")]
use xiuxian_db_store::artifact_cache::ArtifactBlobCache;

#[cfg(feature = "artifact-cache")]
use super::{
    EpistemeOntologyArtifactBundleIdentity, EpistemeOntologyArtifactBundleKind,
    restore_episteme_ontology_artifact_bundle, write_episteme_ontology_artifact_bundle,
};
use super::{
    EpistemeOntologyStructuralFactsConfiguredRequest,
    EpistemeOntologyStructuralFactsReasoningFillPlanRequest,
    EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest,
    EpistemeOntologyStructuralFactsReasoningPacketRequest,
    write_episteme_ontology_structural_facts_from_config,
    write_episteme_ontology_structural_facts_reasoning_fill_plan,
    write_episteme_ontology_structural_facts_reasoning_ledger_seed,
    write_episteme_ontology_structural_facts_reasoning_packet,
};

#[cfg(feature = "artifact-cache")]
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

const BOOTSTRAP_PIPELINE_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_bootstrap_pipeline_report.v1";

#[cfg(feature = "artifact-cache")]
const BOOTSTRAP_ARTIFACT_CACHE_ADMISSION_RUN_DIGEST: &str = "bootstrap-digest-admission";

/// Run the deterministic Episteme ontology bootstrap pipeline.
///
/// # Errors
///
/// Returns an error when any stage fails to resolve runtime defaults, validate
/// source-contract facts, or write its run artifacts.
pub fn run_episteme_ontology_bootstrap_pipeline(
    request: &EpistemeOntologyBootstrapPipelineRequest,
) -> Result<EpistemeOntologyBootstrapPipelineReport> {
    let config = load_episteme_runtime_config(request.episteme_root())
        .context("failed to load Episteme runtime config")?;
    let ontology_generation_run_root = request
        .ontology_generation_run_root()
        .map(Path::to_path_buf)
        .or_else(|| {
            config
                .as_ref()
                .and_then(|config| config.ontology_generation_runs.clone())
        })
        .unwrap_or_else(|| request.episteme_root().join("runs/ontology-generation"));

    let mut structural_request = EpistemeOntologyStructuralFactsConfiguredRequest::new(
        request.episteme_root(),
        request.structural_facts_run_id().to_owned(),
    )
    .with_validation_mode(request.validation_mode());
    if let Some(corpus_root) = request.corpus_root() {
        structural_request = structural_request.with_corpus_root(corpus_root.to_path_buf());
    }
    if let Some(structure_run_root) = request.structure_run_root() {
        structural_request = structural_request.with_run_root(structure_run_root.to_path_buf());
    }
    let structural_facts =
        write_episteme_ontology_structural_facts_from_config(&structural_request)?;

    let mut packet_request = EpistemeOntologyStructuralFactsReasoningPacketRequest::new(
        structural_facts.structural_facts_json.clone(),
        request.reasoning_packet_run_id().to_owned(),
    )
    .with_limit(request.reasoning_packet_limit());
    if let Some(category) = request.category() {
        packet_request = packet_request.with_category(category.to_owned());
    }
    if let Some(route) = request.route() {
        packet_request = packet_request.with_route(route.to_owned());
    }
    let reasoning_packet = write_episteme_ontology_structural_facts_reasoning_packet(
        &packet_request,
        &ontology_generation_run_root,
    )?;

    let ledger_seed_request = EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest::new(
        reasoning_packet.reasoning_packet_json.clone(),
        request.reasoning_ledger_seed_run_id().to_owned(),
    )
    .with_limit(request.reasoning_ledger_seed_limit());
    let reasoning_ledger_seed = write_episteme_ontology_structural_facts_reasoning_ledger_seed(
        &ledger_seed_request,
        &ontology_generation_run_root,
    )?;

    let fill_plan_request = EpistemeOntologyStructuralFactsReasoningFillPlanRequest::new(
        reasoning_ledger_seed.reasoning_ledger_seed_json.clone(),
        request.reasoning_fill_plan_run_id().to_owned(),
    )
    .with_limit(request.reasoning_fill_plan_limit());
    let reasoning_fill_plan = write_episteme_ontology_structural_facts_reasoning_fill_plan(
        &fill_plan_request,
        &ontology_generation_run_root,
    )?;

    Ok(EpistemeOntologyBootstrapPipelineReport {
        schema_version: BOOTSTRAP_PIPELINE_REPORT_SCHEMA_VERSION,
        run_id: request.run_id().to_owned(),
        episteme_root: request.episteme_root().to_path_buf(),
        ontology_generation_run_root,
        structural_facts,
        reasoning_packet,
        reasoning_ledger_seed,
        reasoning_fill_plan,
        safety: EpistemeOntologyBootstrapPipelineSafetyFlags::deterministic_non_mutating(),
    })
}

/// Run the deterministic bootstrap pipeline and cache generated run directories.
///
/// # Errors
///
/// Returns an error when the deterministic pipeline fails, a cache identity is
/// invalid, a generated run directory cannot be packed, or the cache backend
/// cannot persist the bundle bytes.
#[cfg(feature = "artifact-cache")]
pub fn run_episteme_ontology_bootstrap_pipeline_with_artifact_cache(
    request: &EpistemeOntologyBootstrapPipelineRequest,
    cache: &(dyn ArtifactBlobCache + Send + Sync),
    options: &EpistemeOntologyBootstrapArtifactCacheOptions,
) -> Result<EpistemeOntologyBootstrapArtifactCacheReport> {
    validate_bootstrap_artifact_cache_options(options)?;
    let pipeline = run_episteme_ontology_bootstrap_pipeline(request)?;
    let bundles = vec![
        write_bootstrap_stage_bundle(
            cache,
            options,
            "structural-facts",
            request.structural_facts_run_id(),
            pipeline.structural_facts.run_dir.as_path(),
        )?,
        write_bootstrap_stage_bundle(
            cache,
            options,
            "reasoning-packet",
            request.reasoning_packet_run_id(),
            pipeline.reasoning_packet.run_dir.as_path(),
        )?,
        write_bootstrap_stage_bundle(
            cache,
            options,
            "reasoning-ledger-seed",
            request.reasoning_ledger_seed_run_id(),
            pipeline.reasoning_ledger_seed.run_dir.as_path(),
        )?,
        write_bootstrap_stage_bundle(
            cache,
            options,
            "reasoning-fill-plan",
            request.reasoning_fill_plan_run_id(),
            pipeline.reasoning_fill_plan.run_dir.as_path(),
        )?,
    ];
    Ok(EpistemeOntologyBootstrapArtifactCacheReport { pipeline, bundles })
}

/// Restore deterministic bootstrap stage directories from the artifact cache.
///
/// # Errors
///
/// Returns an error when runtime roots cannot be resolved, an artifact identity
/// is invalid, a cached bundle cannot be read, or restored bytes cannot be
/// unpacked into their deterministic target directory.
#[cfg(feature = "artifact-cache")]
pub fn restore_episteme_ontology_bootstrap_pipeline_artifacts(
    request: &EpistemeOntologyBootstrapPipelineRequest,
    cache: &(dyn ArtifactBlobCache + Send + Sync),
    options: &EpistemeOntologyBootstrapArtifactCacheOptions,
) -> Result<EpistemeOntologyBootstrapArtifactCacheRestoreReport> {
    validate_bootstrap_artifact_cache_options(options)?;
    let mut restored = Vec::new();
    let mut missing = Vec::new();
    for target in bootstrap_stage_targets(request)? {
        let identity = target.identity(options);
        if let Some(report) =
            restore_episteme_ontology_artifact_bundle(cache, &identity, target.run_dir.as_path())?
        {
            restored.push(report);
        } else {
            missing.push(EpistemeOntologyBootstrapArtifactCacheRestoreMiss {
                stage: target.stage,
                run_digest: identity.run_digest,
                target_dir: target.run_dir,
            });
        }
    }
    Ok(EpistemeOntologyBootstrapArtifactCacheRestoreReport { restored, missing })
}

/// Restore bootstrap artifacts when present, otherwise generate and cache them.
///
/// # Errors
///
/// Returns an error when restore fails for a non-miss reason, or when the
/// deterministic bootstrap pipeline/cache write fails after a cache miss.
#[cfg(feature = "artifact-cache")]
pub fn read_through_episteme_ontology_bootstrap_artifacts(
    request: &EpistemeOntologyBootstrapPipelineRequest,
    cache: &(dyn ArtifactBlobCache + Send + Sync),
    options: &EpistemeOntologyBootstrapArtifactCacheOptions,
) -> Result<EpistemeOntologyBootstrapArtifactCacheReadThroughReport> {
    let restore = restore_episteme_ontology_bootstrap_pipeline_artifacts(request, cache, options)?;
    if restore.complete() {
        return Ok(EpistemeOntologyBootstrapArtifactCacheReadThroughReport {
            outcome: EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome::Restored,
            restore,
            generated: None,
        });
    }
    let generated =
        run_episteme_ontology_bootstrap_pipeline_with_artifact_cache(request, cache, options)?;
    Ok(EpistemeOntologyBootstrapArtifactCacheReadThroughReport {
        outcome: EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome::Generated,
        restore,
        generated: Some(generated),
    })
}

/// Build artifact-cache options after validating digest components.
///
/// # Errors
///
/// Returns an error when the source or profile digest is not safe for
/// deterministic artifact storage.
#[cfg(feature = "artifact-cache")]
pub fn admit_episteme_ontology_bootstrap_artifact_cache_options(
    source_digest: impl Into<String>,
    profile_digest: impl Into<String>,
) -> Result<EpistemeOntologyBootstrapArtifactCacheOptions> {
    let options = EpistemeOntologyBootstrapArtifactCacheOptions::new(source_digest, profile_digest);
    validate_bootstrap_artifact_cache_options(&options)?;
    Ok(options)
}

#[cfg(feature = "artifact-cache")]
fn write_bootstrap_stage_bundle(
    cache: &(dyn ArtifactBlobCache + Send + Sync),
    options: &EpistemeOntologyBootstrapArtifactCacheOptions,
    stage: &str,
    run_id: &str,
    run_dir: &Path,
) -> Result<super::EpistemeOntologyArtifactBundleWriteReport> {
    let identity = EpistemeOntologyArtifactBundleIdentity {
        kind: EpistemeOntologyArtifactBundleKind::ReasoningProjection,
        source_digest: options.source_digest.clone(),
        profile_digest: options.profile_digest.clone(),
        run_digest: bootstrap_stage_run_digest(stage, run_id),
    };
    write_episteme_ontology_artifact_bundle(cache, &identity, run_dir)
}

#[cfg(feature = "artifact-cache")]
fn validate_bootstrap_artifact_cache_options(
    options: &EpistemeOntologyBootstrapArtifactCacheOptions,
) -> Result<()> {
    EpistemeOntologyArtifactBundleIdentity {
        kind: EpistemeOntologyArtifactBundleKind::ReasoningProjection,
        source_digest: options.source_digest.clone(),
        profile_digest: options.profile_digest.clone(),
        run_digest: BOOTSTRAP_ARTIFACT_CACHE_ADMISSION_RUN_DIGEST.to_owned(),
    }
    .artifact_key()
    .context("invalid Episteme bootstrap artifact-cache digest component")?;
    Ok(())
}

#[cfg(feature = "artifact-cache")]
fn bootstrap_stage_run_digest(stage: &str, run_id: &str) -> String {
    format!("{stage}-{run_id}")
}

#[cfg(feature = "artifact-cache")]
struct BootstrapStageTarget {
    stage: EpistemeOntologyBootstrapArtifactCacheStage,
    stage_label: &'static str,
    run_id: String,
    run_dir: PathBuf,
}

#[cfg(feature = "artifact-cache")]
impl BootstrapStageTarget {
    fn identity(
        &self,
        options: &EpistemeOntologyBootstrapArtifactCacheOptions,
    ) -> EpistemeOntologyArtifactBundleIdentity {
        EpistemeOntologyArtifactBundleIdentity {
            kind: EpistemeOntologyArtifactBundleKind::ReasoningProjection,
            source_digest: options.source_digest.clone(),
            profile_digest: options.profile_digest.clone(),
            run_digest: bootstrap_stage_run_digest(self.stage_label, self.run_id.as_str()),
        }
    }
}

#[cfg(feature = "artifact-cache")]
fn bootstrap_stage_targets(
    request: &EpistemeOntologyBootstrapPipelineRequest,
) -> Result<Vec<BootstrapStageTarget>> {
    let roots = resolve_bootstrap_artifact_roots(request)?;
    Ok(vec![
        BootstrapStageTarget {
            stage: EpistemeOntologyBootstrapArtifactCacheStage::StructuralFacts,
            stage_label: "structural-facts",
            run_id: request.structural_facts_run_id().to_owned(),
            run_dir: roots
                .structure_run_root
                .join(request.structural_facts_run_id()),
        },
        BootstrapStageTarget {
            stage: EpistemeOntologyBootstrapArtifactCacheStage::ReasoningPacket,
            stage_label: "reasoning-packet",
            run_id: request.reasoning_packet_run_id().to_owned(),
            run_dir: roots
                .ontology_generation_run_root
                .join(request.reasoning_packet_run_id()),
        },
        BootstrapStageTarget {
            stage: EpistemeOntologyBootstrapArtifactCacheStage::ReasoningLedgerSeed,
            stage_label: "reasoning-ledger-seed",
            run_id: request.reasoning_ledger_seed_run_id().to_owned(),
            run_dir: roots
                .ontology_generation_run_root
                .join(request.reasoning_ledger_seed_run_id()),
        },
        BootstrapStageTarget {
            stage: EpistemeOntologyBootstrapArtifactCacheStage::ReasoningFillPlan,
            stage_label: "reasoning-fill-plan",
            run_id: request.reasoning_fill_plan_run_id().to_owned(),
            run_dir: roots
                .ontology_generation_run_root
                .join(request.reasoning_fill_plan_run_id()),
        },
    ])
}

#[cfg(feature = "artifact-cache")]
struct BootstrapArtifactRoots {
    structure_run_root: PathBuf,
    ontology_generation_run_root: PathBuf,
}

#[cfg(feature = "artifact-cache")]
fn resolve_bootstrap_artifact_roots(
    request: &EpistemeOntologyBootstrapPipelineRequest,
) -> Result<BootstrapArtifactRoots> {
    let config = load_episteme_runtime_config(request.episteme_root())
        .context("failed to load Episteme runtime config")?;
    let structure_run_root = request
        .structure_run_root()
        .map(Path::to_path_buf)
        .or_else(|| {
            config
                .as_ref()
                .and_then(|config| config.structure_runs.clone())
        })
        .unwrap_or_else(|| request.episteme_root().join("runs/structure"));
    let ontology_generation_run_root = request
        .ontology_generation_run_root()
        .map(Path::to_path_buf)
        .or_else(|| {
            config
                .as_ref()
                .and_then(|config| config.ontology_generation_runs.clone())
        })
        .unwrap_or_else(|| request.episteme_root().join("runs/ontology-generation"));
    Ok(BootstrapArtifactRoots {
        structure_run_root,
        ontology_generation_run_root,
    })
}

fn default_stage_run_id(run_id: &str, suffix: &str) -> String {
    format!("{run_id}_{suffix}")
}

#[must_use]
pub(super) fn structural_facts_stage_run_id(run_id: &str) -> String {
    default_stage_run_id(run_id, "structural_facts")
}

#[must_use]
pub(super) fn reasoning_packet_stage_run_id(run_id: &str) -> String {
    default_stage_run_id(run_id, "reasoning_packet")
}

#[must_use]
pub(super) fn reasoning_ledger_seed_stage_run_id(run_id: &str) -> String {
    default_stage_run_id(run_id, "reasoning_ledger_seed")
}

#[must_use]
pub(super) fn reasoning_fill_plan_stage_run_id(run_id: &str) -> String {
    default_stage_run_id(run_id, "reasoning_fill_plan")
}
