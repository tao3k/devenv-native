//! CLI argument contracts and enum adapters for `wendao`.

#[path = "cli.rs"]
mod cli;
#[path = "commands/mod.rs"]
mod commands;
#[path = "enums.rs"]
mod enums;

pub(crate) use cli::Cli;
pub(crate) use commands::{
    AgenticCommand, AuditArgs, Command, DocsCommand, DocsContextArgs, DocsNavigationArgs,
    DocsNodeArgs, DocsPageArgs, DocsPageIndexArgs, DocsPageIndexOutlineArgs, DocsSearchArgs,
    DocsSearchPageIndexArgs, DocsSegmentArgs, DocsTocArgs, DocsTreeArgs,
    EpistemeApplyOntologySourcePatchArgs, EpistemeCommand, EpistemeEvidenceCommand,
    EpistemeEvidenceReadValidationModeArg, EpistemeEvidenceSelectionValidationModeArg,
    EpistemeGenerateOntologyCandidatesArgs, EpistemePlanExtractionRunArgs,
    EpistemeReadEvidenceArgs, EpistemeReviewOntologyCandidatesArgs,
    EpistemeRunDoclingDocumentCacheArgs, EpistemeRunImageOcrCacheArgs,
    EpistemeRunLegacyOfficeConversionArgs, EpistemeSourceContractCommand,
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
    FixArgs, HmasCommand, SaliencyCommand, SentinelArgs, SentinelCommand, SentinelWatchArgs,
};
#[cfg(feature = "zhenfa-router")]
pub(crate) use commands::{
    GatewayArgs, GatewayCommand, GatewayStartArgs, GraphqlQueryArgs, QueryCommand, RestQueryArgs,
    SqlQueryArgs,
};
pub(crate) use commands::{RepoCommand, RepoSyncModeArg};
pub(crate) use enums::{
    AttachmentKindArg, DecisionTargetStateArg, LinkGraphScopeArg, OutputFormat,
    ProjectionPageKindArg, RelatedPprSubgraphModeArg, SuggestedLinkStateArg,
};
