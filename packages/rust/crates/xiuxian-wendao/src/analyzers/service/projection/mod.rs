//! Repository projection functions (projected pages, retrieval, navigation, and gap reports).

#[path = "docs_tool/mod.rs"]
mod docs_tool;
mod family;
mod gap;
mod index_tree;
mod navigation;
mod page;
mod pages;
#[path = "planner/mod.rs"]
mod planner;
mod registry;
mod retrieval;
mod search;

#[cfg(all(test, feature = "repo-lexical-index", feature = "search-runtime"))]
#[path = "../../../../tests/unit/analyzers/service/projection/mod.rs"]
mod tests;

#[cfg(all(feature = "zhenfa-router", feature = "julia", test))]
pub use docs_tool::DocsDocumentSegmentResult;
pub use docs_tool::{
    DOCS_CONTRACT_IDS, DOCS_DOCUMENT_CONTRACT_ID, DOCS_NAVIGATION_CONTRACT_ID,
    DOCS_PAGE_INDEX_TREE_CONTRACT_ID, DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID, DOCS_SEARCH_CONTRACT_ID,
    DocsCapabilityContractAssets, DocsCapabilityContractSnapshot, DocsCliContractSnapshot,
    DocsContractDefaultValue, DocsContractParamSnapshot, DocsDocumentToolArgs,
    DocsHttpContractSnapshot, DocsNavigationOptions, DocsNavigationToolArgs,
    DocsPageIndexTreeToolArgs, DocsRetrievalContextOptions, DocsRetrievalContextToolArgs,
    DocsSearchToolArgs, DocsToolContractSnapshot, DocsToolService, docs_capability_contract_assets,
    docs_capability_contract_snapshot, docs_capability_schema_snapshot,
};
#[cfg(feature = "zhenfa-router")]
pub(crate) use docs_tool::{DocsToolRuntime, DocsToolRuntimeHandle};
pub use family::{
    build_docs_family_cluster, build_docs_family_context, build_docs_family_search,
    build_repo_projected_page_family_cluster, build_repo_projected_page_family_context,
    build_repo_projected_page_family_search, docs_family_cluster_from_config,
    docs_family_cluster_from_config_with_registry, docs_family_context_from_config,
    docs_family_context_from_config_with_registry, docs_family_search_from_config,
    docs_family_search_from_config_with_registry, repo_projected_page_family_cluster_from_config,
    repo_projected_page_family_cluster_from_config_with_registry,
    repo_projected_page_family_context_from_config,
    repo_projected_page_family_context_from_config_with_registry,
    repo_projected_page_family_search_from_config,
    repo_projected_page_family_search_from_config_with_registry,
};
pub use gap::{
    build_docs_projected_gap_report, build_repo_projected_gap_report,
    docs_projected_gap_report_from_config, docs_projected_gap_report_from_config_with_registry,
    repo_projected_gap_report_from_config, repo_projected_gap_report_from_config_with_registry,
};
pub use index_tree::{
    build_docs_page_index_documents, build_docs_page_index_node, build_docs_page_index_tree,
    build_docs_page_index_tree_search, build_docs_page_index_trees,
    build_repo_projected_page_index_node, build_repo_projected_page_index_tree,
    build_repo_projected_page_index_tree_search, build_repo_projected_page_index_trees,
    docs_page_index_documents_from_config, docs_page_index_documents_from_config_with_registry,
    docs_page_index_node_from_config, docs_page_index_node_from_config_with_registry,
    docs_page_index_tree_from_config, docs_page_index_tree_from_config_with_registry,
    docs_page_index_tree_search_from_config, docs_page_index_tree_search_from_config_with_registry,
    docs_page_index_trees_from_config, docs_page_index_trees_from_config_with_registry,
    repo_projected_page_index_node_from_config,
    repo_projected_page_index_node_from_config_with_registry,
    repo_projected_page_index_tree_from_config,
    repo_projected_page_index_tree_from_config_with_registry,
    repo_projected_page_index_tree_search_from_config,
    repo_projected_page_index_tree_search_from_config_with_registry,
    repo_projected_page_index_trees_from_config,
    repo_projected_page_index_trees_from_config_with_registry,
};
pub use navigation::{
    build_docs_navigation, build_docs_navigation_search, build_repo_projected_page_navigation,
    build_repo_projected_page_navigation_search, docs_navigation_from_config,
    docs_navigation_from_config_with_registry, docs_navigation_search_from_config,
    docs_navigation_search_from_config_with_registry, repo_projected_page_navigation_from_config,
    repo_projected_page_navigation_from_config_with_registry,
    repo_projected_page_navigation_search_from_config,
    repo_projected_page_navigation_search_from_config_with_registry,
};
pub use page::{
    build_docs_markdown_documents, build_docs_page, build_repo_projected_page,
    docs_markdown_documents_from_config, docs_markdown_documents_from_config_with_registry,
    docs_page_from_config, docs_page_from_config_with_registry, repo_projected_page_from_config,
    repo_projected_page_from_config_with_registry,
};
pub use pages::{
    build_repo_projected_pages, repo_projected_pages_from_config,
    repo_projected_pages_from_config_with_registry,
};
pub use planner::{
    build_docs_planner_item, build_docs_planner_queue, build_docs_planner_rank,
    build_docs_planner_search, build_docs_planner_workset, docs_planner_item_from_config,
    docs_planner_item_from_config_with_registry, docs_planner_queue_from_config,
    docs_planner_queue_from_config_with_registry, docs_planner_rank_from_config,
    docs_planner_rank_from_config_with_registry, docs_planner_search_from_config,
    docs_planner_search_from_config_with_registry, docs_planner_workset_from_config,
    docs_planner_workset_from_config_with_registry,
};
pub use retrieval::{
    build_docs_retrieval, build_docs_retrieval_context, build_docs_retrieval_hit,
    build_repo_projected_retrieval, build_repo_projected_retrieval_context,
    build_repo_projected_retrieval_hit, docs_retrieval_context_from_config,
    docs_retrieval_context_from_config_with_registry, docs_retrieval_from_config,
    docs_retrieval_from_config_with_registry, docs_retrieval_hit_from_config,
    docs_retrieval_hit_from_config_with_registry, repo_projected_retrieval_context_from_config,
    repo_projected_retrieval_context_from_config_with_registry,
    repo_projected_retrieval_from_config, repo_projected_retrieval_from_config_with_registry,
    repo_projected_retrieval_hit_from_config,
    repo_projected_retrieval_hit_from_config_with_registry,
};
#[cfg(feature = "search-runtime")]
pub use search::build_repo_projected_page_search_with_artifacts;
pub use search::{
    build_docs_search, build_repo_projected_page_search, docs_search_from_config,
    docs_search_from_config_with_registry, repo_projected_page_search_from_config,
    repo_projected_page_search_from_config_with_registry,
};
