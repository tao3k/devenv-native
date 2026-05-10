use super::{
    ContractOwner, JuliaTaskComplexityClass, LaneCapability, LinkGraphJuliaRerankRuntimeConfig,
    SnapshotInvariantError, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
    WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
    WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID, WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
    WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID, julia_graph_compute_profile_refs,
    julia_graph_compute_snapshot, required, wendaograph_algorithm_ref, wendaograph_algorithm_refs,
    wendaograph_algorithm_refs_for_profile, wendaograph_gnn_algorithm_refs,
    wendaograph_link_graph_algorithm_refs, wendaograph_page_index_algorithm_refs,
    wendaograph_relationship_search_algorithm_refs,
    wendaograph_search_strategy_flow_algorithm_refs,
};

#[test]
fn graph_compute_refs_cover_wendaograph_and_wendaosearch_contracts() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig::default();

    let references = julia_graph_compute_profile_refs(&runtime);

    assert_eq!(references.len(), 6);
    assert!(
        references
            .iter()
            .all(|reference| reference.owner == ContractOwner::Julia)
    );
    assert!(references.iter().any(
        |reference| reference.profile.as_deref() == Some(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID)
    ));
    assert!(
        references
            .iter()
            .any(|reference| reference.profile.as_deref()
                == Some(WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID))
    );
    assert!(references.iter().any(
        |reference| reference.profile.as_deref() == Some(WENDAO_GRAPH_GNN_REASONING_PROFILE_ID)
    ));
    assert!(references.iter().any(
        |reference| reference.profile.as_deref() == Some(WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID)
    ));
    assert!(
        references
            .iter()
            .any(|reference| reference.profile.as_deref()
                == Some(WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID))
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.profile.as_deref()
                == Some(WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID))
    );
}

#[test]
fn graph_compute_snapshot_materializes_contract_refs() -> Result<(), SnapshotInvariantError> {
    let runtime = LinkGraphJuliaRerankRuntimeConfig::default();

    let snapshot = julia_graph_compute_snapshot(&runtime)?;

    assert_eq!(snapshot.route_refs().len(), 6);
    assert!(snapshot.admission_budgets().is_empty());
    assert!(snapshot.lane_evidence().is_empty());
    Ok(())
}

#[test]
fn wendaograph_algorithm_catalog_covers_staged_algorithm_families() {
    let references = wendaograph_algorithm_refs();

    assert_eq!(wendaograph_link_graph_algorithm_refs().len(), 17);
    assert_eq!(wendaograph_relationship_search_algorithm_refs().len(), 10);
    assert_eq!(wendaograph_page_index_algorithm_refs().len(), 3);
    assert_eq!(wendaograph_search_strategy_flow_algorithm_refs().len(), 4);
    assert_eq!(wendaograph_gnn_algorithm_refs().len(), 4);
    assert_eq!(references.len(), 38);
    assert!(
        references
            .iter()
            .all(|reference| reference.capability == LaneCapability::GraphEvidenceCompute)
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "link_graph.topology_community_frontier")
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "relationship_search.hnsw_semantic_fanout")
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "page_index.planner_actions")
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "search_strategy_flow.frontier_rows")
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "gnn.node_scores")
    );
}

#[test]
fn wendaograph_algorithm_catalog_groups_by_profile() {
    let link_graph = wendaograph_algorithm_refs_for_profile(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID);
    let page_index =
        wendaograph_algorithm_refs_for_profile(WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID);
    let gnn = wendaograph_algorithm_refs_for_profile(WENDAO_GRAPH_GNN_REASONING_PROFILE_ID);

    assert_eq!(link_graph.len(), 27);
    assert_eq!(page_index.len(), 7);
    assert_eq!(gnn.len(), 4);
    assert!(wendaograph_algorithm_refs_for_profile("missing").is_empty());
    assert!(
        page_index
            .iter()
            .all(|reference| matches!(reference.family, "page_index" | "search_strategy_flow"))
    );
}

