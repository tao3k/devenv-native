//! Command model for host-runtime-agnostic `wendao get` subcommands.

/// Reusable `wendao get` subcommands that stay host-runtime agnostic.
#[derive(Debug, clap::Subcommand, Clone)]
pub enum GetCommand {
    /// Open one target-scoped TOC/page-index document collection.
    Toc(super::GetScopeArgs),
    /// Open one target-scoped text-free page-index tree collection.
    PageIndex(super::GetScopeArgs),
}
