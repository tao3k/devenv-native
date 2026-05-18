//! Coordinates the Studio wendao types commands branch and keeps its child modules behind one documented reasoning-tree boundary.

mod agentic;
mod attachments;
mod audit;
mod command;
#[path = "docs/mod.rs"]
mod docs;
mod episteme;
mod fix;
#[cfg(feature = "zhenfa-router")]
mod gateway;
mod graph;
mod hmas;
#[cfg(feature = "zhenfa-router")]
#[path = "query/mod.rs"]
mod query;
mod repo;
mod saliency;
mod search;
mod sentinel;

pub(crate) use agentic::AgenticCommand;
pub(crate) use attachments::AttachmentsArgs;
pub(crate) use audit::AuditArgs;
pub(crate) use command::Command;
pub(crate) use docs::{
    DocsCommand, DocsContextArgs, DocsNavigationArgs, DocsNodeArgs, DocsPageArgs,
    DocsPageIndexArgs, DocsPageIndexOutlineArgs, DocsSearchArgs, DocsSearchPageIndexArgs,
    DocsSegmentArgs, DocsTocArgs, DocsTreeArgs,
};
pub(crate) use episteme::{
    EpistemeCommand, EpistemeEvidenceCommand, EpistemeEvidenceReadValidationModeArg,
    EpistemeEvidenceSelectionValidationModeArg, EpistemePlanExtractionRunArgs,
    EpistemeReadEvidenceArgs, EpistemeRunDoclingDocumentCacheArgs, EpistemeRunImageOcrCacheArgs,
    EpistemeSourceContractCommand, EpistemeStructureCommand, EpistemeStructureTocValidationModeArg,
    EpistemeWriteEvidenceSelectionPlanArgs, EpistemeWriteStructureTocArgs,
};
pub(crate) use fix::FixArgs;
#[cfg(feature = "zhenfa-router")]
pub(crate) use gateway::{GatewayArgs, GatewayCommand, GatewayStartArgs};
pub(crate) use graph::{MetadataArgs, NeighborsArgs, RelatedArgs, ResolveArgs, TocArgs};
pub(crate) use hmas::HmasCommand;
#[cfg(feature = "zhenfa-router")]
pub(crate) use query::{GraphqlQueryArgs, QueryCommand, RestQueryArgs, SqlQueryArgs};
pub(crate) use repo::{RepoCommand, RepoSyncModeArg};
pub(crate) use saliency::SaliencyCommand;
pub(crate) use search::SearchArgs;
pub(crate) use sentinel::{SentinelArgs, SentinelCommand, SentinelWatchArgs};
