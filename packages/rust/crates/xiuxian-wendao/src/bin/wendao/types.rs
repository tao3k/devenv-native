//! CLI argument contracts and enum adapters for `wendao`.

#[path = "types/cli.rs"]
mod cli;
#[path = "types/commands/mod.rs"]
mod commands;
#[path = "types/enums.rs"]
mod enums;

pub(crate) use cli::Cli;
pub(crate) use commands::{
    AgenticCommand, AuditArgs, Command, DocsCommand, DocsContextArgs, DocsNavigationArgs,
    DocsNodeArgs, DocsPageArgs, DocsSearchArgs, DocsSearchStructureArgs, DocsSegmentArgs,
    DocsStructureCatalogArgs, DocsTocArgs, DocsTreeArgs, DocsTreeOutlineArgs, FixArgs, HmasCommand,
    SaliencyCommand, SentinelArgs, SentinelCommand, SentinelWatchArgs,
};
#[cfg(feature = "zhenfa-router")]
pub(crate) use commands::{
    GatewayArgs, GatewayCommand, GatewayStartArgs, GraphqlQueryArgs, QueryCommand, RestQueryArgs,
    SqlQueryArgs,
};
pub(crate) use commands::{RepoCommand, RepoSyncModeArg};
pub(crate) use enums::{
    LinkGraphScopeArg, OutputFormat, ProjectionPageKindArg, RelatedPprSubgraphModeArg,
};
