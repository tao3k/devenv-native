use super::{
    WendaoGraphLinkGraphFullStructuralHostProbeReport, WendaoGraphLinkGraphHostProbeReport,
    WendaoGraphPageIndexHostProbeReport, WendaoGraphPageIndexPlannerActionHostProbeReport,
    parse_link_graph_full_structural_probe_report_line, parse_link_graph_probe_report_line,
    parse_page_index_planner_action_probe_report_line, parse_page_index_probe_report_line,
};

#[test]
fn page_index_host_probe_report_parser_accepts_compact_metric_line() {
    let report = parse_page_index_probe_report_line(
        "wendaograph_page_index_host_probe sample_count=3 first_ms=10.5 warm_min_ms=1.1 warm_median_ms=1.2 warm_p95_ms=1.4 warm_max_ms=1.5 frontier_rows=1 trace_rows=1",
    )
    .unwrap_or_else(|error| panic!("parse host probe report: {error}"));

    assert_eq!(
        report,
        WendaoGraphPageIndexHostProbeReport {
            sample_count: 3,
            first_ms: 10.5,
            warm_min_ms: 1.1,
            warm_median_ms: 1.2,
            warm_p95_ms: 1.4,
            warm_max_ms: 1.5,
            frontier_rows: 1,
            trace_rows: 1,
        }
    );
}

#[test]
fn page_index_host_probe_report_parser_rejects_missing_fields() {
    let error = match parse_page_index_probe_report_line(
        "wendaograph_page_index_host_probe sample_count=3 first_ms=10.5",
    ) {
        Ok(report) => panic!("missing warm metric fields must fail, got {report:?}"),
        Err(error) => error,
    };

    assert!(error.contains("warm_min_ms"));
}

#[test]
fn page_index_planner_action_probe_report_parser_accepts_compact_metric_line() {
    let report = parse_page_index_planner_action_probe_report_line(
        "wendaograph_page_index_host_probe sample_count=3 first_ms=10.5 warm_min_ms=1.1 warm_median_ms=1.2 warm_p95_ms=1.4 warm_max_ms=1.5 frontier_rows=1 trace_rows=1 planner_action_rows=3 planner_expand_actions=1 planner_compare_actions=0 planner_jump_actions=1 planner_stop_actions=1",
    )
    .unwrap_or_else(|error| panic!("parse planner action host probe report: {error}"));

    assert_eq!(
        report,
        WendaoGraphPageIndexPlannerActionHostProbeReport {
            base: WendaoGraphPageIndexHostProbeReport {
                sample_count: 3,
                first_ms: 10.5,
                warm_min_ms: 1.1,
                warm_median_ms: 1.2,
                warm_p95_ms: 1.4,
                warm_max_ms: 1.5,
                frontier_rows: 1,
                trace_rows: 1,
            },
            planner_action_rows: 3,
            planner_expand_actions: 1,
            planner_compare_actions: 0,
            planner_jump_actions: 1,
            planner_stop_actions: 1,
        }
    );
}

#[test]
fn link_graph_host_probe_report_parser_accepts_compact_metric_line() {
    let report = parse_link_graph_probe_report_line(
        "wendaograph_link_graph_host_probe sample_count=3 first_ms=12.5 warm_min_ms=2.1 warm_median_ms=2.2 warm_p95_ms=2.4 warm_max_ms=2.5 graph_metric_rows=4 topology_candidate_rows=1 semantic_overlay_rows=2 diffusion_rows=4 frontier_rows=3",
    )
    .unwrap_or_else(|error| panic!("parse LinkGraph host probe report: {error}"));

    assert_eq!(
        report,
        WendaoGraphLinkGraphHostProbeReport {
            mode: "semantic-neighbors".into(),
            node_count: 4,
            edge_count: 2,
            semantic_neighbor_count: 1,
            sample_count: 3,
            first_ms: 12.5,
            warm_min_ms: 2.1,
            warm_median_ms: 2.2,
            warm_p95_ms: 2.4,
            warm_max_ms: 2.5,
            graph_metric_rows: 4,
            topology_candidate_rows: 1,
            semantic_overlay_rows: 2,
            diffusion_rows: 4,
            frontier_rows: 3,
        }
    );
}

#[test]
fn link_graph_host_probe_report_parser_accepts_synthetic_metric_line() {
    let report = parse_link_graph_probe_report_line(
        "wendaograph_link_graph_host_probe mode=synthetic-large node_count=128 edge_count=512 semantic_neighbor_count=64 sample_count=3 first_ms=12.5 warm_min_ms=2.1 warm_median_ms=2.2 warm_p95_ms=2.4 warm_max_ms=2.5 graph_metric_rows=128 topology_candidate_rows=8 semantic_overlay_rows=64 diffusion_rows=128 frontier_rows=9",
    )
    .unwrap_or_else(|error| panic!("parse synthetic LinkGraph host probe report: {error}"));

    assert_eq!(
        report,
        WendaoGraphLinkGraphHostProbeReport {
            mode: "synthetic-large".into(),
            node_count: 128,
            edge_count: 512,
            semantic_neighbor_count: 64,
            sample_count: 3,
            first_ms: 12.5,
            warm_min_ms: 2.1,
            warm_median_ms: 2.2,
            warm_p95_ms: 2.4,
            warm_max_ms: 2.5,
            graph_metric_rows: 128,
            topology_candidate_rows: 8,
            semantic_overlay_rows: 64,
            diffusion_rows: 128,
            frontier_rows: 9,
        }
    );
}

#[test]
fn link_graph_full_structural_probe_report_parser_accepts_compact_metric_line() {
    let report = parse_link_graph_full_structural_probe_report_line(
        "wendaograph_link_graph_host_probe sample_count=3 first_ms=12.5 warm_min_ms=2.1 warm_median_ms=2.2 warm_p95_ms=2.4 warm_max_ms=2.5 graph_metric_rows=4 component_rows=4 topology_profile_rows=4 topology_candidate_rows=1 topology_bottleneck_rows=4 topology_community_rows=4 topology_cover_rows=4 topology_core_rows=4 topology_boundary_rows=4 topology_transition_rows=2 topology_gateway_rows=4 topology_community_summary_rows=2 topology_community_link_rows=0 topology_community_frontier_rows=1 semantic_overlay_rows=2 diffusion_rows=4 frontier_rows=3",
    )
    .unwrap_or_else(|error| panic!("parse full structural LinkGraph probe report: {error}"));

    assert_eq!(
        report,
        WendaoGraphLinkGraphFullStructuralHostProbeReport {
            base: WendaoGraphLinkGraphHostProbeReport {
                mode: "semantic-neighbors".into(),
                node_count: 4,
                edge_count: 2,
                semantic_neighbor_count: 1,
                sample_count: 3,
                first_ms: 12.5,
                warm_min_ms: 2.1,
                warm_median_ms: 2.2,
                warm_p95_ms: 2.4,
                warm_max_ms: 2.5,
                graph_metric_rows: 4,
                topology_candidate_rows: 1,
                semantic_overlay_rows: 2,
                diffusion_rows: 4,
                frontier_rows: 3,
            },
            component_rows: 4,
            topology_profile_rows: 4,
            topology_bottleneck_rows: 4,
            topology_community_rows: 4,
            topology_cover_rows: 4,
            topology_core_rows: 4,
            topology_boundary_rows: 4,
            topology_transition_rows: 2,
            topology_gateway_rows: 4,
            topology_community_summary_rows: 2,
            topology_community_link_rows: 0,
            topology_community_frontier_rows: 1,
        }
    );
}
