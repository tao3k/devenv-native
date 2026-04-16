//! Studio API endpoint handlers.

pub(crate) mod analysis;
pub mod capabilities;
/// Docs-facing deep-wiki planning handlers.
#[path = "docs/mod.rs"]
pub mod docs;
pub mod graph;
pub mod repo;
pub mod vfs;

pub use capabilities::{get as get_ui_capabilities, get_plugin_artifact};
pub(crate) use docs::{
    family_cluster as docs_family_cluster, family_context as docs_family_context,
    family_search as docs_family_search, navigation as docs_navigation,
    navigation_search as docs_navigation_search, page as docs_page,
    page_index_tree as docs_page_index_tree, planner_item as docs_planner_item,
    planner_queue as docs_planner_queue, planner_rank as docs_planner_rank,
    planner_search as docs_planner_search, planner_workset as docs_planner_workset,
    projected_gap_report as docs_projected_gap_report, retrieval as docs_retrieval,
    retrieval_context as docs_retrieval_context, retrieval_hit as docs_retrieval_hit,
    search as docs_search,
};
pub use graph::topology_3d;
pub use repo::{
    doc_coverage, example_search, import_search, module_search, overview, projected_page,
    projected_page_family_cluster, projected_page_family_context, projected_page_family_search,
    projected_page_index_node, projected_page_index_tree, projected_page_index_tree_search,
    projected_page_index_trees, projected_page_navigation, projected_page_navigation_search,
    projected_page_search, projected_pages, projected_retrieval, projected_retrieval_context,
    projected_retrieval_hit, refine_entity_doc, repo_index, repo_index_status, symbol_search, sync,
};
pub use vfs::{
    cat as vfs_cat, entry as vfs_entry, raw as vfs_raw, root_entries as vfs_root_entries,
    scan as vfs_scan,
};
