//! Crate-owned deterministic ontology bootstrap pipeline.

mod types;

use std::path::Path;

use anyhow::{Context, Result};

use crate::load_episteme_runtime_config;

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

pub use types::{
    EpistemeOntologyBootstrapPipelineReport, EpistemeOntologyBootstrapPipelineRequest,
    EpistemeOntologyBootstrapPipelineSafetyFlags,
};

const BOOTSTRAP_PIPELINE_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_bootstrap_pipeline_report.v1";

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
