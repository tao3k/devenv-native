#[derive(Debug, clap::Subcommand, Clone)]
pub(crate) enum DocsCommand {
    /// Open one deterministic docs-facing projected page.
    Page(super::DocsPageArgs),
    /// Open one deterministic docs-facing projected page-index tree.
    Tree(super::DocsTreeArgs),
    /// Open one text-free docs-facing projected page-index tree.
    #[command(name = "tree-outline")]
    PageIndexOutline(super::DocsPageIndexOutlineArgs),
    /// Open one repo-scoped text-free docs-facing projected page-index tree catalog.
    PageIndex(super::DocsPageIndexArgs),
    /// Open one precise docs-facing projected markdown segment.
    Segment(super::DocsSegmentArgs),
    /// Search deterministic docs-facing projected pages.
    Search(super::DocsSearchArgs),
    /// Search deterministic docs-facing projected page-index nodes.
    #[command(name = "search-structure")]
    SearchPageIndex(super::DocsSearchPageIndexArgs),
    /// Open one deterministic docs-facing projected page-index node.
    Node(super::DocsNodeArgs),
    /// Open repository-scoped docs markdown TOC/page-index documents.
    Toc(super::DocsTocArgs),
    /// Open one deterministic docs-facing navigation bundle.
    Navigation(super::DocsNavigationArgs),
    /// Open one deterministic docs-facing retrieval context bundle.
    Context(super::DocsContextArgs),
}

#[cfg(test)]
pub(crate) fn docs(command: DocsCommand) -> crate::bin_support::wendao::types::Command {
    crate::bin_support::wendao::types::Command::Docs { command }
}
