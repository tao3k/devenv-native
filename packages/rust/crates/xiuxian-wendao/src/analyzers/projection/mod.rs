//! `analyzers::projection` owns Wendao analyzers projection behavior.

#[path = "builder/mod.rs"]
mod builder;
mod contracts;
mod family_context;
mod family_lookup;
mod family_search;
mod gap_report;
mod lookup;
mod markdown;
mod mixed_search;
mod navigation_bundle;
mod navigation_search;
mod node_lookup;
mod pages;
mod related_pages;
mod retrieval_context;
mod retrieval_lookup;
#[path = "search/mod.rs"]
mod search;
mod tree_lookup;
mod tree_search;

pub use builder::build_projection_inputs;
pub use contracts::{
    ProjectedMarkdownDocument, ProjectedPageIndexDocument, ProjectedPageIndexNode,
    ProjectedPageIndexSection, ProjectedPageIndexTree, ProjectedPageRecord, ProjectedPageSection,
    ProjectionInputBundle, ProjectionPageKind, ProjectionPageSeed,
};
pub use family_context::build_projected_page_family_context;
pub use family_lookup::build_projected_page_family_cluster;
pub use family_search::build_repo_projected_page_family_search as build_projected_page_family_search;
pub use gap_report::build_projected_gap_report;
pub use lookup::build_projected_page;
pub use markdown::{
    build_projected_page_index_documents, build_projected_page_index_trees,
    render_projected_markdown_documents,
};
pub use mixed_search::build_projected_retrieval;
pub use navigation_bundle::build_projected_page_navigation;
pub use navigation_search::build_repo_projected_page_navigation_search as build_projected_page_navigation_search;
pub use node_lookup::build_repo_projected_page_index_node as build_projected_page_index_node;
pub use pages::build_projected_pages;
pub use retrieval_context::build_projected_retrieval_context;
pub use retrieval_lookup::build_projected_retrieval_hit;
#[cfg(feature = "search-runtime")]
pub use search::build_projected_page_search_index;
pub use search::build_repo_projected_page_search as build_projected_page_search;
#[cfg(feature = "search-runtime")]
pub use search::build_repo_projected_page_search_with_artifacts;
pub use tree_lookup::build_projected_page_index_tree;
pub use tree_search::build_repo_projected_page_index_tree_search as build_projected_page_index_tree_search;
