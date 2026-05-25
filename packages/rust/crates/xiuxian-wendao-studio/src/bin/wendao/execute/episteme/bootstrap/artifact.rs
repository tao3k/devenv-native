use std::path::Path;

use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::{
    Cli, EpistemeBootstrapArtifactCacheModeArg, EpistemeBootstrapPipelineArgs,
};
use anyhow::Result;
use xiuxian_db_store::artifact_cache::{ArtifactBlobCacheBackendConfig, ArtifactKey};
use xiuxian_wendao_episteme::{
    EpistemeOntologyBootstrapArtifactCacheOptions,
    EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome,
    EpistemeOntologyBootstrapArtifactCacheReadThroughReport,
    EpistemeOntologyBootstrapArtifactCacheReport,
    EpistemeOntologyBootstrapArtifactCacheRestoreReport,
    EpistemeOntologyBootstrapArtifactCacheStage, EpistemeOntologyBootstrapPipelineRequest,
    admit_episteme_ontology_bootstrap_artifact_cache_options,
    read_through_episteme_ontology_bootstrap_artifacts,
    restore_episteme_ontology_bootstrap_pipeline_artifacts,
    run_episteme_ontology_bootstrap_pipeline,
    run_episteme_ontology_bootstrap_pipeline_with_artifact_cache,
};

pub(super) fn run_episteme_bootstrap_pipeline_artifact_cache_command(
    cli: &Cli,
    args: &EpistemeBootstrapPipelineArgs,
    request: &EpistemeOntologyBootstrapPipelineRequest,
) -> Result<()> {
    let options = episteme_bootstrap_artifact_cache_options(args)?;
    let config = ArtifactBlobCacheBackendConfig::from_env()?;
    let cache = config.build()?;
    let value = match args.artifact_cache_mode {
        EpistemeBootstrapArtifactCacheModeArg::Disabled => {
            serde_json::to_value(run_episteme_ontology_bootstrap_pipeline(request)?)?
        }
        EpistemeBootstrapArtifactCacheModeArg::WriteThrough => {
            let report = run_episteme_ontology_bootstrap_pipeline_with_artifact_cache(
                request, &cache, &options,
            )?;
            episteme_bootstrap_artifact_report_json(
                "write-through",
                report,
                cache.backend_name(),
                config.root(),
            )
        }
        EpistemeBootstrapArtifactCacheModeArg::ReadThrough => {
            let report =
                read_through_episteme_ontology_bootstrap_artifacts(request, &cache, &options)?;
            episteme_bootstrap_readthrough_report_json(report, cache.backend_name(), config.root())
        }
        EpistemeBootstrapArtifactCacheModeArg::RestoreOnly => {
            let report =
                restore_episteme_ontology_bootstrap_pipeline_artifacts(request, &cache, &options)?;
            episteme_bootstrap_restore_report_json(
                "restore-only",
                &report,
                cache.backend_name(),
                config.root(),
            )
        }
    };
    emit(&value, cli.output_or_json())
}

pub(in crate::bin_support::wendao::execute::episteme) fn episteme_bootstrap_artifact_cache_options(
    args: &EpistemeBootstrapPipelineArgs,
) -> Result<EpistemeOntologyBootstrapArtifactCacheOptions> {
    let source_digest = args
        .artifact_cache_source_digest
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "--artifact-cache-source-digest is required when artifact cache mode is enabled"
            )
        })?;
    let profile_digest = args
        .artifact_cache_profile_digest
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "--artifact-cache-profile-digest is required when artifact cache mode is enabled"
            )
        })?;
    admit_episteme_ontology_bootstrap_artifact_cache_options(source_digest, profile_digest)
}

fn episteme_bootstrap_artifact_report_json(
    mode: &str,
    report: EpistemeOntologyBootstrapArtifactCacheReport,
    backend: &str,
    root: &Path,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": "xiuxian_wendao.studio_episteme_bootstrap_artifact_cache.v1",
        "mode": mode,
        "backend": backend,
        "root": root,
        "pipeline": report.pipeline,
        "bundles": report.bundles.into_iter().map(|bundle| {
            serde_json::json!({
                "artifactKey": episteme_artifact_key_json(&bundle.artifact_key),
                "sourceDir": bundle.source_dir,
                "byteLen": bundle.byte_len,
                "replaced": bundle.replaced,
            })
        }).collect::<Vec<_>>(),
    })
}

fn episteme_bootstrap_readthrough_report_json(
    report: EpistemeOntologyBootstrapArtifactCacheReadThroughReport,
    backend: &str,
    root: &Path,
) -> serde_json::Value {
    let outcome = match report.outcome {
        EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome::Restored => "restored",
        EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome::Generated => "generated",
    };
    serde_json::json!({
        "schemaVersion": "xiuxian_wendao.studio_episteme_bootstrap_artifact_cache.v1",
        "mode": "read-through",
        "backend": backend,
        "root": root,
        "outcome": outcome,
        "restore": episteme_bootstrap_restore_json(&report.restore),
        "generated": report.generated.map(|generated| {
            episteme_bootstrap_artifact_report_json("write-through", generated, backend, root)
        }),
    })
}

fn episteme_bootstrap_restore_report_json(
    mode: &str,
    report: &EpistemeOntologyBootstrapArtifactCacheRestoreReport,
    backend: &str,
    root: &Path,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": "xiuxian_wendao.studio_episteme_bootstrap_artifact_cache.v1",
        "mode": mode,
        "backend": backend,
        "root": root,
        "restore": episteme_bootstrap_restore_json(report),
    })
}

fn episteme_bootstrap_restore_json(
    report: &EpistemeOntologyBootstrapArtifactCacheRestoreReport,
) -> serde_json::Value {
    serde_json::json!({
        "complete": report.complete(),
        "restored": report.restored.iter().map(|restored| {
            serde_json::json!({
                "artifactKey": episteme_artifact_key_json(&restored.artifact_key),
                "targetDir": restored.target_dir,
                "byteLen": restored.byte_len,
            })
        }).collect::<Vec<_>>(),
        "missing": report.missing.iter().map(|missing| {
            serde_json::json!({
                "stage": episteme_bootstrap_stage_name(missing.stage),
                "runDigest": missing.run_digest,
                "targetDir": missing.target_dir,
            })
        }).collect::<Vec<_>>(),
    })
}

fn episteme_artifact_key_json(key: &ArtifactKey) -> serde_json::Value {
    serde_json::json!({
        "namespace": key.namespace().as_str(),
        "kind": key.kind().as_storage_component(),
        "sourceDigest": key.source_digest().as_str(),
        "profileDigest": key.profile_digest().as_str(),
        "shardDigest": key.shard_digest().as_str(),
    })
}

const fn episteme_bootstrap_stage_name(
    stage: EpistemeOntologyBootstrapArtifactCacheStage,
) -> &'static str {
    match stage {
        EpistemeOntologyBootstrapArtifactCacheStage::StructuralFacts => "structural-facts",
        EpistemeOntologyBootstrapArtifactCacheStage::ReasoningPacket => "reasoning-packet",
        EpistemeOntologyBootstrapArtifactCacheStage::ReasoningLedgerSeed => "reasoning-ledger-seed",
        EpistemeOntologyBootstrapArtifactCacheStage::ReasoningFillPlan => "reasoning-fill-plan",
    }
}
