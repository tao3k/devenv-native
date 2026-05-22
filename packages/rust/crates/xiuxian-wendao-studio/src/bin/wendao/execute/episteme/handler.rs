use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::{
    Cli, Command, EpistemeCommand, EpistemeEvidenceCommand, EpistemeEvidenceReadValidationModeArg,
    EpistemeEvidenceSelectionValidationModeArg, EpistemeGenerateOntologyCandidatesArgs,
    EpistemePlanExtractionRunArgs, EpistemeReadEvidenceArgs, EpistemeReviewOntologyCandidatesArgs,
    EpistemeSourceContractCommand, EpistemeStructureCommand, EpistemeStructureTocValidationModeArg,
    EpistemeWriteEvidenceSelectionPlanArgs, EpistemeWriteOntologyPromotionApplyPlanArgs,
    EpistemeWriteOntologyPromotionReviewArgs, EpistemeWriteOntologyRdfDraftArgs,
    EpistemeWriteStructureTocArgs,
};
use anyhow::Result;
use xiuxian_wendao::episteme::{
    EpistemeEvidenceReadRequest, EpistemeEvidenceReadValidationMode,
    EpistemeEvidenceSelectionPlanRequest, EpistemeEvidenceSelectionValidationMode,
    EpistemeRunPlanRequest, EpistemeStructureTocRequest, EpistemeStructureTocValidationMode,
    read_episteme_evidence, read_episteme_evidence_selection_file_ids,
    write_episteme_evidence_selection_plan, write_episteme_extraction_run_plan,
    write_episteme_structure_toc,
};
use xiuxian_wendao_episteme::{
    EpistemeOntologyCandidateGenerationRequest, EpistemeOntologyCandidateReviewRequest,
    EpistemeOntologyPromotionApplyPlanRequest, EpistemeOntologyPromotionReviewPacketRequest,
    EpistemeOntologyRdfDraftExportRequest, export_episteme_ontology_rdf_draft,
    generate_episteme_ontology_candidates, review_episteme_ontology_candidates,
    write_episteme_ontology_promotion_apply_plan, write_episteme_ontology_promotion_review_packet,
};

use super::cache::{
    run_episteme_docling_document_cache, run_episteme_image_ocr_cache,
    run_episteme_legacy_office_conversion,
};
use super::root::{
    load_runtime_config, resolve_corpus_root, resolve_episteme_root, resolve_run_root,
};

