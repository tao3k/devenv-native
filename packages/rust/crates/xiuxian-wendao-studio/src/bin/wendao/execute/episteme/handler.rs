use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::{
    Cli, Command, EpistemeApplyOntologySourcePatchArgs, EpistemeCommand, EpistemeEvidenceCommand,
    EpistemeEvidenceReadValidationModeArg, EpistemeEvidenceSelectionValidationModeArg,
    EpistemeGenerateOntologyCandidatesArgs, EpistemePlanExtractionRunArgs,
    EpistemeReadEvidenceArgs, EpistemeReviewOntologyCandidatesArgs, EpistemeSourceContractCommand,
    EpistemeStructuralIdfValidationModeArg, EpistemeStructureCommand,
    EpistemeStructureTocValidationModeArg, EpistemeWriteEvidenceSelectionPlanArgs,
    EpistemeWriteOntologyPromotionApplyPlanArgs, EpistemeWriteOntologyPromotionReviewArgs,
    EpistemeWriteOntologyRdfDraftArgs, EpistemeWriteOntologySourcePatchApplyPlanArgs,
    EpistemeWriteOntologySourcePatchApplyPreviewArgs, EpistemeWriteOntologySourcePatchDraftArgs,
    EpistemeWriteOntologySourcePatchPreflightArgs,
    EpistemeWriteOntologySourcePatchRdfReadModelArgs,
    EpistemeWriteOntologySourcePatchReviewPacketArgs,
    EpistemeWriteOntologySourcePatchSemanticPreviewArgs, EpistemeWriteStructuralIdfArgs,
    EpistemeWriteStructuralIdfReasoningFillPlanArgs,
    EpistemeWriteStructuralIdfReasoningLedgerSeedArgs,
    EpistemeWriteStructuralIdfReasoningPacketArgs,
    EpistemeWriteStructuralIdfReasoningQianjiSchedulePlanArgs, EpistemeWriteStructureTocArgs,
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
    EpistemeOntologyRdfDraftExportRequest, EpistemeOntologySourcePatchApplyPlanRequest,
    EpistemeOntologySourcePatchApplyPreviewRequest, EpistemeOntologySourcePatchApplyRequest,
    EpistemeOntologySourcePatchDraftRequest, EpistemeOntologySourcePatchPreflightRequest,
    EpistemeOntologySourcePatchRdfReadModelRequest, EpistemeOntologySourcePatchReviewPacketRequest,
    EpistemeOntologySourcePatchSemanticPreviewRequest,
    EpistemeOntologyStructuralIdfReasoningFillPlanRequest,
    EpistemeOntologyStructuralIdfReasoningLedgerSeedRequest,
    EpistemeOntologyStructuralIdfReasoningPacketRequest,
    EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest,
    EpistemeOntologyStructuralIdfRequest, EpistemeOntologyStructuralIdfValidationMode,
    apply_episteme_ontology_source_patch, export_episteme_ontology_rdf_draft,
    export_episteme_ontology_source_patch_draft, generate_episteme_ontology_candidates,
    review_episteme_ontology_candidates, write_episteme_ontology_promotion_apply_plan,
    write_episteme_ontology_promotion_review_packet,
    write_episteme_ontology_source_patch_apply_plan,
    write_episteme_ontology_source_patch_apply_preview,
    write_episteme_ontology_source_patch_preflight,
    write_episteme_ontology_source_patch_rdf_read_model,
    write_episteme_ontology_source_patch_review_packet,
    write_episteme_ontology_source_patch_semantic_preview, write_episteme_ontology_structural_idf,
    write_episteme_ontology_structural_idf_reasoning_fill_plan,
    write_episteme_ontology_structural_idf_reasoning_ledger_seed,
    write_episteme_ontology_structural_idf_reasoning_packet,
    write_episteme_ontology_structural_idf_reasoning_qianji_schedule_plan,
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
            EpistemeSourceContractCommand::WriteStructuralIdf(args) => {
                write_episteme_structural_idf_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteStructuralIdfReasoningPacket(args) => {
                write_episteme_structural_idf_reasoning_packet_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteStructuralIdfReasoningLedgerSeed(args) => {
                write_episteme_structural_idf_reasoning_ledger_seed_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteStructuralIdfReasoningFillPlan(args) => {
                write_episteme_structural_idf_reasoning_fill_plan_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteStructuralIdfReasoningQianjiSchedulePlan(args) => {
                write_episteme_structural_idf_reasoning_qianji_schedule_plan_command(cli, args)
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
            EpistemeSourceContractCommand::WriteOntologySourcePatchPreflight(args) => {
                write_episteme_ontology_source_patch_preflight_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteOntologySourcePatchDraft(args) => {
                write_episteme_ontology_source_patch_draft_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteOntologySourcePatchApplyPlan(args) => {
                write_episteme_ontology_source_patch_apply_plan_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteOntologySourcePatchReviewPacket(args) => {
                write_episteme_ontology_source_patch_review_packet_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteOntologySourcePatchApplyPreview(args) => {
                write_episteme_ontology_source_patch_apply_preview_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteOntologySourcePatchSemanticPreview(args) => {
                write_episteme_ontology_source_patch_semantic_preview_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteOntologySourcePatchRdfReadModel(args) => {
                write_episteme_ontology_source_patch_rdf_read_model_command(cli, args)
            }
            EpistemeSourceContractCommand::ApplyOntologySourcePatch(args) => {
                apply_episteme_ontology_source_patch_command(cli, args)
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

fn write_episteme_structural_idf_command(
    cli: &Cli,
    args: &EpistemeWriteStructuralIdfArgs,
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
        EpistemeOntologyStructuralIdfRequest::new(&episteme_root, corpus_root, args.run_id.clone())
            .with_validation_mode(args.validation_mode.into());
    let report = write_episteme_ontology_structural_idf(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_structural_idf_reasoning_packet_command(
    cli: &Cli,
    args: &EpistemeWriteStructuralIdfReasoningPacketArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let structure_run_root = resolve_run_root(
        args.structure_run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.structure_runs.as_ref()),
        || episteme_root.join("runs/structure"),
    );
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let structural_idf_json = structure_run_root
        .join(&args.structural_idf_run_id)
        .join("structural_idf.json");
    let mut request = EpistemeOntologyStructuralIdfReasoningPacketRequest::new(
        structural_idf_json,
        args.run_id.clone(),
    )
    .with_limit(args.limit);
    if let Some(category) = &args.category {
        request = request.with_category(category.clone());
    }
    if let Some(route) = &args.route {
        request = request.with_route(route.clone());
    }
    let report = write_episteme_ontology_structural_idf_reasoning_packet(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_structural_idf_reasoning_ledger_seed_command(
    cli: &Cli,
    args: &EpistemeWriteStructuralIdfReasoningLedgerSeedArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let reasoning_packet_root = resolve_run_root(
        args.reasoning_packet_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let reasoning_packet_json = reasoning_packet_root
        .join(&args.reasoning_packet_run_id)
        .join("reasoning_packet.json");
    let request = EpistemeOntologyStructuralIdfReasoningLedgerSeedRequest::new(
        reasoning_packet_json,
        args.run_id.clone(),
    )
    .with_limit(args.limit);
    let report = write_episteme_ontology_structural_idf_reasoning_ledger_seed(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_structural_idf_reasoning_fill_plan_command(
    cli: &Cli,
    args: &EpistemeWriteStructuralIdfReasoningFillPlanArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let ledger_seed_root = resolve_run_root(
        args.ledger_seed_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let reasoning_ledger_seed_json = ledger_seed_root
        .join(&args.ledger_seed_run_id)
        .join("reasoning_ledger_seed.json");
    let request = EpistemeOntologyStructuralIdfReasoningFillPlanRequest::new(
        reasoning_ledger_seed_json,
        args.run_id.clone(),
    )
    .with_limit(args.limit);
    let report = write_episteme_ontology_structural_idf_reasoning_fill_plan(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_structural_idf_reasoning_qianji_schedule_plan_command(
    cli: &Cli,
    args: &EpistemeWriteStructuralIdfReasoningQianjiSchedulePlanArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let config = load_runtime_config(episteme_root.as_path())?;
    let fill_plan_root = resolve_run_root(
        args.fill_plan_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let run_root = resolve_run_root(
        args.run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.ontology_generation_runs.as_ref()),
        || episteme_root.join("runs/ontology-generation"),
    );
    let reasoning_fill_plan_json = fill_plan_root
        .join(&args.fill_plan_run_id)
        .join("reasoning_fill_plan.json");
    let mut request = EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest::new(
        reasoning_fill_plan_json,
        args.run_id.clone(),
    )
    .with_limit(args.limit);
    if let Some(qianji_run_id) = &args.qianji_run_id {
        request = request.with_qianji_run_id(qianji_run_id.clone());
    }
    if let Some(model) = &args.openai_compatible_model {
        request = request
            .with_openai_compatible_prompt_audit(model.clone(), args.openai_compatible_max_tokens);
    }
    let report =
        write_episteme_ontology_structural_idf_reasoning_qianji_schedule_plan(&request, run_root)?;
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

fn write_episteme_ontology_source_patch_preflight_command(
    cli: &Cli,
    args: &EpistemeWriteOntologySourcePatchPreflightArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let run_root = resolve_run_root(args.run_root.as_ref(), None, || {
        episteme_root.join("runs/source-patch-preflight")
    });
    let request = EpistemeOntologySourcePatchPreflightRequest::new(
        &episteme_root,
        run_root.join(&args.run_id),
    );
    let report = write_episteme_ontology_source_patch_preflight(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_ontology_source_patch_draft_command(
    cli: &Cli,
    args: &EpistemeWriteOntologySourcePatchDraftArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let run_root = resolve_run_root(args.run_root.as_ref(), None, || {
        episteme_root.join("runs/source-patch-preflight")
    });
    let request = EpistemeOntologySourcePatchDraftRequest::new(run_root.join(&args.run_id));
    let report = export_episteme_ontology_source_patch_draft(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_ontology_source_patch_apply_plan_command(
    cli: &Cli,
    args: &EpistemeWriteOntologySourcePatchApplyPlanArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let run_root = resolve_run_root(args.run_root.as_ref(), None, || {
        episteme_root.join("runs/source-patch-preflight")
    });
    let request = EpistemeOntologySourcePatchApplyPlanRequest::new(run_root.join(&args.run_id));
    let report = write_episteme_ontology_source_patch_apply_plan(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_ontology_source_patch_review_packet_command(
    cli: &Cli,
    args: &EpistemeWriteOntologySourcePatchReviewPacketArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let run_root = resolve_run_root(args.run_root.as_ref(), None, || {
        episteme_root.join("runs/source-patch-preflight")
    });
    let request = EpistemeOntologySourcePatchReviewPacketRequest::new(
        &episteme_root,
        run_root.join(&args.run_id),
    );
    let report = write_episteme_ontology_source_patch_review_packet(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_ontology_source_patch_apply_preview_command(
    cli: &Cli,
    args: &EpistemeWriteOntologySourcePatchApplyPreviewArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let run_root = resolve_run_root(args.run_root.as_ref(), None, || {
        episteme_root.join("runs/source-patch-preflight")
    });
    let request = EpistemeOntologySourcePatchApplyPreviewRequest::new(
        &episteme_root,
        run_root.join(&args.run_id),
        args.expected_apply_plan_tsv_sha256.clone(),
    );
    let report = write_episteme_ontology_source_patch_apply_preview(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_ontology_source_patch_semantic_preview_command(
    cli: &Cli,
    args: &EpistemeWriteOntologySourcePatchSemanticPreviewArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let run_root = resolve_run_root(args.run_root.as_ref(), None, || {
        episteme_root.join("runs/source-patch-preflight")
    });
    let request =
        EpistemeOntologySourcePatchSemanticPreviewRequest::new(run_root.join(&args.run_id));
    let report = write_episteme_ontology_source_patch_semantic_preview(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_ontology_source_patch_rdf_read_model_command(
    cli: &Cli,
    args: &EpistemeWriteOntologySourcePatchRdfReadModelArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let run_root = resolve_run_root(args.run_root.as_ref(), None, || {
        episteme_root.join("runs/source-patch-preflight")
    });
    let request = EpistemeOntologySourcePatchRdfReadModelRequest::new(
        &episteme_root,
        run_root.join(&args.run_id),
    );
    let report = write_episteme_ontology_source_patch_rdf_read_model(&request)?;
    emit(&report, cli.output_or_json())
}

fn apply_episteme_ontology_source_patch_command(
    cli: &Cli,
    args: &EpistemeApplyOntologySourcePatchArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let run_root = resolve_run_root(args.run_root.as_ref(), None, || {
        episteme_root.join("runs/source-patch-preflight")
    });
    let request =
        EpistemeOntologySourcePatchApplyRequest::new(&episteme_root, run_root.join(&args.run_id))
            .with_expected_apply_plan_tsv_sha256(args.expected_apply_plan_tsv_sha256.clone())
            .with_allow_source_mutation(args.allow_source_mutation);
    let report = apply_episteme_ontology_source_patch(&request)?;
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

impl From<EpistemeStructuralIdfValidationModeArg> for EpistemeOntologyStructuralIdfValidationMode {
    fn from(value: EpistemeStructuralIdfValidationModeArg) -> Self {
        match value {
            EpistemeStructuralIdfValidationModeArg::MetadataOnly => Self::MetadataOnly,
            EpistemeStructuralIdfValidationModeArg::FullHash => Self::FullHash,
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
