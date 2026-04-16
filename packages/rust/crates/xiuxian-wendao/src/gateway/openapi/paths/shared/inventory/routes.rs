use super::{core, docs, graph, projected, repo, search, ui, vfs};
use crate::gateway::RouteContract;

macro_rules! route_contracts {
    ( $( $module:ident :: [ $( $name:ident ),+ $(,)? ] ),+ $(,)? ) => {
        &[
            $(
                $( $module::$name ),+
            ),+
        ]
    };
}

/// Stable inventory for the current Wendao gateway route surface.
pub const WENDAO_GATEWAY_ROUTE_CONTRACTS: &[RouteContract] = route_contracts![
    core::[HEALTH, STATS, NOTIFY],
    vfs::[VFS_ROOT, VFS_SCAN, VFS_CAT, VFS_ENTRY],
    graph::[TOPOLOGY_3D],
    search::[SEARCH_INDEX_STATUS],
    docs::[
        PROJECTED_GAP_REPORT,
        PLANNER_ITEM,
        PLANNER_SEARCH,
        PLANNER_QUEUE,
        PLANNER_RANK,
        PLANNER_WORKSET,
        DOCS_SEARCH,
        DOCS_RETRIEVAL,
        DOCS_RETRIEVAL_CONTEXT,
        DOCS_RETRIEVAL_HIT,
        DOCS_PAGE,
        DOCS_PAGE_INDEX_TREE,
        DOCS_FAMILY_CONTEXT,
        DOCS_FAMILY_SEARCH,
        DOCS_FAMILY_CLUSTER,
        DOCS_NAVIGATION,
        DOCS_NAVIGATION_SEARCH
    ],
    ui::[UI_CAPABILITIES, UI_PLUGIN_ARTIFACT],
    repo::[
        REPO_OVERVIEW,
        REPO_MODULE_SEARCH,
        REPO_SYMBOL_SEARCH,
        REPO_EXAMPLE_SEARCH,
        REPO_IMPORT_SEARCH,
        REPO_DOC_COVERAGE,
        REPO_INDEX_STATUS,
        REPO_INDEX,
        REPO_SYNC
    ],
    projected::[
        REPO_PROJECTED_PAGES,
        REPO_PROJECTED_GAP_REPORT,
        REPO_PROJECTED_PAGE,
        REPO_PROJECTED_PAGE_INDEX_NODE,
        REPO_PROJECTED_RETRIEVAL_HIT,
        REPO_PROJECTED_RETRIEVAL_CONTEXT,
        REPO_PROJECTED_PAGE_FAMILY_CONTEXT,
        REPO_PROJECTED_PAGE_FAMILY_SEARCH,
        REPO_PROJECTED_PAGE_FAMILY_CLUSTER,
        REPO_PROJECTED_PAGE_NAVIGATION,
        REPO_PROJECTED_PAGE_NAVIGATION_SEARCH,
        REPO_PROJECTED_PAGE_INDEX_TREE,
        REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH,
        REPO_PROJECTED_PAGE_SEARCH,
        REPO_PROJECTED_RETRIEVAL,
        REPO_PROJECTED_PAGE_INDEX_TREES
    ],
];
