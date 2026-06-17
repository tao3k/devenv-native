//! Validation gate for xiuxian-wendao.

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/support/mod.rs"]
mod support;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/coactivation_multihop_diffusion.rs"]
mod coactivation_multihop_diffusion;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/coactivation_weighted_propagation.rs"]
mod coactivation_weighted_propagation;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/ppr_weight_precision.rs"]
mod ppr_weight_precision;

#[cfg(all(not(feature = "performance"), feature = "vector-store"))]
#[path = "integration/quantum_fusion_openai_ignition.rs"]
mod quantum_fusion_openai_ignition;

#[cfg(all(not(feature = "performance"), feature = "vector-store"))]
#[path = "integration/quantum_fusion_saliency_budget.rs"]
mod quantum_fusion_saliency_budget;

#[cfg(all(not(feature = "performance"), feature = "vector-store"))]
#[path = "integration/quantum_fusion_saliency_window.rs"]
mod quantum_fusion_saliency_window;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_doc_coverage.rs"]
mod repo_doc_coverage;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_markdown_documents.rs"]
mod docs_markdown_documents;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_search.rs"]
mod docs_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_retrieval.rs"]
mod docs_retrieval;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_retrieval_context.rs"]
mod docs_retrieval_context;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_retrieval_hit.rs"]
mod docs_retrieval_hit;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_planner_item.rs"]
mod docs_planner_item;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_planner_queue.rs"]
mod docs_planner_queue;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_planner_rank.rs"]
mod docs_planner_rank;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_planner_search.rs"]
mod docs_planner_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_planner_workset.rs"]
mod docs_planner_workset;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_navigation_search.rs"]
mod docs_navigation_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_projected_gap_report.rs"]
mod docs_projected_gap_report;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_navigation.rs"]
mod docs_navigation;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_family_search.rs"]
mod docs_family_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_family_context.rs"]
mod docs_family_context;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_family_cluster.rs"]
mod docs_family_cluster;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_page.rs"]
mod docs_page;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_page_index_tree.rs"]
mod docs_page_index_tree;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_page_index_documents.rs"]
mod docs_page_index_documents;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_page_index_trees.rs"]
mod docs_page_index_trees;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_page_index_tree_search.rs"]
mod docs_page_index_tree_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_page_index_node.rs"]
mod docs_page_index_node;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/docs_tool_service.rs"]
mod docs_tool_service;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/dependency_indexer_pyproject.rs"]
mod dependency_indexer_pyproject;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_example_search.rs"]
mod repo_example_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_gap_report.rs"]
mod repo_projected_gap_report;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_intelligence_registry.rs"]
mod repo_intelligence_registry;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_module_search.rs"]
mod repo_module_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_overview.rs"]
mod repo_overview;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page.rs"]
mod repo_projected_page;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_family_cluster.rs"]
mod repo_projected_page_family_cluster;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_family_context.rs"]
mod repo_projected_page_family_context;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_family_search.rs"]
mod repo_projected_page_family_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_index_documents.rs"]
mod repo_projected_page_index_documents;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_index_node.rs"]
mod repo_projected_page_index_node;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_index_tree.rs"]
mod repo_projected_page_index_tree;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_index_tree_search.rs"]
mod repo_projected_page_index_tree_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_index_trees.rs"]
mod repo_projected_page_index_trees;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_navigation.rs"]
mod repo_projected_page_navigation;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_navigation_search.rs"]
mod repo_projected_page_navigation_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_page_search.rs"]
mod repo_projected_page_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_pages.rs"]
mod repo_projected_pages;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_retrieval.rs"]
mod repo_projected_retrieval;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_retrieval_context.rs"]
mod repo_projected_retrieval_context;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projected_retrieval_hit.rs"]
mod repo_projected_retrieval_hit;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "unit/link_graph_agentic/mod.rs"]
mod link_graph_agentic;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "unit/link_graph_saliency/mod.rs"]
mod link_graph_saliency;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_projection_inputs.rs"]
mod repo_projection_inputs;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_relations.rs"]
mod repo_relations;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_symbol_search.rs"]
mod repo_symbol_search;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/repo_sync.rs"]
mod repo_sync;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/scenarios.rs"]
mod scenarios;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "integration/pybindings_feature_smoke.rs"]
mod pybindings_feature_smoke;

#[cfg(feature = "performance")]
#[path = "performance/mod.rs"]
mod performance;

#[cfg(feature = "performance-stress")]
#[path = "performance/stress/mod.rs"]
mod performance_stress;

#[cfg(all(not(feature = "performance"), feature = "test-support"))]
#[path = "unit/semantic_check.rs"]
mod semantic_check;

#[path = "support/relative_visibility_gate.rs"]
mod relative_visibility_gate;
