#[derive(Debug, clap::Subcommand, Clone)]
pub(crate) enum DocsCommand {
    /// Open one deterministic docs-facing projected page.
    Page(super::DocsPageArgs),
    /// Open one deterministic docs-facing projected page-index tree.
    Tree(super::DocsTreeArgs),
    /// Open one text-free docs-facing projected page-index tree.
    TreeOutline(super::DocsTreeOutlineArgs),
    /// Open one repo-scoped text-free docs-facing projected page-index tree catalog.
    StructureCatalog(super::DocsStructureCatalogArgs),
    /// Open one precise docs-facing projected markdown segment.
    Segment(super::DocsSegmentArgs),
    /// Search deterministic docs-facing projected page-index nodes.
    SearchStructure(super::DocsSearchStructureArgs),
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
pub(crate) fn docs(command: DocsCommand) -> super::super::Command {
    super::super::Command::Docs { command }
}
