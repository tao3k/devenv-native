use crate::bin_support::wendao::cli_support::emit;
#[cfg(feature = "episteme-foyer-artifact-cache")]
use crate::bin_support::wendao::types::EpistemeBootstrapArtifactCacheModeArg;
use crate::bin_support::wendao::types::{Cli, EpistemeBootstrapPipelineArgs};
use anyhow::Result;
use xiuxian_wendao_episteme::{
    EpistemeOntologyBootstrapPipelineRequest, run_episteme_ontology_bootstrap_pipeline,
};

#[cfg(feature = "episteme-foyer-artifact-cache")]
use super::artifact::run_episteme_bootstrap_pipeline_artifact_cache_command;
use crate::bin_support::wendao::execute::episteme::root::resolve_episteme_root;

pub(in crate::bin_support::wendao::execute::episteme) fn run_episteme_bootstrap_pipeline_command(
    cli: &Cli,
    args: &EpistemeBootstrapPipelineArgs,
) -> Result<()> {
    let request = episteme_bootstrap_pipeline_request(cli, args)?;
    #[cfg(feature = "episteme-foyer-artifact-cache")]
    {
        if args.artifact_cache_mode != EpistemeBootstrapArtifactCacheModeArg::Disabled {
            return run_episteme_bootstrap_pipeline_artifact_cache_command(cli, args, &request);
        }
    }
    let report = run_episteme_ontology_bootstrap_pipeline(&request)?;
    emit(&report, cli.output_or_json())
}

fn episteme_bootstrap_pipeline_request(
    cli: &Cli,
    args: &EpistemeBootstrapPipelineArgs,
) -> Result<EpistemeOntologyBootstrapPipelineRequest> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let mut request =
        EpistemeOntologyBootstrapPipelineRequest::new(episteme_root, args.run_id.clone())
            .with_validation_mode(args.validation_mode.into())
            .with_reasoning_packet_limit(args.reasoning_packet_limit)
            .with_reasoning_ledger_seed_limit(args.reasoning_ledger_seed_limit)
            .with_reasoning_fill_plan_limit(args.reasoning_fill_plan_limit);
    if let Some(corpus_root) = &args.corpus_root {
        request = request.with_corpus_root(corpus_root.clone());
    }
    if let Some(run_root) = &args.structure_run_root {
        request = request.with_structure_run_root(run_root.clone());
    }
    if let Some(run_root) = &args.ontology_generation_run_root {
        request = request.with_ontology_generation_run_root(run_root.clone());
    }
    if let Some(category) = &args.category {
        request = request.with_category(category.clone());
    }
    if let Some(route) = &args.route {
        request = request.with_route(route.clone());
    }
    Ok(request)
}