pub(crate) fn handle(cli: &Cli) -> Result<()> {
    let Command::Episteme { command } = &cli.command else {
        unreachable!("episteme handler must be called with episteme command");
    };

    match command {
        EpistemeCommand::Evidence { command } => match command {
            EpistemeEvidenceCommand::Read(args) => read_episteme_evidence_command(cli, args),
            EpistemeEvidenceCommand::WriteSelectionPlan(args) => {
                write_episteme_evidence_selection_plan_command(cli, args)
            }
        },
        EpistemeCommand::SourceContract { command } => match command {
            EpistemeSourceContractCommand::PlanExtractionRun(args) => {
                plan_episteme_source_contract(cli, args)
            }
            EpistemeSourceContractCommand::RunImageOcrCache(args) => {
                run_episteme_image_ocr_cache(cli, args)
            }
            EpistemeSourceContractCommand::RunDoclingDocumentCache(args) => {
                run_episteme_docling_document_cache(cli, args)
            }
            EpistemeSourceContractCommand::RunLegacyOfficeConversion(args) => {
                run_episteme_legacy_office_conversion(cli, args)
            }
            EpistemeSourceContractCommand::GenerateOntologyCandidates(args) => {
                generate_episteme_ontology_candidates_command(cli, args)
            }
            EpistemeSourceContractCommand::ReviewOntologyCandidates(args) => {
                review_episteme_ontology_candidates_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteOntologyRdfDraft(args) => {
                write_episteme_ontology_rdf_draft_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteOntologyPromotionReview(args) => {
                write_episteme_ontology_promotion_review_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteOntologyPromotionApplyPlan(args) => {
                write_episteme_ontology_promotion_apply_plan_command(cli, args)
            }
        },
        EpistemeCommand::Structure { command } => match command {
            EpistemeStructureCommand::WriteToc(args) => {
                write_episteme_structure_toc_command(cli, args)
            }
        },
    }
}

fn write_episteme_evidence_selection_plan_command(
    cli: &Cli,
    args: &EpistemeWriteEvidenceSelectionPlanArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let corpus_root = resolve_corpus_root(
        args.corpus_root.as_ref(),
        episteme_root.as_path(),
        config.as_ref(),
    )?;
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.evidence_selection_runs.as_ref()),
        || episteme_root.join("runs/evidence-selection"),
    );
    let request = EpistemeEvidenceSelectionPlanRequest::new(
        &episteme_root,
        corpus_root,
        args.run_id.clone(),
        args.file_ids.clone(),
    )
    .with_selection_reason(args.selection_reason.clone())
    .with_validation_mode(args.validation_mode.into());
    let report = write_episteme_evidence_selection_plan(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

fn read_episteme_evidence_command(cli: &Cli, args: &EpistemeReadEvidenceArgs) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let corpus_root = resolve_corpus_root(
        args.corpus_root.as_ref(),
        episteme_root.as_path(),
        config.as_ref(),
    )?;
    let request =
        EpistemeEvidenceReadRequest::new(&episteme_root, corpus_root, args.file_id.clone())
            .with_max_preview_bytes(args.max_preview_bytes)
            .with_validation_mode(args.validation_mode.into());
    let report = read_episteme_evidence(&request)?;
    emit(&report, cli.output_or_json())
}

fn plan_episteme_source_contract(cli: &Cli, args: &EpistemePlanExtractionRunArgs) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let corpus_root = resolve_corpus_root(
        args.corpus_root.as_ref(),
        episteme_root.as_path(),
        config.as_ref(),
    )?;
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.extraction_runs.as_ref()),
        || episteme_root.join("runs/extraction"),
    );
    let mut request = EpistemeRunPlanRequest::new(&episteme_root, corpus_root, args.run_id.clone())
        .with_limit(args.limit);
    if let Some(route) = &args.route {
        request = request.with_route(route.clone());
    }
    if let Some(category) = &args.category {
        request = request.with_category(category.clone());
    }
    if let Some(selection_run_id) = &args.selection_run_id {
        let selection_root = args
            .selection_root
            .clone()
            .or_else(|| {
                config
                    .as_ref()
                    .and_then(|config| config.evidence_selection_runs.clone())
            })
            .unwrap_or_else(|| episteme_root.join("runs/evidence-selection"));
        let selection_tsv_path = selection_root.join(selection_run_id).join("selection.tsv");
        let selected_file_ids = read_episteme_evidence_selection_file_ids(selection_tsv_path)?;
        request = request.with_selected_file_ids(selected_file_ids);
    }

    let report = write_episteme_extraction_run_plan(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

fn generate_episteme_ontology_candidates_command(
    cli: &Cli,
    args: &EpistemeGenerateOntologyCandidatesArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let extraction_run_root = resolve_run_root(
        args.extraction_run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.extraction_runs.as_ref()),
        || episteme_root.join("runs/extraction"),
    );
    let request = EpistemeOntologyCandidateGenerationRequest::new(
        &episteme_root,
        args.run_id.clone(),
        extraction_run_root,
    )
    .with_extraction_run_ids(args.extraction_run_ids.clone());
    let report = generate_episteme_ontology_candidates(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

fn review_episteme_ontology_candidates_command(
    cli: &Cli,
    args: &EpistemeReviewOntologyCandidatesArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let request = EpistemeOntologyCandidateReviewRequest::new(run_root.join(&args.run_id));
    let report = review_episteme_ontology_candidates(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_ontology_rdf_draft_command(
    cli: &Cli,
    args: &EpistemeWriteOntologyRdfDraftArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let request = EpistemeOntologyRdfDraftExportRequest::new(run_root.join(&args.run_id));
    let report = export_episteme_ontology_rdf_draft(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_ontology_promotion_review_command(
    cli: &Cli,
    args: &EpistemeWriteOntologyPromotionReviewArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let request = EpistemeOntologyPromotionReviewPacketRequest::new(run_root.join(&args.run_id));
    let report = write_episteme_ontology_promotion_review_packet(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_ontology_promotion_apply_plan_command(
    cli: &Cli,
    args: &EpistemeWriteOntologyPromotionApplyPlanArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let request = EpistemeOntologyPromotionApplyPlanRequest::new(run_root.join(&args.run_id));
    let report = write_episteme_ontology_promotion_apply_plan(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_structure_toc_command(
    cli: &Cli,
    args: &EpistemeWriteStructureTocArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let corpus_root = resolve_corpus_root(
        args.corpus_root.as_ref(),
        episteme_root.as_path(),
        config.as_ref(),
    )?;
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.structure_runs.as_ref()),
        || episteme_root.join("runs/structure"),
    );
    let request =
        EpistemeStructureTocRequest::new(&episteme_root, corpus_root, args.run_id.clone())
            .with_validation_mode(args.validation_mode.into());
    let report = write_episteme_structure_toc(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

impl From<EpistemeStructureTocValidationModeArg> for EpistemeStructureTocValidationMode {
    fn from(value: EpistemeStructureTocValidationModeArg) -> Self {
        match value {
            EpistemeStructureTocValidationModeArg::MetadataOnly => Self::MetadataOnly,
            EpistemeStructureTocValidationModeArg::FullHash => Self::FullHash,
        }
    }
}

impl From<EpistemeEvidenceReadValidationModeArg> for EpistemeEvidenceReadValidationMode {
    fn from(value: EpistemeEvidenceReadValidationModeArg) -> Self {
        match value {
            EpistemeEvidenceReadValidationModeArg::MetadataOnly => Self::MetadataOnly,
            EpistemeEvidenceReadValidationModeArg::FullHash => Self::FullHash,
        }
    }
}

impl From<EpistemeEvidenceSelectionValidationModeArg> for EpistemeEvidenceSelectionValidationMode {
    fn from(value: EpistemeEvidenceSelectionValidationModeArg) -> Self {
        match value {
            EpistemeEvidenceSelectionValidationModeArg::MetadataOnly => Self::MetadataOnly,
            EpistemeEvidenceSelectionValidationModeArg::FullHash => Self::FullHash,
        }
    }
}
