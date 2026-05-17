use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub(crate) enum EpistemeCommand {
    /// Read targeted episteme evidence by source-contract id.
    Evidence {
        #[command(subcommand)]
        command: EpistemeEvidenceCommand,
    },
    /// Manage episteme source-contract workflows.
    SourceContract {
        #[command(subcommand)]
        command: EpistemeSourceContractCommand,
    },
    /// Generate episteme structure and TOC artifacts.
    Structure {
        #[command(subcommand)]
        command: EpistemeStructureCommand,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum EpistemeEvidenceCommand {
    /// Read one source-contract evidence row by file id.
    Read(EpistemeReadEvidenceArgs),
    /// Write an evidence-only selection plan for chosen file ids.
    WriteSelectionPlan(EpistemeWriteEvidenceSelectionPlanArgs),
}

#[derive(Subcommand, Debug)]
pub(crate) enum EpistemeSourceContractCommand {
    /// Write a deterministic extraction run plan without executing extraction.
    PlanExtractionRun(EpistemePlanExtractionRunArgs),
    /// Run the image OCR cache bridge for source-contract image tasks.
    RunImageOcrCache(EpistemeRunImageOcrCacheArgs),
}

#[derive(Subcommand, Debug)]
pub(crate) enum EpistemeStructureCommand {
    /// Write an evidence-only Org TOC ledger without executing extraction.
    WriteToc(EpistemeWriteStructureTocArgs),
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeReadEvidenceArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Corpus root. Defaults to the env var named by episteme config.
    #[arg(long, value_name = "DIR")]
    pub corpus_root: Option<PathBuf>,
    /// Source-contract file id to read.
    #[arg(long, value_name = "ID")]
    pub file_id: String,
    /// Maximum bytes to include for supported text previews.
    #[arg(long, default_value_t = 8192)]
    pub max_preview_bytes: usize,
    /// Evidence read validation policy.
    #[arg(long, value_enum, default_value_t = EpistemeEvidenceReadValidationModeArg::MetadataOnly)]
    pub validation_mode: EpistemeEvidenceReadValidationModeArg,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteEvidenceSelectionPlanArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Corpus root. Defaults to the env var named by episteme config.
    #[arg(long, value_name = "DIR")]
    pub corpus_root: Option<PathBuf>,
    /// Run artifact root. Defaults to <episteme-root>/runs/evidence-selection.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
    /// Source-contract file id selected for downstream evidence work.
    #[arg(long = "file-id", value_name = "ID", required = true)]
    pub file_ids: Vec<String>,
    /// Run-level reason recorded in the selection ledger.
    #[arg(long, default_value = "manual_or_agent_selected")]
    pub selection_reason: String,
    /// Evidence selection validation policy.
    #[arg(long, value_enum, default_value_t = EpistemeEvidenceSelectionValidationModeArg::MetadataOnly)]
    pub validation_mode: EpistemeEvidenceSelectionValidationModeArg,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemePlanExtractionRunArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Corpus root. Defaults to the env var named by episteme config.
    #[arg(long, value_name = "DIR")]
    pub corpus_root: Option<PathBuf>,
    /// Run artifact root. Defaults to <episteme-root>/runs/extraction.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
    /// Optional extraction route filter.
    #[arg(long, value_name = "ROUTE")]
    pub route: Option<String>,
    /// Optional source category filter.
    #[arg(long, value_name = "CATEGORY")]
    pub category: Option<String>,
    /// Maximum number of queue rows to select.
    #[arg(long, default_value_t = 12)]
    pub limit: usize,
    /// Evidence selection run id used to constrain extraction planning.
    #[arg(long, value_name = "ID")]
    pub selection_run_id: Option<String>,
    /// Evidence selection artifact root. Defaults to <episteme-root>/runs/evidence-selection.
    #[arg(long, value_name = "DIR")]
    pub selection_root: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeRunImageOcrCacheArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Corpus root. Defaults to the env var named by episteme config.
    #[arg(long, value_name = "DIR")]
    pub corpus_root: Option<PathBuf>,
    /// Run artifact root. Defaults to <episteme-root>/runs/extraction.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
    /// Optional source category filter.
    #[arg(long, value_name = "CATEGORY")]
    pub category: Option<String>,
    /// Maximum number of image queue rows to select.
    #[arg(long, default_value_t = 12)]
    pub limit: usize,
    /// Evidence selection run id used to constrain image OCR planning.
    #[arg(long, value_name = "ID")]
    pub selection_run_id: Option<String>,
    /// Evidence selection artifact root. Defaults to <episteme-root>/runs/evidence-selection.
    #[arg(long, value_name = "DIR")]
    pub selection_root: Option<PathBuf>,
    /// Analyzer command that writes queue-keyed OCR JSONL.
    #[arg(long, default_value = "wendao-image-ocr-jsonl")]
    pub analyzer_command: String,
    /// Python command used to run the private cache bridge script.
    #[arg(long, default_value = "python")]
    pub python_command: String,
    /// Private cache bridge script. Defaults to <episteme-root>/tools/run_extraction_plan.py.
    #[arg(long, value_name = "FILE")]
    pub cache_bridge_script: Option<PathBuf>,
    /// OCR JSONL path. Defaults to <run-root>/<run-id>/ocr_results.jsonl.
    #[arg(long, value_name = "FILE")]
    pub ocr_results_jsonl: Option<PathBuf>,
    /// Write the run plan and print command specs without executing Python commands.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteStructureTocArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Corpus root. Defaults to the env var named by episteme config.
    #[arg(long, value_name = "DIR")]
    pub corpus_root: Option<PathBuf>,
    /// Run artifact root. Defaults to <episteme-root>/runs/structure.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Structure TOC validation policy.
    #[arg(long, value_enum, default_value_t = EpistemeStructureTocValidationModeArg::MetadataOnly)]
    pub validation_mode: EpistemeStructureTocValidationModeArg,
    /// Safe ASCII run id.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default, Eq, PartialEq)]
#[value(rename_all = "kebab-case")]
pub(crate) enum EpistemeStructureTocValidationModeArg {
    /// Validate manifest and file metadata without hashing file contents.
    #[default]
    MetadataOnly,
    /// Run full source-contract validation, including sha256 drift checks.
    FullHash,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default, Eq, PartialEq)]
#[value(rename_all = "kebab-case")]
pub(crate) enum EpistemeEvidenceReadValidationModeArg {
    /// Validate manifest and file metadata without hashing file contents.
    #[default]
    MetadataOnly,
    /// Run full source-contract validation, including sha256 drift checks.
    FullHash,
}

#[derive(ValueEnum, Debug, Clone, Copy, Default, Eq, PartialEq)]
#[value(rename_all = "kebab-case")]
pub(crate) enum EpistemeEvidenceSelectionValidationModeArg {
    /// Validate manifest and file metadata without hashing file contents.
    #[default]
    MetadataOnly,
    /// Run full source-contract validation, including sha256 drift checks.
    FullHash,
}

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/types/commands/episteme/mod.rs"]
mod tests;
