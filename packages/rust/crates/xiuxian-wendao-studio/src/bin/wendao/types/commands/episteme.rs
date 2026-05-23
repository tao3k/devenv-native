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
    /// Compile deterministic structural IDF seed rows from source-contract files.
    WriteStructuralIdf(EpistemeWriteStructuralIdfArgs),
    /// Compile structural IDF rows into a deterministic reasoning packet.
    WriteStructuralIdfReasoningPacket(EpistemeWriteStructuralIdfReasoningPacketArgs),
    /// Compile a reasoning packet into a fillable Org ledger seed.
    WriteStructuralIdfReasoningLedgerSeed(EpistemeWriteStructuralIdfReasoningLedgerSeedArgs),
    /// Compile a reasoning ledger seed into workflow fill-plan rows.
    WriteStructuralIdfReasoningFillPlan(EpistemeWriteStructuralIdfReasoningFillPlanArgs),
    /// Compile a reasoning fill plan into Qianji schedule-admission inputs.
    WriteStructuralIdfReasoningQianjiSchedulePlan(
        EpistemeWriteStructuralIdfReasoningQianjiSchedulePlanArgs,
    ),
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
    /// Write a non-mutating source-patch preflight from approved review ledgers.
    WriteOntologySourcePatchPreflight(EpistemeWriteOntologySourcePatchPreflightArgs),
    /// Export a non-mutating RDF draft from source-patch preflight rows.
    WriteOntologySourcePatchDraft(EpistemeWriteOntologySourcePatchDraftArgs),
    /// Write a non-mutating source-patch apply plan from draft receipts.
    WriteOntologySourcePatchApplyPlan(EpistemeWriteOntologySourcePatchApplyPlanArgs),
    /// Write a hash-guarded source-patch review packet from an apply plan.
    WriteOntologySourcePatchReviewPacket(EpistemeWriteOntologySourcePatchReviewPacketArgs),
    /// Write a no-mutation preview of a reviewed source patch.
    WriteOntologySourcePatchApplyPreview(EpistemeWriteOntologySourcePatchApplyPreviewArgs),
    /// Compile source-patch preview rows into graph-ready semantic read-model artifacts.
    WriteOntologySourcePatchSemanticPreview(EpistemeWriteOntologySourcePatchSemanticPreviewArgs),
    /// Compile applied source-patch RDF into graph-ready semantic read-model artifacts.
    WriteOntologySourcePatchRdfReadModel(EpistemeWriteOntologySourcePatchRdfReadModelArgs),
    /// Apply a reviewed source patch through an explicit hash gate.
    ApplyOntologySourcePatch(EpistemeApplyOntologySourcePatchArgs),
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
pub(crate) struct EpistemeWriteStructuralIdfArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Corpus root. Defaults to the env var or episteme runtime config.
    #[arg(long, value_name = "DIR")]
    pub corpus_root: Option<PathBuf>,
    /// Run artifact root. Defaults to <episteme-root>/runs/structure.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Structural IDF validation policy.
    #[arg(long, value_enum, default_value_t = EpistemeStructuralIdfValidationModeArg::MetadataOnly)]
    pub validation_mode: EpistemeStructuralIdfValidationModeArg,
    /// Safe ASCII run id.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteStructuralIdfReasoningPacketArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Structural IDF run root. Defaults to <episteme-root>/runs/structure.
    #[arg(long, value_name = "DIR")]
    pub structure_run_root: Option<PathBuf>,
    /// Reasoning packet run root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Structural IDF run id used as packet input.
    #[arg(long, value_name = "ID")]
    pub structural_idf_run_id: String,
    /// Safe ASCII packet run id.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
    /// Optional source category filter.
    #[arg(long, value_name = "CATEGORY")]
    pub category: Option<String>,
    /// Optional extraction route filter.
    #[arg(long, value_name = "ROUTE")]
    pub route: Option<String>,
    /// Maximum number of packet rows to emit.
    #[arg(long, default_value_t = 256)]
    pub limit: usize,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteStructuralIdfReasoningLedgerSeedArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Reasoning packet run root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub reasoning_packet_root: Option<PathBuf>,
    /// Ledger-seed run root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Reasoning packet run id used as ledger-seed input.
    #[arg(long, value_name = "ID")]
    pub reasoning_packet_run_id: String,
    /// Safe ASCII ledger-seed run id.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
    /// Maximum number of packet rows to seed.
    #[arg(long, default_value_t = 512)]
    pub limit: usize,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteStructuralIdfReasoningFillPlanArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Ledger-seed run root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub ledger_seed_root: Option<PathBuf>,
    /// Fill-plan run root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Ledger-seed run id used as fill-plan input.
    #[arg(long, value_name = "ID")]
    pub ledger_seed_run_id: String,
    /// Safe ASCII fill-plan run id.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
    /// Maximum number of seed rows to plan.
    #[arg(long, default_value_t = 1024)]
    pub limit: usize,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteStructuralIdfReasoningQianjiSchedulePlanArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Fill-plan run root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub fill_plan_root: Option<PathBuf>,
    /// Qianji schedule-plan run root. Defaults to <episteme-root>/runs/ontology-generation.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Fill-plan run id used as Qianji schedule-plan input.
    #[arg(long, value_name = "ID")]
    pub fill_plan_run_id: String,
    /// Safe ASCII schedule-plan run id.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
    /// Optional Qianji run id carried by generated task payloads.
    #[arg(long, value_name = "ID")]
    pub qianji_run_id: Option<String>,
    /// Maximum number of fill-plan rows to schedule.
    #[arg(long, default_value_t = 1024)]
    pub limit: usize,
    /// Optional OpenAI-compatible model id for prompt-audit task admission.
    #[arg(long, value_name = "MODEL")]
    pub openai_compatible_model: Option<String>,
    /// Maximum model output tokens for OpenAI-compatible prompt-audit tasks.
    #[arg(long, default_value_t = 1024)]
    pub openai_compatible_max_tokens: u32,
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
pub(crate) struct EpistemeWriteOntologySourcePatchPreflightArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/source-patch-preflight.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id whose approved review ledgers should be preflighted.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteOntologySourcePatchDraftArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/source-patch-preflight.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id whose source-patch preflight should be drafted.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteOntologySourcePatchApplyPlanArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/source-patch-preflight.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id whose source-patch draft should be planned.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteOntologySourcePatchReviewPacketArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/source-patch-preflight.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id whose source-patch apply plan should be packetized.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteOntologySourcePatchApplyPreviewArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/source-patch-preflight.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id whose reviewed source patch should be previewed.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
    /// Expected apply-plan TSV hash copied from the review packet.
    #[arg(long, value_name = "SHA256")]
    pub expected_apply_plan_tsv_sha256: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteOntologySourcePatchSemanticPreviewArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/source-patch-preflight.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id whose admitted preview should be compiled.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeWriteOntologySourcePatchRdfReadModelArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/source-patch-preflight.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id whose applied source-patch RDF should be projected.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct EpistemeApplyOntologySourcePatchArgs {
    /// Episteme repository root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub episteme_root: PathBuf,
    /// Episteme registry id from `wendao.toml`.
    #[arg(long, value_name = "ID")]
    pub episteme_registry_id: Option<String>,
    /// Run artifact root. Defaults to <episteme-root>/runs/source-patch-preflight.
    #[arg(long, value_name = "DIR")]
    pub run_root: Option<PathBuf>,
    /// Safe ASCII run id whose reviewed source patch should be applied.
    #[arg(long, value_name = "ID")]
    pub run_id: String,
    /// Expected apply-plan TSV hash copied from the review packet.
    #[arg(long, value_name = "SHA256")]
    pub expected_apply_plan_tsv_sha256: String,
    /// Explicitly authorize mutation of the target ontology source file.
    #[arg(long)]
    pub allow_source_mutation: bool,
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
pub(crate) enum EpistemeStructuralIdfValidationModeArg {
    /// Validate manifest and file metadata without hashing file contents.
    #[default]
    MetadataOnly,
    /// Run full source validation, including sha256 drift checks.
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
