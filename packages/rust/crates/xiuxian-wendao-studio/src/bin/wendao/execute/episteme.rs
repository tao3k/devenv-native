//! Episteme command execution.

use std::{
    env,
    path::{Path, PathBuf},
};

use crate::bin_support::wendao::cli_support::emit;
use crate::bin_support::wendao::types::{
    Cli, Command, EpistemeCommand, EpistemeEvidenceCommand, EpistemeEvidenceReadValidationModeArg,
    EpistemeEvidenceSelectionValidationModeArg, EpistemePlanExtractionRunArgs,
    EpistemeReadEvidenceArgs, EpistemeSourceContractCommand, EpistemeStructureCommand,
    EpistemeStructureTocValidationModeArg, EpistemeWriteEvidenceSelectionPlanArgs,
    EpistemeWriteStructureTocArgs,
};
use anyhow::{Context, Result};
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
