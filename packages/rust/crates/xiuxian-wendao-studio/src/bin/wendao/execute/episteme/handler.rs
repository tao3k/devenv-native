use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::{
    Cli, Command, EpistemeApplyOntologySourcePatchArgs, EpistemeCommand, EpistemeEvidenceCommand,
    EpistemeEvidenceReadValidationModeArg, EpistemeEvidenceSelectionValidationModeArg,
    EpistemeGenerateOntologyCandidatesArgs, EpistemeImportQianjiReviewCandidatesArgs,
    EpistemeInspectOntologyCandidatesArgs, EpistemePlanExtractionRunArgs, EpistemeReadEvidenceArgs,
    EpistemeReviewOntologyCandidatesArgs, EpistemeSourceContractCommand,
    EpistemeStructuralFactsValidationModeArg, EpistemeStructureCommand,
    EpistemeStructureTocValidationModeArg, EpistemeWriteEvidenceSelectionPlanArgs,
    EpistemeWriteOntologyPromotionApplyPlanArgs, EpistemeWriteOntologyPromotionReviewArgs,
    EpistemeWriteOntologyRdfDraftArgs, EpistemeWriteOntologySourcePatchApplyPlanArgs,
    EpistemeWriteOntologySourcePatchApplyPreviewArgs, EpistemeWriteOntologySourcePatchDraftArgs,
    EpistemeWriteOntologySourcePatchPreflightArgs,
    EpistemeWriteOntologySourcePatchRdfReadModelArgs,
    EpistemeWriteOntologySourcePatchReviewPacketArgs,
    EpistemeWriteOntologySourcePatchSemanticPreviewArgs, EpistemeWriteStructuralFactsArgs,
    EpistemeWriteStructuralFactsReasoningFillPlanArgs,
    EpistemeWriteStructuralFactsReasoningLedgerSeedArgs,
    EpistemeWriteStructuralFactsReasoningPacketArgs,
    EpistemeWriteStructuralFactsReasoningQianjiSchedulePlanArgs, EpistemeWriteStructureTocArgs,
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
    EpistemeOntologyQianjiReviewCandidateImportRequest, EpistemeOntologyRdfDraftExportRequest,
    EpistemeOntologySourcePatchApplyPlanRequest, EpistemeOntologySourcePatchApplyPreviewRequest,
    EpistemeOntologySourcePatchApplyRequest, EpistemeOntologySourcePatchDraftRequest,
    EpistemeOntologySourcePatchPreflightRequest, EpistemeOntologySourcePatchRdfReadModelRequest,
    EpistemeOntologySourcePatchReviewPacketRequest,
    EpistemeOntologySourcePatchSemanticPreviewRequest,
    EpistemeOntologyStructuralFactsConfiguredRequest,
    EpistemeOntologyStructuralFactsReasoningFillPlanRequest,
    EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest,
    EpistemeOntologyStructuralFactsReasoningPacketRequest,
    EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest,
    EpistemeOntologyStructuralFactsValidationMode, apply_episteme_ontology_source_patch,
    export_episteme_ontology_rdf_draft, export_episteme_ontology_source_patch_draft,
    generate_episteme_ontology_candidates, import_episteme_ontology_qianji_review_candidates,
    review_episteme_ontology_candidates, write_episteme_ontology_promotion_apply_plan,
    write_episteme_ontology_promotion_review_packet,
    write_episteme_ontology_source_patch_apply_plan,
    write_episteme_ontology_source_patch_apply_preview,
    write_episteme_ontology_source_patch_preflight,
    write_episteme_ontology_source_patch_rdf_read_model,
    write_episteme_ontology_source_patch_review_packet,
    write_episteme_ontology_source_patch_semantic_preview,
    write_episteme_ontology_structural_facts_from_config,
    write_episteme_ontology_structural_facts_reasoning_fill_plan,
    write_episteme_ontology_structural_facts_reasoning_ledger_seed,
    write_episteme_ontology_structural_facts_reasoning_packet,
    write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan,
};
use xiuxian_wendao_sql::candidate_read_model::{
    CandidateReadModelDuckDbInspectionRequest, inspect_candidate_read_model_with_duckdb,
};

