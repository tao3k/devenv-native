use std::env;

use super::{
    WENDAOGRAPH_PACKAGE_DIR_ENV, WendaoGraphLinkGraphFullStructuralHostProbeReport,
    WendaoGraphLinkGraphHostProbeReport, WendaoGraphPageIndexHostProbeReport,
    WendaoGraphPageIndexPlannerActionHostProbeReport,
    parse_link_graph_full_structural_probe_report_line, parse_link_graph_probe_report_line,
    parse_page_index_planner_action_probe_report_line, parse_page_index_probe_report_line,
    probe_wendaograph_link_graph_full_structural_host_request,
    probe_wendaograph_page_index_host_request,
    probe_wendaograph_page_index_planner_action_host_request,
};

#[path = "wendaograph/relationship_search.rs"]
mod relationship_search;

const RUN_WENDAOGRAPH_PAGE_INDEX_HOST_PROBE_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_PAGE_INDEX_HOST_PROBE_TEST";
const RUN_WENDAOGRAPH_LINK_GRAPH_HOST_PROBE_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_LINK_GRAPH_HOST_PROBE_TEST";
const WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS_ENV: &str =
    "WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS";

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
    let error = parse_page_index_probe_report_line(
        "wendaograph_page_index_host_probe sample_count=3 first_ms=10.5",
    )
    .expect_err("missing warm metric fields must fail");

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
            mode: "semantic-neighbors".to_owned(),
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
            mode: "synthetic-large".to_owned(),
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
                mode: "semantic-neighbors".to_owned(),
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

#[test]
fn wendaograph_page_index_host_probe_runs_real_julia_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_PAGE_INDEX_HOST_PROBE_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph PageIndex host probe; set {RUN_WENDAOGRAPH_PAGE_INDEX_HOST_PROBE_TEST_ENV}=1 and {WENDAOGRAPH_PACKAGE_DIR_ENV}"
        );
        return;
    }

    if env::var_os(WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS_ENV).is_some() {
        let report =
            probe_wendaograph_page_index_planner_action_host_request(2).unwrap_or_else(|error| {
                panic!("run real WendaoGraph PageIndex planner-action host probe: {error}")
            });

        assert_eq!(report.base.sample_count, 2);
        assert_eq!(report.base.frontier_rows, 1);
        assert_eq!(report.base.trace_rows, 1);
        assert_eq!(report.planner_action_rows, 3);
        assert_eq!(report.planner_expand_actions, 1);
        assert_eq!(report.planner_compare_actions, 0);
        assert_eq!(report.planner_jump_actions, 1);
        assert_eq!(report.planner_stop_actions, 1);
        eprintln!(
            "wendaograph_page_index_planner_action_host_probe_summary sample_count={} first_ms={:.3} warm_median_ms={:.3} warm_p95_ms={:.3} warm_max_ms={:.3} planner_action_rows={}",
            report.base.sample_count,
            report.base.first_ms,
            report.base.warm_median_ms,
            report.base.warm_p95_ms,
            report.base.warm_max_ms,
            report.planner_action_rows
        );
        return;
    }

    let report = probe_wendaograph_page_index_host_request(2)
        .unwrap_or_else(|error| panic!("run real WendaoGraph PageIndex host probe: {error}"));

    assert_eq!(report.sample_count, 2);
    assert_eq!(report.frontier_rows, 1);
    assert_eq!(report.trace_rows, 1);
    assert!(report.first_ms >= 0.0);
    assert!(report.warm_min_ms >= 0.0);
    assert!(report.warm_median_ms >= report.warm_min_ms);
    assert!(report.warm_p95_ms >= report.warm_median_ms);
    assert!(report.warm_max_ms >= report.warm_p95_ms);
    eprintln!(
        "wendaograph_page_index_host_probe_summary sample_count={} first_ms={:.3} warm_median_ms={:.3} warm_p95_ms={:.3} warm_max_ms={:.3}",
        report.sample_count,
        report.first_ms,
        report.warm_median_ms,
        report.warm_p95_ms,
        report.warm_max_ms
    );
}

