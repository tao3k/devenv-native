use std::env;

use super::{
    WENDAOGRAPH_PACKAGE_DIR_ENV, WendaoGraphGnnBackendLoadDiagnostics,
    WendaoGraphGnnHostProbeReport, parse_gnn_probe_report_line, probe_wendaograph_gnn_host_request,
};

const RUN_WENDAOGRAPH_GNN_HOST_PROBE_TEST_ENV: &str = "RUN_WENDAOGRAPH_GNN_HOST_PROBE_TEST";

#[test]
fn gnn_host_probe_report_parser_accepts_compact_metric_line() {
    let report = parse_gnn_probe_report_line(
        "wendaograph_gnn_host_probe sample_count=3 first_ms=12.5 warm_min_ms=2.1 warm_median_ms=2.2 warm_p95_ms=2.4 warm_max_ms=2.5 node_count=4 edge_count=4 feature_rows=7 feature_cols=4 score_count=4 frontier_rows=3 metal_loaded=true cuda_loaded=false amdgpu_loaded=false metal_functional=true metal_score_count=4",
    )
    .unwrap_or_else(|error| panic!("parse GNN host probe report: {error}"));

    assert_eq!(
        report,
        WendaoGraphGnnHostProbeReport {
            sample_count: 3,
            first_ms: 12.5,
            warm_min_ms: 2.1,
            warm_median_ms: 2.2,
            warm_p95_ms: 2.4,
            warm_max_ms: 2.5,
            node_count: 4,
            edge_count: 4,
            feature_rows: 7,
            feature_cols: 4,
            score_count: 4,
            frontier_rows: 3,
            backend_load: WendaoGraphGnnBackendLoadDiagnostics {
                metal_loaded: true,
                cuda_loaded: false,
                amdgpu_loaded: false,
            },
            metal_functional: true,
            metal_score_count: 4,
        }
    );
}

#[test]
fn gnn_host_probe_report_parser_rejects_missing_fields() {
    let error =
        parse_gnn_probe_report_line("wendaograph_gnn_host_probe sample_count=3 first_ms=12.5")
            .expect_err("missing GNN shape fields must fail");

    assert!(error.contains("warm_min_ms"));
}

#[test]
fn gnn_host_probe_report_parser_rejects_invalid_bool() {
    let error = parse_gnn_probe_report_line(
        "wendaograph_gnn_host_probe sample_count=3 first_ms=12.5 warm_min_ms=2.1 warm_median_ms=2.2 warm_p95_ms=2.4 warm_max_ms=2.5 node_count=4 edge_count=4 feature_rows=7 feature_cols=4 score_count=4 frontier_rows=3 metal_loaded=yes cuda_loaded=false amdgpu_loaded=false metal_functional=false metal_score_count=0",
    )
    .expect_err("invalid bool fields must fail");

    assert!(error.contains("metal_loaded"));
}

#[test]
fn wendaograph_gnn_host_probe_runs_real_julia_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_GNN_HOST_PROBE_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph GNN host probe; set {RUN_WENDAOGRAPH_GNN_HOST_PROBE_TEST_ENV}=1 and {WENDAOGRAPH_PACKAGE_DIR_ENV}"
        );
        return;
    }

    let report = probe_wendaograph_gnn_host_request(2)
        .unwrap_or_else(|error| panic!("run real WendaoGraph GNN host probe: {error}"));

    assert_eq!(report.sample_count, 2);
    assert_eq!(report.node_count, 4);
    assert_eq!(report.edge_count, 4);
    assert_eq!(report.feature_rows, 7);
    assert_eq!(report.feature_cols, 4);
    assert_eq!(report.score_count, 4);
    assert_eq!(report.frontier_rows, 3);
    if report.metal_functional {
        assert!(report.backend_load.metal_loaded);
        assert_eq!(report.metal_score_count, report.score_count);
    } else {
        assert_eq!(report.metal_score_count, 0);
    }
    assert!(report.first_ms >= 0.0);
    assert!(report.warm_min_ms >= 0.0);
    assert!(report.warm_median_ms >= report.warm_min_ms);
    assert!(report.warm_p95_ms >= report.warm_median_ms);
    assert!(report.warm_max_ms >= report.warm_p95_ms);

    eprintln!(
        "wendaograph_gnn_host_probe_summary sample_count={} first_ms={:.3} warm_median_ms={:.3} warm_p95_ms={:.3} warm_max_ms={:.3} node_count={} edge_count={} feature_shape={}x{} score_count={} frontier_rows={} metal_loaded={} metal_functional={} metal_score_count={}",
        report.sample_count,
        report.first_ms,
        report.warm_median_ms,
        report.warm_p95_ms,
        report.warm_max_ms,
        report.node_count,
        report.edge_count,
        report.feature_rows,
        report.feature_cols,
        report.score_count,
        report.frontier_rows,
        report.backend_load.metal_loaded,
        report.metal_functional,
        report.metal_score_count,
    );
}
