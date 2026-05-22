use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Serialize;
use xiuxian_wendao::episteme::{
    EpistemeRunPlanRequest, read_episteme_evidence_selection_file_ids,
    write_episteme_extraction_run_plan,
};
use xiuxian_wendao_episteme::{
    EPISTEME_DOCLING_DOCUMENT_RESULTS_JSONL, EPISTEME_DOCLING_DOCUMENT_ROUTE,
    EPISTEME_DOCLING_DOCUMENT_WRAPPER_SCHEMA, EPISTEME_IMAGE_OCR_RESULTS_JSONL,
    EPISTEME_IMAGE_OCR_ROUTE, EPISTEME_IMAGE_OCR_WRAPPER_SCHEMA,
    EPISTEME_LEGACY_OFFICE_CONVERSION_WRAPPER_SCHEMA, EPISTEME_LEGACY_OFFICE_DOCUMENT_ROUTE,
    EpistemeDoclingDocumentCacheBridgeReport, EpistemeImageOcrCacheBridgeReport,
    EpistemeLegacyOfficeConversionReport, EpistemeLegacyOfficeConversionRequest,
    convert_legacy_office_tasks, read_docling_document_tasks_tsv, read_image_ocr_tasks_tsv,
    read_legacy_office_conversion_tasks_tsv, skipped_docling_document_cache_bridge_report,
    skipped_image_ocr_cache_bridge_report, validate_docling_document_tasks,
    validate_image_ocr_tasks, validate_legacy_office_conversion_tasks,
    write_docling_document_cache_outputs, write_image_ocr_cache_outputs,
};

use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::{
    Cli, EpistemeRunDoclingDocumentCacheArgs, EpistemeRunImageOcrCacheArgs,
    EpistemeRunLegacyOfficeConversionArgs,
};

use super::external::{
    EpistemeExternalCommandReport, EpistemeExternalCommandSpec,
    docling_document_analyzer_command_spec, image_ocr_analyzer_command_spec,
    run_external_command_if_needed, should_skip_analyzer,
};
use super::root::{
    absolute_runtime_path, load_runtime_config, path_display, resolve_corpus_root,
    resolve_episteme_root, resolve_legacy_office_converter, resolve_run_root,
};

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EpistemeLegacyOfficeConversionExecutionReport {
    schema_version: &'static str,
    run_id: String,
    route: &'static str,
    dry_run: bool,
    tasks_tsv: String,
    raw_to_rdf_promotion_allowed: bool,
    plan: serde_json::Value,
    conversion: EpistemeLegacyOfficeConversionReport,
}

struct EpistemeLegacyOfficeConversionRunContext {
    corpus_root: PathBuf,
    run_root: PathBuf,
    request: EpistemeRunPlanRequest,
    converter_path: PathBuf,
}

pub(super) fn run_episteme_image_ocr_cache(
    cli: &Cli,
    args: &EpistemeRunImageOcrCacheArgs,
) -> Result<()> {
    let context = image_ocr_cache_run_context(cli, args)?;
    let plan = write_episteme_extraction_run_plan(&context.request, &context.run_root)?;
    let planned_tasks = read_image_ocr_tasks_tsv(&plan.tasks_path)?;
    validate_image_ocr_tasks(&planned_tasks)?;
    let commands = image_ocr_cache_commands(args, &context)?;
    let analyzer_skipped = should_skip_analyzer(args.dry_run, args.use_existing_results);
    let analyzer_exit_code =
        run_external_command_if_needed(analyzer_skipped, &commands.analyzer, "image OCR analyzer")?;
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
            skipped: analyzer_skipped,
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

pub(super) fn run_episteme_docling_document_cache(
    cli: &Cli,
    args: &EpistemeRunDoclingDocumentCacheArgs,
) -> Result<()> {
    let context = docling_document_cache_run_context(cli, args)?;
    let plan = write_episteme_extraction_run_plan(&context.request, &context.run_root)?;
    let planned_tasks = read_docling_document_tasks_tsv(&plan.tasks_path)?;
    validate_docling_document_tasks(&planned_tasks)?;
    let commands = docling_document_cache_commands(args, &context)?;
    let analyzer_skipped = should_skip_analyzer(args.dry_run, args.use_existing_results);
    let analyzer_exit_code = run_external_command_if_needed(
        analyzer_skipped,
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
            &context.corpus_root,
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
            skipped: analyzer_skipped,
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

pub(super) fn run_episteme_legacy_office_conversion(
    cli: &Cli,
    args: &EpistemeRunLegacyOfficeConversionArgs,
) -> Result<()> {
    let context = legacy_office_conversion_run_context(cli, args)?;
    let plan = write_episteme_extraction_run_plan(&context.request, &context.run_root)?;
    let planned_tasks = read_legacy_office_conversion_tasks_tsv(&plan.tasks_path)?;
    validate_legacy_office_conversion_tasks(&planned_tasks)?;
    let run_dir = context.run_root.join(&args.run_id);
    let request = EpistemeLegacyOfficeConversionRequest::new(context.converter_path)
        .with_dry_run(args.dry_run);
    let conversion =
        convert_legacy_office_tasks(&planned_tasks, &run_dir, &context.corpus_root, &request)?;
    let failed_count = conversion.failed_count;
    let report = EpistemeLegacyOfficeConversionExecutionReport {
        schema_version: EPISTEME_LEGACY_OFFICE_CONVERSION_WRAPPER_SCHEMA,
        run_id: args.run_id.clone(),
        route: EPISTEME_LEGACY_OFFICE_DOCUMENT_ROUTE,
        dry_run: args.dry_run,
        tasks_tsv: path_display(&plan.tasks_path),
        raw_to_rdf_promotion_allowed: false,
        plan: serde_json::to_value(&plan)
            .context("failed to serialize legacy Office conversion run plan")?,
        conversion,
    };
    emit(&report, cli.output_or_json())?;
    if failed_count > 0 {
        anyhow::bail!("legacy Office conversion wrote {failed_count} failed rows");
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

fn legacy_office_conversion_run_context(
    cli: &Cli,
    args: &EpistemeRunLegacyOfficeConversionArgs,
) -> Result<EpistemeLegacyOfficeConversionRunContext> {
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
    let converter_path = resolve_legacy_office_converter(
        args.converter_command.as_ref(),
        config.as_ref(),
        args.dry_run,
    )?;
    let mut request = EpistemeRunPlanRequest::new(
        episteme_root.clone(),
        corpus_root.clone(),
        args.run_id.clone(),
    )
    .with_limit(args.limit)
    .with_route(EPISTEME_LEGACY_OFFICE_DOCUMENT_ROUTE);
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
    Ok(EpistemeLegacyOfficeConversionRunContext {
        corpus_root,
        run_root,
        request,
        converter_path,
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