#[test]
fn wendaograph_link_graph_host_probe_runs_real_julia_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_LINK_GRAPH_HOST_PROBE_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph LinkGraph host probe; set {RUN_WENDAOGRAPH_LINK_GRAPH_HOST_PROBE_TEST_ENV}=1 and {WENDAOGRAPH_PACKAGE_DIR_ENV}"
        );
        return;
    }

    let report = probe_wendaograph_link_graph_full_structural_host_request(2)
        .unwrap_or_else(|error| panic!("run real WendaoGraph LinkGraph host probe: {error}"));

    assert_eq!(report.base.mode, "semantic-neighbors");
    assert_eq!(report.base.node_count, 4);
    assert_eq!(report.base.edge_count, 2);
    assert_eq!(report.base.semantic_neighbor_count, 1);
    assert_eq!(report.base.sample_count, 2);
    assert_eq!(report.base.graph_metric_rows, 4);
    assert_eq!(report.component_rows, 4);
    assert_eq!(report.topology_profile_rows, 4);
    assert_eq!(report.base.topology_candidate_rows, 1);
    assert_eq!(report.topology_bottleneck_rows, 4);
    assert_eq!(report.topology_community_rows, 4);
    assert_eq!(report.topology_cover_rows, 4);
    assert_eq!(report.topology_core_rows, 4);
    assert_eq!(report.topology_boundary_rows, 4);
    assert_eq!(report.topology_transition_rows, 2);
    assert_eq!(report.topology_gateway_rows, 4);
    assert_eq!(report.topology_community_summary_rows, 2);
    assert_eq!(report.topology_community_link_rows, 0);
    assert_eq!(report.topology_community_frontier_rows, 1);
    assert_eq!(report.base.semantic_overlay_rows, 2);
    assert_eq!(report.base.diffusion_rows, 4);
    assert_eq!(report.base.frontier_rows, 3);
    assert!(report.base.first_ms >= 0.0);
    assert!(report.base.warm_min_ms >= 0.0);
    assert!(report.base.warm_median_ms >= report.base.warm_min_ms);
    assert!(report.base.warm_p95_ms >= report.base.warm_median_ms);
    assert!(report.base.warm_max_ms >= report.base.warm_p95_ms);
    eprintln!(
        "wendaograph_link_graph_host_probe_summary mode={} node_count={} edge_count={} semantic_neighbor_count={} sample_count={} first_ms={:.3} warm_median_ms={:.3} warm_p95_ms={:.3} warm_max_ms={:.3} graph_metric_rows={} component_rows={} topology_profile_rows={} topology_candidate_rows={} topology_bottleneck_rows={} topology_community_rows={} topology_cover_rows={} topology_core_rows={} topology_boundary_rows={} topology_transition_rows={} topology_gateway_rows={} topology_community_summary_rows={} topology_community_link_rows={} topology_community_frontier_rows={} semantic_overlay_rows={} diffusion_rows={} frontier_rows={}",
        report.base.mode,
        report.base.node_count,
        report.base.edge_count,
        report.base.semantic_neighbor_count,
        report.base.sample_count,
        report.base.first_ms,
        report.base.warm_median_ms,
        report.base.warm_p95_ms,
        report.base.warm_max_ms,
        report.base.graph_metric_rows,
        report.component_rows,
        report.topology_profile_rows,
        report.base.topology_candidate_rows,
        report.topology_bottleneck_rows,
        report.topology_community_rows,
        report.topology_cover_rows,
        report.topology_core_rows,
        report.topology_boundary_rows,
        report.topology_transition_rows,
        report.topology_gateway_rows,
        report.topology_community_summary_rows,
        report.topology_community_link_rows,
        report.topology_community_frontier_rows,
        report.base.semantic_overlay_rows,
        report.base.diffusion_rows,
        report.base.frontier_rows
    );
}
