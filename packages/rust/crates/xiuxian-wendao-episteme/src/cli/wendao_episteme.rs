//! Crate-owned operator CLI for deterministic Episteme workflows.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::ArtifactBlobCacheBackendConfig;
#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_wendao_episteme::{
    EpistemeOntologyBootstrapArtifactCacheOptions,
    EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome,
    EpistemeOntologyBootstrapArtifactCacheReadThroughReport,
    EpistemeOntologyBootstrapArtifactCacheReport,
    EpistemeOntologyBootstrapArtifactCacheRestoreReport,
    EpistemeOntologyBootstrapArtifactCacheStage,
    admit_episteme_ontology_bootstrap_artifact_cache_options,
    read_through_episteme_ontology_bootstrap_artifacts,
    restore_episteme_ontology_bootstrap_pipeline_artifacts,
    run_episteme_ontology_bootstrap_pipeline_with_artifact_cache,
};
use xiuxian_wendao_episteme::{
    EpistemeOntologyBootstrapPipelineRequest,
    EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest,
    EpistemeOntologyStructuralFactsValidationMode, run_episteme_ontology_bootstrap_pipeline,
    write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan,
};

const DEFAULT_OPENAI_COMPATIBLE_MODEL: &str = "deepseek/deepseek-v4-pro";
const DEFAULT_OPENAI_COMPATIBLE_MAX_TOKENS: u32 = 8_192;