use super::bootstrap::run_episteme_bootstrap_pipeline_command;
use super::cache::{
    run_episteme_docling_document_cache, run_episteme_image_ocr_cache,
    run_episteme_legacy_office_conversion,
};
use super::root::{
    load_runtime_config, resolve_corpus_root, resolve_episteme_root, resolve_run_root,
};

pub(super) const DEFAULT_EPISTEME_OPENAI_COMPATIBLE_PROMPT_AUDIT_MODEL: &str =
    "deepseek/deepseek-v4-pro";

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
            EpistemeSourceContractCommand::BootstrapPipeline(args) => {
                run_episteme_bootstrap_pipeline_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteStructuralFacts(args) => {
                write_episteme_structural_facts_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteStructuralFactsReasoningPacket(args) => {
                write_episteme_structural_facts_reasoning_packet_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteStructuralFactsReasoningLedgerSeed(args) => {
                write_episteme_structural_facts_reasoning_ledger_seed_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteStructuralFactsReasoningFillPlan(args) => {
                write_episteme_structural_facts_reasoning_fill_plan_command(cli, args)
            }
            EpistemeSourceContractCommand::WriteStructuralFactsReasoningQianjiSchedulePlan(
                args,
            ) => write_episteme_structural_facts_reasoning_qianji_schedule_plan_command(cli, args),
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
            EpistemeSourceContractCommand::InspectOntologyCandidates(args) => {
                inspect_episteme_ontology_candidates_command(cli, args)
            }
            EpistemeSourceContractCommand::ImportQianjiReviewCandidates(args) => {
                import_episteme_qianji_review_candidates_command(cli, args)
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

fn write_episteme_structural_facts_command(
    cli: &Cli,
    args: &EpistemeWriteStructuralFactsArgs,
) -> Result<()> {
    let episteme_root = resolve_episteme_root(
        cli,
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
    )?;
    let mut request =
        EpistemeOntologyStructuralFactsConfiguredRequest::new(&episteme_root, args.run_id.clone())
            .with_validation_mode(args.validation_mode.into());
    if let Some(corpus_root) = &args.corpus_root {
        request = request.with_corpus_root(corpus_root.clone());
    }
    if let Some(run_root) = &args.run_root {
        request = request.with_run_root(run_root.clone());
    }
    let report = write_episteme_ontology_structural_facts_from_config(&request)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_structural_facts_reasoning_packet_command(
    cli: &Cli,
    args: &EpistemeWriteStructuralFactsReasoningPacketArgs,
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
    let structural_facts_json = structure_run_root
        .join(&args.structural_facts_run_id)
        .join("structural_facts.json");
    let mut request = EpistemeOntologyStructuralFactsReasoningPacketRequest::new(
        structural_facts_json,
        args.run_id.clone(),
    )
    .with_limit(args.limit);
    if let Some(category) = &args.category {
        request = request.with_category(category.clone());
    }
    if let Some(route) = &args.route {
        request = request.with_route(route.clone());
    }
    let report = write_episteme_ontology_structural_facts_reasoning_packet(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_structural_facts_reasoning_ledger_seed_command(
    cli: &Cli,
    args: &EpistemeWriteStructuralFactsReasoningLedgerSeedArgs,
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
    let request = EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest::new(
        reasoning_packet_json,
        args.run_id.clone(),
    )
    .with_limit(args.limit);
    let report =
        write_episteme_ontology_structural_facts_reasoning_ledger_seed(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_structural_facts_reasoning_fill_plan_command(
    cli: &Cli,
    args: &EpistemeWriteStructuralFactsReasoningFillPlanArgs,
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
    let request = EpistemeOntologyStructuralFactsReasoningFillPlanRequest::new(
        reasoning_ledger_seed_json,
        args.run_id.clone(),
    )
    .with_limit(args.limit);
    let report = write_episteme_ontology_structural_facts_reasoning_fill_plan(&request, run_root)?;
    emit(&report, cli.output_or_json())
}

fn write_episteme_structural_facts_reasoning_qianji_schedule_plan_command(
    cli: &Cli,
    args: &EpistemeWriteStructuralFactsReasoningQianjiSchedulePlanArgs,
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
    let evidence_extraction_run_root = resolve_run_root(
        args.evidence_extraction_run_root.as_ref(),
        config
            .as_ref()
            .and_then(|config| config.extraction_runs.as_ref()),
        || episteme_root.join("runs/extraction"),
    );
    let reasoning_fill_plan_json = fill_plan_root
        .join(&args.fill_plan_run_id)
        .join("reasoning_fill_plan.json");
    let mut request = EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest::new(
        reasoning_fill_plan_json,
        args.run_id.clone(),
    )
    .with_limit(args.limit)
    .with_reasoning_context_shard_mode(args.reasoning_context_shard_mode.clone())
    .with_reasoning_context_shard_row_limit(args.reasoning_context_shard_row_limit)
    .with_evidence_extraction_run_root(evidence_extraction_run_root);
    if let Some(target_ledger_field_group) = &args.target_ledger_field_group {
        request = request.with_target_ledger_field_group(target_ledger_field_group.clone());
    }
    if let Some(evidence_target_intent) = &args.evidence_target_intent {
        request = request.with_evidence_target_intent(evidence_target_intent.clone());
    }
    for run_id in &args.evidence_extraction_run_ids {
        request = request.with_evidence_extraction_run_id(run_id.clone());
    }
    if let Some(qianji_run_id) = &args.qianji_run_id {
        request = request.with_qianji_run_id(qianji_run_id.clone());
    }
    if let Some(model) = openai_compatible_prompt_audit_model(args) {
        request =
            request.with_openai_compatible_prompt_audit(model, args.openai_compatible_max_tokens);
    }
    let report = write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan(
        &request, run_root,
    )?;
    emit(&report, cli.output_or_json())
}

pub(super) fn openai_compatible_prompt_audit_model(
    args: &EpistemeWriteStructuralFactsReasoningQianjiSchedulePlanArgs,
) -> Option<String> {
    if let Some(model) = &args.openai_compatible_model {
        return Some(model.clone());
    }
    if args.evidence_extraction_run_ids.is_empty() {
        None
    } else {
        Some(DEFAULT_EPISTEME_OPENAI_COMPATIBLE_PROMPT_AUDIT_MODEL.to_owned())
    }
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

fn inspect_episteme_ontology_candidates_command(
    cli: &Cli,
    args: &EpistemeInspectOntologyCandidatesArgs,
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
    let request = CandidateReadModelDuckDbInspectionRequest::from_candidate_run_dir(
        run_root.join(&args.run_id),
    );
    let report = inspect_candidate_read_model_with_duckdb(&request)
        .map_err(|error| anyhow::anyhow!(error))?;
    emit(&report, cli.output_or_json())
}

fn import_episteme_qianji_review_candidates_command(
    cli: &Cli,
    args: &EpistemeImportQianjiReviewCandidatesArgs,
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
    let mut request =
        EpistemeOntologyQianjiReviewCandidateImportRequest::new(run_root.join(&args.run_id));
    for artifact in &args.qianji_review_artifacts {
        request = request.with_review_artifact(artifact);
    }
    let report = import_episteme_ontology_qianji_review_candidates(&request)?;
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

impl From<EpistemeStructuralFactsValidationModeArg>
    for EpistemeOntologyStructuralFactsValidationMode
{
    fn from(value: EpistemeStructuralFactsValidationModeArg) -> Self {
        match value {
            EpistemeStructuralFactsValidationModeArg::MetadataOnly => Self::MetadataOnly,
            EpistemeStructuralFactsValidationModeArg::FullHash => Self::FullHash,
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
