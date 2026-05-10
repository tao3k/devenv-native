mod analysis;
mod ast;
mod attachments;
mod autocomplete;
mod contracts;
mod definition;
mod fixtures;
mod global_search;
mod graph;
mod headers;
mod helpers;
mod materialization;
mod provider;
mod vfs;

use super::{
    StudioSearchFlightRouteProvider, build_studio_search_flight_service_with_repo_provider,
};
use fixtures::{
    build_analysis_route_service, build_service, make_gateway_state_with_attachments,
    make_gateway_state_with_docs, make_gateway_state_with_search_routes,
    make_gateway_state_with_search_strategy_flow_routes,
};
use headers::{
    populate_attachment_headers, populate_autocomplete_headers, populate_definition_headers,
    populate_graph_neighbors_headers, populate_markdown_analysis_headers,
    populate_repo_index_headers, populate_repo_index_status_headers, populate_repo_search_headers,
    populate_repo_sync_headers, populate_search_headers, populate_topology_3d_headers,
    populate_vfs_content_headers, populate_vfs_resolve_headers, populate_vfs_scan_headers,
};
use headers::{
    populate_code_ast_analysis_headers, populate_refine_doc_headers,
    populate_repo_doc_coverage_headers, populate_repo_overview_headers,
    populate_repo_projected_page_index_tree_headers,
    populate_repo_projected_retrieval_context_headers,
};
use helpers::{
    assert_route_ticket, assert_studio_flight_snapshot, collect_route_batches, fetch_flight_info,
    first_string, first_ticket,
};
