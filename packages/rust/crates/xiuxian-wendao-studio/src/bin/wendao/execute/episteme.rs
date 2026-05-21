//! Episteme command execution.

use std::{
    env,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
};

use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::{
    Cli, Command, EpistemeCommand, EpistemeEvidenceCommand, EpistemeEvidenceReadValidationModeArg,
    EpistemeEvidenceSelectionValidationModeArg, EpistemePlanExtractionRunArgs,
    EpistemeReadEvidenceArgs, EpistemeRunDoclingDocumentCacheArgs, EpistemeRunImageOcrCacheArgs,
    EpistemeSourceContractCommand, EpistemeStructureCommand, EpistemeStructureTocValidationModeArg,
    EpistemeWriteEvidenceSelectionPlanArgs, EpistemeWriteStructureTocArgs,
};
use anyhow::{Context, Result};
use serde::Serialize;
use xiuxian_git_repo::SyncMode;
use xiuxian_wendao::episteme::{
    EpistemeEvidenceReadRequest, EpistemeEvidenceReadValidationMode,
    EpistemeEvidenceSelectionPlanRequest, EpistemeEvidenceSelectionValidationMode,
    EpistemeRegistryEntry, EpistemeRunPlanRequest, EpistemeRuntimeConfig,
    EpistemeStructureTocRequest, EpistemeStructureTocValidationMode,
    configured_episteme_corpus_root_env, load_episteme_registry_entries_with_mode,
    load_episteme_runtime_config, read_episteme_evidence,
    read_episteme_evidence_selection_file_ids, validate_episteme_registry_reference_graph,
    write_episteme_evidence_selection_plan, write_episteme_extraction_run_plan,
    write_episteme_structure_toc,
};
use xiuxian_wendao_episteme::{
    EPISTEME_DOCLING_DOCUMENT_RESULTS_JSONL, EPISTEME_DOCLING_DOCUMENT_ROUTE,
    EPISTEME_DOCLING_DOCUMENT_WRAPPER_SCHEMA, EPISTEME_IMAGE_OCR_RESULTS_JSONL,
    EPISTEME_IMAGE_OCR_ROUTE, EPISTEME_IMAGE_OCR_WRAPPER_SCHEMA,
    EpistemeDoclingDocumentCacheBridgeReport, EpistemeImageOcrCacheBridgeReport,
    read_docling_document_tasks_tsv, read_image_ocr_tasks_tsv,
    skipped_docling_document_cache_bridge_report, skipped_image_ocr_cache_bridge_report,
    validate_docling_document_tasks, validate_image_ocr_tasks,
    write_docling_document_cache_outputs, write_image_ocr_cache_outputs,
};

use crate::studio::router::{
    load_episteme_registry_from_wendao_toml, load_episteme_registry_from_wendao_toml_path,
};

pub(super) fn handle(cli: &Cli) -> Result<()> {
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
        },
        EpistemeCommand::Structure { command } => match command {
            EpistemeStructureCommand::WriteToc(args) => {
                write_episteme_structure_toc_command(cli, args)
            }
        },
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeExternalCommandSpec {
    program: String,
    args: Vec<String>,
    current_dir: Option<String>,
}

fn image_ocr_analyzer_command_spec(
    analyzer_command: &str,
    episteme_root: &Path,
    tasks_path: &Path,
    corpus_root: &Path,
    ocr_results_jsonl: &Path,
) -> EpistemeExternalCommandSpec {
    EpistemeExternalCommandSpec {
        program: analyzer_command.to_string(),
        args: vec![
            "--tasks".to_string(),
            path_display(tasks_path),
            "--corpus-root".to_string(),
            path_display(corpus_root),
            "--output-jsonl".to_string(),
            path_display(ocr_results_jsonl),
        ],
        current_dir: Some(path_display(episteme_root)),
    }
}