#[test]
fn wendaograph_search_strategy_flow_catalog_aligns_graph_owned_contract() {
    let references = wendaograph_search_strategy_flow_algorithm_refs();

    let candidates = required(
        wendaograph_algorithm_ref("search_strategy_flow.candidate_rows"),
        "search_strategy_flow.candidate_rows algorithm ref",
    );
    let transitions = required(
        wendaograph_algorithm_ref("search_strategy_flow.transition_rows"),
        "search_strategy_flow.transition_rows algorithm ref",
    );
    let frontier = required(
        wendaograph_algorithm_ref("search_strategy_flow.frontier_rows"),
        "search_strategy_flow.frontier_rows algorithm ref",
    );
    let tables = required(
        wendaograph_algorithm_ref("search_strategy_flow.tables"),
        "search_strategy_flow.tables algorithm ref",
    );

    assert_eq!(references.len(), 4);
    assert!(references.iter().all(|reference| {
        reference.family == "search_strategy_flow"
            && reference.profile_id == WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
            && reference.capability == LaneCapability::GraphEvidenceCompute
    }));
    assert_eq!(
        candidates.julia_entrypoint,
        "WendaoGraph.strategy_flow_candidate_rows"
    );
    assert_eq!(candidates.output_table, Some("strategy_candidates"));
    assert_eq!(
        transitions.julia_entrypoint,
        "WendaoGraph.strategy_flow_transition_rows"
    );
    assert_eq!(transitions.output_table, Some("strategy_transitions"));
    assert_eq!(transitions.complexity, JuliaTaskComplexityClass::Simple);
    assert_eq!(
        frontier.julia_entrypoint,
        "WendaoGraph.strategy_flow_frontier_rows"
    );
    assert_eq!(frontier.output_table, Some("strategy_frontier"));
    assert_eq!(tables.julia_entrypoint, "WendaoGraph.strategy_flow_tables");
    assert_eq!(tables.output_table, None);
}

#[test]
fn wendaograph_relationship_search_catalog_aligns_graph_search_families() {
    let references = wendaograph_relationship_search_algorithm_refs();

    let hnsw = required(
        wendaograph_algorithm_ref("relationship_search.hnsw_semantic_fanout"),
        "relationship_search.hnsw_semantic_fanout algorithm ref",
    );
    let moc = required(
        wendaograph_algorithm_ref("relationship_search.moc_community_grouping"),
        "relationship_search.moc_community_grouping algorithm ref",
    );
    let ppr = required(
        wendaograph_algorithm_ref("relationship_search.ppr_like_relatedness"),
        "relationship_search.ppr_like_relatedness algorithm ref",
    );
    let ranking = required(
        wendaograph_algorithm_ref("relationship_search.graph_search_ranking"),
        "relationship_search.graph_search_ranking algorithm ref",
    );
    let traversal = required(
        wendaograph_algorithm_ref("relationship_search.large_object_graph_traversal"),
        "relationship_search.large_object_graph_traversal algorithm ref",
    );
    let snapshot = required(
        wendaograph_algorithm_ref("relationship_search.graph_snapshot_traversal"),
        "relationship_search.graph_snapshot_traversal algorithm ref",
    );

    assert_eq!(references.len(), 10);
    assert!(references.iter().all(|reference| {
        reference.family == "relationship_search"
            && reference.profile_id == WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID
    }));
    assert_eq!(hnsw.julia_entrypoint, "WendaoGraph.hnsw_neighbor_rows");
    assert_eq!(hnsw.output_table, Some("semantic_neighbors"));
    assert!(hnsw.is_heavy());
    assert_eq!(moc.julia_entrypoint, "WendaoGraph.topology_community_rows");
    assert_eq!(
        ppr.julia_entrypoint,
        "WendaoGraph.multi_plane_diffusion_scores"
    );
    assert_eq!(ranking.julia_entrypoint, "WendaoGraph.link_frontier_rows");
    assert_eq!(traversal.julia_entrypoint, "WendaoGraph.sparse_adjacency");
    assert_eq!(traversal.output_table, None);
    assert_eq!(
        snapshot.julia_entrypoint,
        "WendaoGraph.build_graph_snapshot"
    );
    assert_eq!(snapshot.output_table, None);
}

#[test]
fn wendaograph_algorithm_catalog_marks_heavy_julia_helpers() {
    let core = required(
        wendaograph_algorithm_ref("link_graph.topology_core"),
        "link_graph.topology_core algorithm ref",
    );
    let diffusion = required(
        wendaograph_algorithm_ref("link_graph.diffusion_scores"),
        "link_graph.diffusion_scores algorithm ref",
    );
    let gnn = required(
        wendaograph_algorithm_ref("gnn.node_scores"),
        "gnn.node_scores algorithm ref",
    );
    let trace = required(
        wendaograph_algorithm_ref("page_index.disclosure_trace"),
        "page_index.disclosure_trace algorithm ref",
    );
    let transition = required(
        wendaograph_algorithm_ref("search_strategy_flow.transition_rows"),
        "search_strategy_flow.transition_rows algorithm ref",
    );

    assert!(core.is_heavy());
    assert_eq!(core.output_table, Some("topology_core"));
    assert_eq!(core.julia_entrypoint, "WendaoGraph.topology_core_rows");
    assert!(diffusion.is_heavy());
    assert_eq!(
        diffusion.julia_entrypoint,
        "WendaoGraph.multi_plane_diffusion_scores"
    );
    assert!(gnn.is_heavy());
    assert_eq!(gnn.profile_id, WENDAO_GRAPH_GNN_REASONING_PROFILE_ID);
    assert_eq!(trace.complexity, JuliaTaskComplexityClass::Simple);
    assert_eq!(transition.complexity, JuliaTaskComplexityClass::Simple);
}