/// Run the `wendao-episteme` CLI from process arguments.
pub fn run_from_env() -> Result<()> {
    let cli = Cli::parse();
    let report = run_cli(cli)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn run_cli(cli: Cli) -> Result<serde_json::Value> {
    match cli.command {
        Command::Ontology(args) => run_ontology_command(args.command),
    }
}

fn run_ontology_command(command: OntologyCommand) -> Result<serde_json::Value> {
    match command {
        OntologyCommand::BootstrapPipeline(args) => run_bootstrap_pipeline_command(&args),
        OntologyCommand::QianjiSchedulePlan(args) => {
            let run_root = args.run_root()?;
            let report = write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan(
                &args.into_request(),
                run_root,
            )?;
            Ok(serde_json::to_value(report)?)
        }
    }
}

fn run_bootstrap_pipeline_command(args: &BootstrapPipelineArgs) -> Result<serde_json::Value> {
    #[cfg(feature = "foyer-artifact-cache")]
    {
        if args.artifact_cache_mode != BootstrapArtifactCacheModeArg::Disabled {
            return run_bootstrap_pipeline_artifact_cache_command(args);
        }
    }
    let report = run_episteme_ontology_bootstrap_pipeline(&args.to_request())?;
    Ok(serde_json::to_value(report)?)
}

#[cfg(feature = "foyer-artifact-cache")]
fn run_bootstrap_pipeline_artifact_cache_command(
    args: &BootstrapPipelineArgs,
) -> Result<serde_json::Value> {
    let request = args.to_request();
    let options = args.artifact_cache_options()?;
    let config = ArtifactBlobCacheBackendConfig::from_env()
        .context("failed to resolve artifact cache backend config")?;
    let cache = config
        .build()
        .context("failed to build artifact cache backend")?;
    match args.artifact_cache_mode {
        BootstrapArtifactCacheModeArg::Disabled => {
            let report = run_episteme_ontology_bootstrap_pipeline(&request)?;
            Ok(serde_json::to_value(report)?)
        }
        BootstrapArtifactCacheModeArg::WriteThrough => {
            let report = run_episteme_ontology_bootstrap_pipeline_with_artifact_cache(
                &request, &cache, &options,
            )?;
            Ok(bootstrap_artifact_report_json(
                "write-through",
                report,
                cache.backend_name(),
                config.root(),
            ))
        }
        BootstrapArtifactCacheModeArg::ReadThrough => {
            let report =
                read_through_episteme_ontology_bootstrap_artifacts(&request, &cache, &options)?;
            Ok(bootstrap_readthrough_report_json(
                report,
                cache.backend_name(),
                config.root(),
            ))
        }
        BootstrapArtifactCacheModeArg::RestoreOnly => {
            let report =
                restore_episteme_ontology_bootstrap_pipeline_artifacts(&request, &cache, &options)?;
            Ok(bootstrap_restore_report_json(
                "restore-only",
                &report,
                cache.backend_name(),
                config.root(),
            ))
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "wendao-episteme",
    about = "Run deterministic Episteme source-contract workflows"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run ontology source-contract workflows.
    Ontology(OntologyArgs),
}

#[derive(Debug, Args)]
struct OntologyArgs {
    #[command(subcommand)]
    command: OntologyCommand,
}

#[derive(Debug, Subcommand)]
enum OntologyCommand {
    /// Compile deterministic structure, reasoning packet, ledger seed, and fill plan artifacts.
    BootstrapPipeline(BootstrapPipelineArgs),
    /// Compile a reasoning fill plan into Qianji activity schedule inputs.
    QianjiSchedulePlan(QianjiSchedulePlanArgs),
}

#[derive(Debug, Args)]
struct BootstrapPipelineArgs {
    /// Episteme repository root containing episteme.toml and ontology/.
    #[arg(long)]
    episteme_root: PathBuf,
    /// Stable run id used to derive deterministic stage run ids.
    #[arg(long)]
    run_id: String,
    /// Optional raw corpus root override. Defaults to episteme.toml runtime config.
    #[arg(long)]
    corpus_root: Option<PathBuf>,
    /// Optional structural run root override. Defaults to episteme.toml runtime config.
    #[arg(long)]
    structure_run_root: Option<PathBuf>,
    /// Optional ontology-generation run root override. Defaults to episteme.toml runtime config.
    #[arg(long)]
    ontology_generation_run_root: Option<PathBuf>,
    /// Structural facts validation mode.
    #[arg(long, value_enum, default_value_t = ValidationModeArg::MetadataOnly)]
    validation_mode: ValidationModeArg,
    /// Optional reasoning packet source category filter.
    #[arg(long)]
    category: Option<String>,
    /// Optional reasoning packet extraction route filter.
    #[arg(long)]
    route: Option<String>,
    /// Maximum reasoning packet rows.
    #[arg(long, default_value_t = 256)]
    reasoning_packet_limit: usize,
    /// Maximum reasoning ledger seed rows.
    #[arg(long, default_value_t = 512)]
    reasoning_ledger_seed_limit: usize,
    /// Maximum reasoning fill-plan rows.
    #[arg(long, default_value_t = 1024)]
    reasoning_fill_plan_limit: usize,
    /// Artifact cache mode for generated bootstrap run directories.
    #[cfg(feature = "foyer-artifact-cache")]
    #[arg(long, value_enum, default_value_t = BootstrapArtifactCacheModeArg::Disabled)]
    artifact_cache_mode: BootstrapArtifactCacheModeArg,
    /// Source digest component for artifact-cache identities.
    #[cfg(feature = "foyer-artifact-cache")]
    #[arg(long)]
    artifact_cache_source_digest: Option<String>,
    /// Profile digest component for artifact-cache identities.
    #[cfg(feature = "foyer-artifact-cache")]
    #[arg(long)]
    artifact_cache_profile_digest: Option<String>,
}

#[derive(Debug, Args)]
struct QianjiSchedulePlanArgs {
    /// Reasoning fill-plan JSON produced by the bootstrap pipeline.
    #[arg(long)]
    reasoning_fill_plan_json: PathBuf,
    /// Stable schedule-plan run id.
    #[arg(long)]
    run_id: String,
    /// Optional Qianji run id carried by generated activity tasks.
    #[arg(long)]
    qianji_run_id: Option<String>,
    /// Optional run artifact root. Defaults to the fill-plan run directory parent.
    #[arg(long)]
    run_root: Option<PathBuf>,
    /// Maximum schedule rows to emit.
    #[arg(long, default_value_t = 1024)]
    limit: usize,
    /// Restrict scheduling to fill-plan rows with this target ledger field group.
    #[arg(long)]
    target_ledger_field_group: Option<String>,
    /// Restrict scheduling to fill-plan rows with this evidence target intent.
    #[arg(long)]
    evidence_target_intent: Option<String>,
    /// Deterministic reasoning context shard mode for prompt-audit tasks.
    #[arg(long, default_value = "disabled")]
    reasoning_context_shard_mode: String,
    /// Maximum table rows per reasoning context shard.
    #[arg(long, default_value_t = 2)]
    reasoning_context_shard_row_limit: usize,
    /// Optional extraction-run root used to materialize context evidence.
    #[arg(long)]
    evidence_extraction_run_root: Option<PathBuf>,
    /// Extraction run id to include in prompt-audit context evidence. Repeatable.
    #[arg(long)]
    evidence_extraction_run_id: Vec<String>,
    /// Emit OpenAI-compatible prompt audit metadata without executing the model.
    #[arg(long)]
    openai_compatible_prompt_audit: bool,
    /// OpenAI-compatible model id for prompt-audit metadata.
    #[arg(long, default_value = DEFAULT_OPENAI_COMPATIBLE_MODEL)]
    openai_compatible_model: String,
    /// OpenAI-compatible max token budget for prompt-audit metadata.
    #[arg(long, default_value_t = DEFAULT_OPENAI_COMPATIBLE_MAX_TOKENS)]
    openai_compatible_max_tokens: u32,
}

impl QianjiSchedulePlanArgs {
    fn run_root(&self) -> Result<PathBuf> {
        if let Some(run_root) = &self.run_root {
            return Ok(run_root.clone());
        }
        self.reasoning_fill_plan_json
            .parent()
            .and_then(|path| path.parent())
            .map(PathBuf::from)
            .with_context(|| {
                format!(
                    "cannot infer Qianji schedule run root from `{}`; pass --run-root",
                    self.reasoning_fill_plan_json.display()
                )
            })
    }

    fn into_request(self) -> EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest {
        let mut request = EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest::new(
            self.reasoning_fill_plan_json,
            self.run_id,
        )
        .with_limit(self.limit)
        .with_reasoning_context_shard_mode(self.reasoning_context_shard_mode)
        .with_reasoning_context_shard_row_limit(self.reasoning_context_shard_row_limit);

        if let Some(qianji_run_id) = self.qianji_run_id {
            request = request.with_qianji_run_id(qianji_run_id);
        }
        if let Some(target_ledger_field_group) = self.target_ledger_field_group {
            request = request.with_target_ledger_field_group(target_ledger_field_group);
        }
        if let Some(evidence_target_intent) = self.evidence_target_intent {
            request = request.with_evidence_target_intent(evidence_target_intent);
        }
        if let Some(root) = self.evidence_extraction_run_root {
            request = request.with_evidence_extraction_run_root(root);
        }
        for run_id in self.evidence_extraction_run_id {
            request = request.with_evidence_extraction_run_id(run_id);
        }
        if self.openai_compatible_prompt_audit {
            request = request.with_openai_compatible_prompt_audit(
                self.openai_compatible_model,
                self.openai_compatible_max_tokens,
            );
        }
        request
    }
}

impl BootstrapPipelineArgs {
    fn to_request(&self) -> EpistemeOntologyBootstrapPipelineRequest {
        let mut request = EpistemeOntologyBootstrapPipelineRequest::new(
            self.episteme_root.clone(),
            self.run_id.clone(),
        )
        .with_validation_mode(self.validation_mode.into())
        .with_reasoning_packet_limit(self.reasoning_packet_limit)
        .with_reasoning_ledger_seed_limit(self.reasoning_ledger_seed_limit)
        .with_reasoning_fill_plan_limit(self.reasoning_fill_plan_limit);

        if let Some(corpus_root) = &self.corpus_root {
            request = request.with_corpus_root(corpus_root.clone());
        }
        if let Some(run_root) = &self.structure_run_root {
            request = request.with_structure_run_root(run_root.clone());
        }
        if let Some(run_root) = &self.ontology_generation_run_root {
            request = request.with_ontology_generation_run_root(run_root.clone());
        }
        if let Some(category) = &self.category {
            request = request.with_category(category.clone());
        }
        if let Some(route) = &self.route {
            request = request.with_route(route.clone());
        }
        request
    }

    #[cfg(feature = "foyer-artifact-cache")]
    fn artifact_cache_options(&self) -> Result<EpistemeOntologyBootstrapArtifactCacheOptions> {
        let source_digest = self
            .artifact_cache_source_digest
            .clone()
            .filter(|value| !value.trim().is_empty())
            .context(
                "--artifact-cache-source-digest is required when artifact cache mode is enabled",
            )?;
        let profile_digest = self
            .artifact_cache_profile_digest
            .clone()
            .filter(|value| !value.trim().is_empty())
            .context(
                "--artifact-cache-profile-digest is required when artifact cache mode is enabled",
            )?;
        admit_episteme_ontology_bootstrap_artifact_cache_options(source_digest, profile_digest)
    }
}

#[cfg(feature = "foyer-artifact-cache")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum BootstrapArtifactCacheModeArg {
    /// Run the deterministic bootstrap pipeline without artifact-cache use.
    #[default]
    Disabled,
    /// Run the pipeline and write generated stage directories to the artifact cache.
    WriteThrough,
    /// Restore all stage directories when cached, otherwise generate and write them.
    ReadThrough,
    /// Restore cached stage directories and report missing stage bundles.
    RestoreOnly,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum ValidationModeArg {
    /// Validate file presence and size without hashing source bytes.
    #[default]
    MetadataOnly,
    /// Validate file presence, size, and SHA-256.
    FullHash,
}

impl From<ValidationModeArg> for EpistemeOntologyStructuralFactsValidationMode {
    fn from(value: ValidationModeArg) -> Self {
        match value {
            ValidationModeArg::MetadataOnly => Self::MetadataOnly,
            ValidationModeArg::FullHash => Self::FullHash,
        }
    }
}

#[cfg(feature = "foyer-artifact-cache")]
fn bootstrap_artifact_report_json(
    mode: &str,
    report: EpistemeOntologyBootstrapArtifactCacheReport,
    backend: &str,
    root: &std::path::Path,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": "xiuxian_wendao.episteme_cli_bootstrap_artifact_cache.v1",
        "mode": mode,
        "backend": backend,
        "root": root,
        "pipeline": report.pipeline,
        "bundles": report.bundles.into_iter().map(|bundle| {
            serde_json::json!({
                "artifactKey": artifact_key_json(&bundle.artifact_key),
                "sourceDir": bundle.source_dir,
                "byteLen": bundle.byte_len,
                "replaced": bundle.replaced,
            })
        }).collect::<Vec<_>>(),
    })
}

#[cfg(feature = "foyer-artifact-cache")]
fn bootstrap_readthrough_report_json(
    report: EpistemeOntologyBootstrapArtifactCacheReadThroughReport,
    backend: &str,
    root: &std::path::Path,
) -> serde_json::Value {
    let outcome = match report.outcome {
        EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome::Restored => "restored",
        EpistemeOntologyBootstrapArtifactCacheReadThroughOutcome::Generated => "generated",
    };
    serde_json::json!({
        "schemaVersion": "xiuxian_wendao.episteme_cli_bootstrap_artifact_cache.v1",
        "mode": "read-through",
        "backend": backend,
        "root": root,
        "outcome": outcome,
        "restore": restore_report_json(&report.restore),
        "generated": report.generated.map(|generated| {
            bootstrap_artifact_report_json("write-through", generated, backend, root)
        }),
    })
}

#[cfg(feature = "foyer-artifact-cache")]
fn bootstrap_restore_report_json(
    mode: &str,
    report: &EpistemeOntologyBootstrapArtifactCacheRestoreReport,
    backend: &str,
    root: &std::path::Path,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": "xiuxian_wendao.episteme_cli_bootstrap_artifact_cache.v1",
        "mode": mode,
        "backend": backend,
        "root": root,
        "restore": restore_report_json(report),
    })
}