fn docling_document_analyzer_command_spec(
    analyzer_command: &str,
    episteme_root: &Path,
    tasks_path: &Path,
    corpus_root: &Path,
    document_results_jsonl: &Path,
    docling_profile: &str,
) -> EpistemeExternalCommandSpec {
    EpistemeExternalCommandSpec {
        program: analyzer_command.to_string(),
        args: vec![
            "--tasks".to_string(),
            path_display(tasks_path),
            "--corpus-root".to_string(),
            path_display(corpus_root),
            "--output-jsonl".to_string(),
            path_display(document_results_jsonl),
            "--profile".to_string(),
            docling_profile.to_string(),
        ],
        current_dir: Some(path_display(episteme_root)),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct EpistemeExternalCommandReport {
    command: EpistemeExternalCommandSpec,
    skipped: bool,
    exit_code: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EpistemeImageOcrCacheExecutionReport {
    schema_version: &'static str,
    run_id: String,
    route: &'static str,
    dry_run: bool,
    tasks_tsv: String,
    ocr_results_jsonl: String,
    raw_to_rdf_promotion_allowed: bool,
    plan: serde_json::Value,
    analyzer: EpistemeExternalCommandReport,
    cache_bridge: EpistemeImageOcrCacheBridgeReport,
}

struct EpistemeImageOcrCacheRunContext {
    episteme_root: PathBuf,
    corpus_root: PathBuf,
    run_root: PathBuf,
    request: EpistemeRunPlanRequest,
}

struct EpistemeImageOcrCacheCommands {
    tasks_tsv: PathBuf,
    ocr_results_jsonl: PathBuf,
    analyzer: EpistemeExternalCommandSpec,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EpistemeDoclingDocumentCacheExecutionReport {
    schema_version: &'static str,
    run_id: String,
    route: &'static str,
    dry_run: bool,
    tasks_tsv: String,
    document_results_jsonl: String,
    raw_to_rdf_promotion_allowed: bool,
    plan: serde_json::Value,
    analyzer: EpistemeExternalCommandReport,
    cache_bridge: EpistemeDoclingDocumentCacheBridgeReport,
}

struct EpistemeDoclingDocumentCacheRunContext {
    episteme_root: PathBuf,
    corpus_root: PathBuf,
    run_root: PathBuf,
    request: EpistemeRunPlanRequest,
}

struct EpistemeDoclingDocumentCacheCommands {
    tasks_tsv: PathBuf,
    document_results_jsonl: PathBuf,
    analyzer: EpistemeExternalCommandSpec,
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

fn run_episteme_image_ocr_cache(cli: &Cli, args: &EpistemeRunImageOcrCacheArgs) -> Result<()> {
    let context = image_ocr_cache_run_context(cli, args)?;
    let plan = write_episteme_extraction_run_plan(&context.request, &context.run_root)?;
    let planned_tasks = read_image_ocr_tasks_tsv(&plan.tasks_path)?;
    validate_image_ocr_tasks(&planned_tasks)?;
    let commands = image_ocr_cache_commands(args, &context)?;
    let analyzer_exit_code =
        run_external_command_if_needed(args.dry_run, &commands.analyzer, "image OCR analyzer")?;
    let run_dir = context.run_root.join(&args.run_id);
    let outputs_dir = run_dir.join("outputs");
    let cache_receipt_path = run_dir.join("image_ocr_cache_receipt.json");
    let cache_bridge = if args.dry_run {
        skipped_image_ocr_cache_bridge_report(
            &commands.ocr_results_jsonl,
            &outputs_dir,
            &cache_receipt_path,
        )
    } else {
        write_image_ocr_cache_outputs(
            &planned_tasks,
            &commands.ocr_results_jsonl,
            &run_dir,
            &context.corpus_root,
        )?
    };
    let cache_failed_count = cache_bridge.failed_count;
    let report = EpistemeImageOcrCacheExecutionReport {
        schema_version: EPISTEME_IMAGE_OCR_WRAPPER_SCHEMA,
        run_id: args.run_id.clone(),
        route: EPISTEME_IMAGE_OCR_ROUTE,
        dry_run: args.dry_run,
        tasks_tsv: path_display(&commands.tasks_tsv),
        ocr_results_jsonl: path_display(&commands.ocr_results_jsonl),
        raw_to_rdf_promotion_allowed: false,
        plan: serde_json::to_value(&plan).context("failed to serialize image OCR run plan")?,
        analyzer: EpistemeExternalCommandReport {
            command: commands.analyzer,
            skipped: args.dry_run,
            exit_code: analyzer_exit_code,
        },
        cache_bridge,
    };
    emit(&report, cli.output_or_json())?;
    if cache_failed_count > 0 {
        anyhow::bail!("image OCR cache bridge wrote {cache_failed_count} failed rows");
    }
    Ok(())
}

fn run_episteme_docling_document_cache(
    cli: &Cli,
    args: &EpistemeRunDoclingDocumentCacheArgs,
) -> Result<()> {
    let context = docling_document_cache_run_context(cli, args)?;
    let plan = write_episteme_extraction_run_plan(&context.request, &context.run_root)?;
    let planned_tasks = read_docling_document_tasks_tsv(&plan.tasks_path)?;
    validate_docling_document_tasks(&planned_tasks)?;
    let commands = docling_document_cache_commands(args, &context)?;
    let analyzer_exit_code = run_external_command_if_needed(
        args.dry_run,
        &commands.analyzer,
        "Docling document analyzer",
    )?;
    let run_dir = context.run_root.join(&args.run_id);
    let outputs_dir = run_dir.join("outputs");
    let cache_receipt_path = run_dir.join("document_cache_receipt.json");
    let cache_bridge = if args.dry_run {
        skipped_docling_document_cache_bridge_report(
            &commands.document_results_jsonl,
            &outputs_dir,
            &cache_receipt_path,
        )
    } else {
        write_docling_document_cache_outputs(
            &planned_tasks,
            &commands.document_results_jsonl,
            &run_dir,
        )?
    };
    let cache_failed_count = cache_bridge.failed_count;
    let report = EpistemeDoclingDocumentCacheExecutionReport {
        schema_version: EPISTEME_DOCLING_DOCUMENT_WRAPPER_SCHEMA,
        run_id: args.run_id.clone(),
        route: EPISTEME_DOCLING_DOCUMENT_ROUTE,
        dry_run: args.dry_run,
        tasks_tsv: path_display(&commands.tasks_tsv),
        document_results_jsonl: path_display(&commands.document_results_jsonl),
        raw_to_rdf_promotion_allowed: false,
        plan: serde_json::to_value(&plan)
            .context("failed to serialize Docling document run plan")?,
        analyzer: EpistemeExternalCommandReport {
            command: commands.analyzer,
            skipped: args.dry_run,
            exit_code: analyzer_exit_code,
        },
        cache_bridge,
    };
    emit(&report, cli.output_or_json())?;
    if cache_failed_count > 0 {
        anyhow::bail!("Docling document cache bridge wrote {cache_failed_count} failed rows");
    }
    Ok(())
}

fn image_ocr_cache_run_context(
    cli: &Cli,
    args: &EpistemeRunImageOcrCacheArgs,
) -> Result<EpistemeImageOcrCacheRunContext> {
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
    let mut request = EpistemeRunPlanRequest::new(
        episteme_root.clone(),
        corpus_root.clone(),
        args.run_id.clone(),
    )
    .with_limit(args.limit)
    .with_route(EPISTEME_IMAGE_OCR_ROUTE);
    if let Some(category) = &args.category {
        request = request.with_category(category.as_str());
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
    Ok(EpistemeImageOcrCacheRunContext {
        episteme_root,
        corpus_root,
        run_root,
        request,
    })
}

fn image_ocr_cache_commands(
    args: &EpistemeRunImageOcrCacheArgs,
    context: &EpistemeImageOcrCacheRunContext,
) -> Result<EpistemeImageOcrCacheCommands> {
    let run_dir = context.run_root.join(&args.run_id);
    let tasks_path = run_dir.join("tasks.tsv");
    let ocr_results_jsonl = args
        .ocr_results_jsonl
        .clone()
        .unwrap_or_else(|| run_dir.join(EPISTEME_IMAGE_OCR_RESULTS_JSONL));
    let episteme_root_for_command = absolute_runtime_path(&context.episteme_root)?;
    let tasks_path_for_command = absolute_runtime_path(&tasks_path)?;
    let corpus_root_for_command = absolute_runtime_path(&context.corpus_root)?;
    let ocr_results_jsonl_for_command = absolute_runtime_path(&ocr_results_jsonl)?;
    let analyzer_command = image_ocr_analyzer_command_spec(
        &args.analyzer_command,
        &episteme_root_for_command,
        &tasks_path_for_command,
        &corpus_root_for_command,
        &ocr_results_jsonl_for_command,
    );
    Ok(EpistemeImageOcrCacheCommands {
        tasks_tsv: tasks_path_for_command,
        ocr_results_jsonl: ocr_results_jsonl_for_command,
        analyzer: analyzer_command,
    })
}

fn docling_document_cache_run_context(
    cli: &Cli,
    args: &EpistemeRunDoclingDocumentCacheArgs,
) -> Result<EpistemeDoclingDocumentCacheRunContext> {
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
    let mut request = EpistemeRunPlanRequest::new(
        episteme_root.clone(),
        corpus_root.clone(),
        args.run_id.clone(),
    )
    .with_limit(args.limit)
    .with_route(EPISTEME_DOCLING_DOCUMENT_ROUTE);
    if let Some(category) = &args.category {
        request = request.with_category(category.as_str());
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
    Ok(EpistemeDoclingDocumentCacheRunContext {
        episteme_root,
        corpus_root,
        run_root,
        request,
    })
}

fn docling_document_cache_commands(
    args: &EpistemeRunDoclingDocumentCacheArgs,
    context: &EpistemeDoclingDocumentCacheRunContext,
) -> Result<EpistemeDoclingDocumentCacheCommands> {
    let run_dir = context.run_root.join(&args.run_id);
    let tasks_path = run_dir.join("tasks.tsv");
    let document_results_jsonl = args
        .document_results_jsonl
        .clone()
        .unwrap_or_else(|| run_dir.join(EPISTEME_DOCLING_DOCUMENT_RESULTS_JSONL));
    let episteme_root_for_command = absolute_runtime_path(&context.episteme_root)?;
    let tasks_path_for_command = absolute_runtime_path(&tasks_path)?;
    let corpus_root_for_command = absolute_runtime_path(&context.corpus_root)?;
    let document_results_jsonl_for_command = absolute_runtime_path(&document_results_jsonl)?;
    let analyzer_command = docling_document_analyzer_command_spec(
        &args.analyzer_command,
        &episteme_root_for_command,
        &tasks_path_for_command,
        &corpus_root_for_command,
        &document_results_jsonl_for_command,
        &args.docling_profile,
    );
    Ok(EpistemeDoclingDocumentCacheCommands {
        tasks_tsv: tasks_path_for_command,
        document_results_jsonl: document_results_jsonl_for_command,
        analyzer: analyzer_command,
    })
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

fn resolve_episteme_root(
    cli: &Cli,
    episteme_root: &Path,
    episteme_registry_id: Option<&str>,
) -> Result<PathBuf> {
    let Some(registry_id) = episteme_registry_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(episteme_root.to_path_buf());
    };
    let entries = load_episteme_registry_entries(cli)?;
    let entry = entries
        .iter()
        .find(|entry| entry.id == registry_id)
        .with_context(|| format!("episteme registry `{registry_id}` is not configured"))?;
    if !entry.enabled {
        anyhow::bail!("episteme registry `{registry_id}` is disabled");
    }
    let receipt = load_episteme_registry_entries_with_mode(&entries, &cli.root, SyncMode::Ensure)
        .with_context(|| format!("failed to load episteme registry `{registry_id}`"))?;
    validate_episteme_registry_reference_graph(&receipt)
        .with_context(|| format!("failed to validate episteme registry `{registry_id}` graph"))?;
    receipt
        .entries
        .into_iter()
        .find(|entry| entry.id == registry_id)
        .map(|entry| entry.episteme_root)
        .with_context(|| format!("episteme registry `{registry_id}` did not load an episteme root"))
}

fn load_episteme_registry_entries(cli: &Cli) -> Result<Vec<EpistemeRegistryEntry>> {
    if let Some(config_file) = &cli.config_file {
        return load_episteme_registry_from_wendao_toml_path(config_file.as_path()).map_err(
            |error| {
                anyhow::anyhow!(
                    "failed to load episteme registry from `{}`: {error}",
                    config_file.display()
                )
            },
        );
    }
    load_episteme_registry_from_wendao_toml(cli.root.as_path()).map_err(|error| {
        anyhow::anyhow!(
            "failed to load episteme registry from `{}`: {error}",
            cli.root.display()
        )
    })
}

fn load_runtime_config(episteme_root: &Path) -> Result<Option<EpistemeRuntimeConfig>> {
    load_episteme_runtime_config(episteme_root).with_context(|| {
        format!(
            "failed to load episteme runtime config from `{}`",
            episteme_root.join("episteme.toml").display()
        )
    })
}

fn resolve_corpus_root(
    corpus_root: Option<&PathBuf>,
    episteme_root: &Path,
    config: Option<&EpistemeRuntimeConfig>,
) -> Result<PathBuf> {
    if let Some(path) = corpus_root {
        return Ok(path.clone());
    }
    if let Some(path) = config.and_then(|config| config.corpus.clone()) {
        return Ok(path);
    }
    let corpus_root_env = configured_episteme_corpus_root_env(episteme_root)
        .context("failed to read episteme-configured corpus root env")?;
    env::var_os(corpus_root_env.as_str())
        .map(PathBuf::from)
        .with_context(|| {
            format!(
                "--corpus-root is required when episteme.toml has no runtime.corpus_root and {corpus_root_env} is not set"
            )
        })
}

fn resolve_run_root(
    explicit: Option<&PathBuf>,
    configured: Option<&PathBuf>,
    fallback: impl FnOnce() -> PathBuf,
) -> PathBuf {
    explicit
        .cloned()
        .or_else(|| configured.cloned())
        .unwrap_or_else(fallback)
}

fn run_external_command_if_needed(
    dry_run: bool,
    spec: &EpistemeExternalCommandSpec,
    label: &str,
) -> Result<Option<i32>> {
    if dry_run {
        return Ok(None);
    }
    run_external_command(spec, label).map(Some)
}

fn run_external_command(spec: &EpistemeExternalCommandSpec, label: &str) -> Result<i32> {
    let mut command = ProcessCommand::new(&spec.program);
    command.args(&spec.args);
    if let Some(current_dir) = &spec.current_dir {
        command.current_dir(current_dir);
    }
    let status = command
        .status()
        .with_context(|| format!("failed to start {label} command `{}`", spec.program))?;
    let exit_code = status.code().unwrap_or(1);
    if !status.success() {
        anyhow::bail!("{label} command failed with exit code {exit_code}");
    }
    Ok(exit_code)
}

fn absolute_runtime_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    env::current_dir()
        .map(|current_dir| current_dir.join(path))
        .context("failed to resolve current directory for image OCR command paths")
}

fn path_display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
#[path = "../../../../tests/unit/bin/wendao/execute/episteme/mod.rs"]
mod tests;
