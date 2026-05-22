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
    /// Run the Docling document cache bridge for supported document tasks.
    RunDoclingDocumentCache(EpistemeRunDoclingDocumentCacheArgs),
    /// Run the legacy Office converter for doc/ppt/xls evidence tasks.
    RunLegacyOfficeConversion(EpistemeRunLegacyOfficeConversionArgs),
    /// Generate review-gated ontology candidate rows from source-contract evidence.
    GenerateOntologyCandidates(EpistemeGenerateOntologyCandidatesArgs),
    /// Review generated ontology candidate rows before any promotion slice.
    ReviewOntologyCandidates(EpistemeReviewOntologyCandidatesArgs),
    /// Export reviewed ontology candidates as RDF draft proposal artifacts.
    WriteOntologyRdfDraft(EpistemeWriteOntologyRdfDraftArgs),
    /// Write a pending-review promotion packet from a clean RDF draft run.
    WriteOntologyPromotionReview(EpistemeWriteOntologyPromotionReviewArgs),
    /// Write a non-mutating apply plan from explicit promotion review decisions.
    WriteOntologyPromotionApplyPlan(EpistemeWriteOntologyPromotionApplyPlanArgs),
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
    /// OCR JSONL path. Defaults to `<run-root>/<run-id>/ocr_results.jsonl`.
    #[arg(long, value_name = "FILE")]
    pub ocr_results_jsonl: Option<PathBuf>,
    /// Reuse an existing OCR JSONL file and skip analyzer execution.
    #[arg(long)]
    pub use_existing_results: bool,
    /// Write the run plan and print command specs without executing Python commands.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeRunDoclingDocumentCacheArgs {
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
    /// Maximum number of document queue rows to select.
    #[arg(long, default_value_t = 12)]
    pub limit: usize,
    /// Evidence selection run id used to constrain Docling document planning.
    #[arg(long, value_name = "ID")]
    pub selection_run_id: Option<String>,
    /// Evidence selection artifact root. Defaults to <episteme-root>/runs/evidence-selection.
    #[arg(long, value_name = "DIR")]
    pub selection_root: Option<PathBuf>,
    /// Analyzer command that writes queue-keyed Docling document JSONL.
    #[arg(long, default_value = "wendao-docling-document-jsonl")]
    pub analyzer_command: String,
    /// Wendao Docling document extraction profile.
    #[arg(long, default_value = "full")]
    pub docling_profile: String,
    /// Docling document JSONL path. Defaults to `<run-root>/<run-id>/document_results.jsonl`.
    #[arg(long, value_name = "FILE")]
    pub document_results_jsonl: Option<PathBuf>,
    /// Reuse an existing Docling document JSONL file and skip analyzer execution.
    #[arg(long)]
    pub use_existing_results: bool,
    /// Write the run plan and print command specs without executing extraction.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeRunLegacyOfficeConversionArgs {
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
    /// Maximum number of legacy Office queue rows to select.
    #[arg(long, default_value_t = 12)]
    pub limit: usize,
    /// Evidence selection run id used to constrain legacy Office conversion planning.
    #[arg(long, value_name = "ID")]
    pub selection_run_id: Option<String>,
    /// Evidence selection artifact root. Defaults to <episteme-root>/runs/evidence-selection.
    #[arg(long, value_name = "DIR")]
    pub selection_root: Option<PathBuf>,
    /// Converter executable or wrapper. Defaults to episteme.toml runtime config.
    #[arg(long, value_name = "FILE")]
    pub converter_command: Option<PathBuf>,
    /// Write the run plan and conversion receipt without executing the converter.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeGenerateOntologyCandidatesArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Extraction run root. Defaults to <episteme-root>/runs/extraction.
    #[arg(long, value_name = "DIR")]
    pub extraction_run_root: Option<PathBuf>,
    /// Safe ASCII run id.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
    /// Extraction run id whose cache outputs should seed evidence. May repeat.
    #[arg(long = "extraction-run-id", value_name = "ID")]
    pub extraction_run_ids: Vec<String>,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeReviewOntologyCandidatesArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id to review.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteOntologyRdfDraftArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id to export.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteOntologyPromotionReviewArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id to review for promotion.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteOntologyPromotionApplyPlanArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id whose promotion review decisions should be planned.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
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