#[cfg(feature = "foyer-artifact-cache")]
fn restore_report_json(
    report: &EpistemeOntologyBootstrapArtifactCacheRestoreReport,
) -> serde_json::Value {
    serde_json::json!({
        "complete": report.complete(),
        "restored": report.restored.iter().map(|restored| {
            serde_json::json!({
                "artifactKey": artifact_key_json(&restored.artifact_key),
                "targetDir": restored.target_dir,
                "byteLen": restored.byte_len,
            })
        }).collect::<Vec<_>>(),
        "missing": report.missing.iter().map(|missing| {
            serde_json::json!({
                "stage": bootstrap_stage_name(missing.stage),
                "runDigest": missing.run_digest,
                "targetDir": missing.target_dir,
            })
        }).collect::<Vec<_>>(),
    })
}

#[cfg(feature = "foyer-artifact-cache")]
fn artifact_key_json(key: &xiuxian_db_store::artifact_cache::ArtifactKey) -> serde_json::Value {
    serde_json::json!({
        "namespace": key.namespace().as_str(),
        "kind": key.kind().as_storage_component(),
        "sourceDigest": key.source_digest().as_str(),
        "profileDigest": key.profile_digest().as_str(),
        "shardDigest": key.shard_digest().as_str(),
    })
}

#[cfg(feature = "foyer-artifact-cache")]
const fn bootstrap_stage_name(stage: EpistemeOntologyBootstrapArtifactCacheStage) -> &'static str {
    match stage {
        EpistemeOntologyBootstrapArtifactCacheStage::StructuralFacts => "structural-facts",
        EpistemeOntologyBootstrapArtifactCacheStage::ReasoningPacket => "reasoning-packet",
        EpistemeOntologyBootstrapArtifactCacheStage::ReasoningLedgerSeed => "reasoning-ledger-seed",
        EpistemeOntologyBootstrapArtifactCacheStage::ReasoningFillPlan => "reasoning-fill-plan",
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[cfg(feature = "foyer-artifact-cache")]
    use super::BootstrapArtifactCacheModeArg;
    #[cfg(feature = "foyer-artifact-cache")]
    use super::run_cli;
    use super::{Cli, Command, OntologyCommand, ValidationModeArg};

    #[test]
    fn parses_bootstrap_pipeline_command() {
        let cli = Cli::try_parse_from([
            "wendao-episteme",
            "ontology",
            "bootstrap-pipeline",
            "--episteme-root",
            "private-episteme",
            "--run-id",
            "ltc_bootstrap",
            "--validation-mode",
            "full-hash",
            "--category",
            "policy",
            "--route",
            "document_text_evidence",
            "--reasoning-packet-limit",
            "16",
        ])
        .unwrap_or_else(|error| panic!("parse Episteme bootstrap CLI: {error}"));

        let Command::Ontology(args) = cli.command;
        let OntologyCommand::BootstrapPipeline(args) = args.command else {
            panic!("expected bootstrap pipeline command");
        };
        assert_eq!(args.episteme_root, PathBuf::from("private-episteme"));
        assert_eq!(args.run_id, "ltc_bootstrap");
        assert_eq!(args.validation_mode, ValidationModeArg::FullHash);
        assert_eq!(args.category.as_deref(), Some("policy"));
        assert_eq!(args.route.as_deref(), Some("document_text_evidence"));
        assert_eq!(args.reasoning_packet_limit, 16);
    }

    #[cfg(feature = "foyer-artifact-cache")]
    #[test]
    fn parses_bootstrap_pipeline_artifact_readthrough_command() {
        let cli = Cli::try_parse_from([
            "wendao-episteme",
            "ontology",
            "bootstrap-pipeline",
            "--episteme-root",
            "private-episteme",
            "--run-id",
            "ltc_bootstrap",
            "--artifact-cache-mode",
            "read-through",
            "--artifact-cache-source-digest",
            "source-contract-v1",
            "--artifact-cache-profile-digest",
            "bootstrap-v1",
        ])
        .unwrap_or_else(|error| panic!("parse Episteme artifact-cache bootstrap CLI: {error}"));

        let Command::Ontology(args) = cli.command;
        let OntologyCommand::BootstrapPipeline(args) = args.command else {
            panic!("expected bootstrap pipeline command");
        };
        assert_eq!(
            args.artifact_cache_mode,
            BootstrapArtifactCacheModeArg::ReadThrough
        );
        assert_eq!(
            args.artifact_cache_source_digest.as_deref(),
            Some("source-contract-v1")
        );
        assert_eq!(
            args.artifact_cache_profile_digest.as_deref(),
            Some("bootstrap-v1")
        );
    }

    #[cfg(feature = "foyer-artifact-cache")]
    #[test]
    fn bootstrap_pipeline_artifact_cache_mode_requires_digests() {
        let cli = Cli::try_parse_from([
            "wendao-episteme",
            "ontology",
            "bootstrap-pipeline",
            "--episteme-root",
            "private-episteme",
            "--run-id",
            "ltc_bootstrap",
            "--artifact-cache-mode",
            "restore-only",
        ])
        .unwrap_or_else(|error| panic!("parse Episteme artifact-cache bootstrap CLI: {error}"));

        let Err(error) = run_cli(cli) else {
            panic!("artifact-cache mode should require digests");
        };

        assert!(
            error
                .to_string()
                .contains("--artifact-cache-source-digest is required")
        );
    }

    #[cfg(feature = "foyer-artifact-cache")]
    #[test]
    fn bootstrap_pipeline_artifact_cache_mode_rejects_unsafe_digests() {
        let cli = Cli::try_parse_from([
            "wendao-episteme",
            "ontology",
            "bootstrap-pipeline",
            "--episteme-root",
            "private-episteme",
            "--run-id",
            "ltc_bootstrap",
            "--artifact-cache-mode",
            "restore-only",
            "--artifact-cache-source-digest",
            "../source",
            "--artifact-cache-profile-digest",
            "bootstrap-v1",
        ])
        .unwrap_or_else(|error| panic!("parse Episteme artifact-cache bootstrap CLI: {error}"));

        let Err(error) = run_cli(cli) else {
            panic!("artifact-cache mode should reject unsafe digests");
        };
        let error = format!("{error:#}");

        assert!(error.contains("invalid Episteme bootstrap artifact-cache digest component"));
        assert!(error.contains("source_digest"));
    }

    #[test]
    fn parses_qianji_schedule_plan_command() {
        let cli = Cli::try_parse_from([
            "wendao-episteme",
            "ontology",
            "qianji-schedule-plan",
            "--reasoning-fill-plan-json",
            "runs/ontology-generation/bootstrap_reasoning_fill_plan/reasoning_fill_plan.json",
            "--run-id",
            "qianji_schedule",
            "--qianji-run-id",
            "episteme.ontology.reasoning.test",
            "--target-ledger-field-group",
            "service_catalog_review",
            "--limit",
            "8",
        ])
        .unwrap_or_else(|error| panic!("parse Episteme Qianji schedule CLI: {error}"));

        let Command::Ontology(args) = cli.command;
        let OntologyCommand::QianjiSchedulePlan(args) = args.command else {
            panic!("expected qianji schedule-plan command");
        };
        assert_eq!(
            args.reasoning_fill_plan_json,
            PathBuf::from(
                "runs/ontology-generation/bootstrap_reasoning_fill_plan/reasoning_fill_plan.json"
            )
        );
        assert_eq!(args.run_id, "qianji_schedule");
        assert_eq!(
            args.qianji_run_id.as_deref(),
            Some("episteme.ontology.reasoning.test")
        );
        assert_eq!(
            args.target_ledger_field_group.as_deref(),
            Some("service_catalog_review")
        );
        assert_eq!(args.limit, 8);
    }

    use std::path::PathBuf;
}
