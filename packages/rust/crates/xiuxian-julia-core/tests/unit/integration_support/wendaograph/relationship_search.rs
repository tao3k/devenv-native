use std::path::{Path, PathBuf};
use std::{env, fs};

use super::{
    WENDAOGRAPH_PACKAGE_DIR_ENV, WendaoGraphLinkGraphFullStructuralHostProbeReport,
    parse_link_graph_full_structural_probe_report_line,
    probe_wendaograph_link_graph_full_structural_host_request,
};
use crate::polyglot::{
    JuliaProfileSchedulingFacts, WendaoGraphAlgorithmWorkload,
    WendaoGraphRelationshipSearchEvidence, wendaograph_relationship_search_algorithm_refs,
    wendaograph_relationship_search_evidence_from_full_structural_host_probe,
};
use xiuxian_polyglot_orchestrator::{BenchmarkState, JuliaRuntimeStats, JuliaScheduleAction};

const RUN_WENDAOGRAPH_RELATIONSHIP_SEARCH_LIVE_PERF_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_RELATIONSHIP_SEARCH_LIVE_PERF_TEST";
const RUN_WENDAOGRAPH_RELATIONSHIP_SEARCH_SYNTHETIC_STABILITY_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_RELATIONSHIP_SEARCH_SYNTHETIC_STABILITY_TEST";
const WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE_ENV: &str = "WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_RUNS_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_RUNS";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_RECEIPT_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_RECEIPT";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_REQUIRE_CANDIDATE_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_REQUIRE_CANDIDATE";
const RELATIONSHIP_SEARCH_STABILITY_RECEIPT_SCHEMA: &str =
    "xiuxian_wendao.wendaograph.relationship_search.synthetic_stability_receipt.v1";
const RELATIONSHIP_SEARCH_PROMOTION_P95_THRESHOLD_MS: u32 = 15;
const RELATIONSHIP_SEARCH_PROMOTION_SPREAD_THRESHOLD: f64 = 4.0;

#[derive(Clone, Debug, PartialEq)]
struct RelationshipSearchStabilitySummary {
    run_count: usize,
    algorithm_count: usize,
    evidence_row_count: usize,
    dispatch_count: usize,
    queue_count: usize,
    fallback_count: usize,
    reject_count: usize,
    latency_p50_ms: u32,
    latency_p95_ms: u32,
    warm_max_ms: f64,
    warm_spread_ratio: f64,
    max_selected_batch_size: u32,
    min_node_count: usize,
    max_node_count: usize,
    min_edge_count: usize,
    max_edge_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RelationshipSearchPromotionGateStatus {
    Candidate,
    Reject,
}

#[derive(Clone, Debug, PartialEq)]
struct RelationshipSearchPromotionGateDecision {
    status: RelationshipSearchPromotionGateStatus,
    reason: String,
    latency_p95_ms: u32,
    warm_spread_ratio: f64,
    expected_evidence_rows: usize,
    actual_evidence_rows: usize,
}

#[test]
fn relationship_search_stability_summary_counts_repeated_runs() {
    let reports = vec![
        synthetic_full_structural_report(128, 512, 64, 2.0, 3.0, 4.0),
        synthetic_full_structural_report(128, 512, 64, 4.0, 5.0, 8.0),
    ];
    let evidence_runs: Vec<Vec<WendaoGraphRelationshipSearchEvidence>> = reports
        .iter()
        .map(relationship_search_evidence_for_report)
        .collect();

    let summary = relationship_search_stability_summary(&reports, &evidence_runs)
        .unwrap_or_else(|error| panic!("summarize relationship-search stability: {error}"));

    assert_eq!(
        summary,
        RelationshipSearchStabilitySummary {
            run_count: 2,
            algorithm_count: 10,
            evidence_row_count: 20,
            dispatch_count: 20,
            queue_count: 0,
            fallback_count: 0,
            reject_count: 0,
            latency_p50_ms: 2,
            latency_p95_ms: 5,
            warm_max_ms: 8.0,
            warm_spread_ratio: 4.0,
            max_selected_batch_size: 4,
            min_node_count: 128,
            max_node_count: 128,
            min_edge_count: 512,
            max_edge_count: 512,
        }
    );
}

#[test]
fn relationship_search_stability_receipt_writes_json() {
    let reports = vec![
        synthetic_full_structural_report(128, 512, 64, 2.0, 3.0, 4.0),
        synthetic_full_structural_report(128, 512, 64, 4.0, 5.0, 8.0),
    ];
    let evidence_runs: Vec<Vec<WendaoGraphRelationshipSearchEvidence>> = reports
        .iter()
        .map(relationship_search_evidence_for_report)
        .collect();
    let summary = relationship_search_stability_summary(&reports, &evidence_runs)
        .unwrap_or_else(|error| panic!("summarize relationship-search stability: {error}"));
    let temp_dir =
        tempfile::tempdir().unwrap_or_else(|error| panic!("create temp receipt dir: {error}"));
    let receipt_path = temp_dir.path().join("synthetic_stability_receipt.json");

    write_relationship_search_stability_receipt(&summary, "synthetic-large", &receipt_path)
        .unwrap_or_else(|error| panic!("write stability receipt: {error}"));

    let receipt = fs::read_to_string(&receipt_path)
        .unwrap_or_else(|error| panic!("read stability receipt: {error}"));
    let receipt: serde_json::Value = serde_json::from_str(&receipt)
        .unwrap_or_else(|error| panic!("parse stability receipt: {error}"));
    assert_eq!(
        receipt["schema"].as_str(),
        Some(RELATIONSHIP_SEARCH_STABILITY_RECEIPT_SCHEMA)
    );
    assert_eq!(
        receipt["workload"]["mode"].as_str(),
        Some("synthetic-large")
    );
    assert_eq!(receipt["summary"]["run_count"].as_u64(), Some(2));
    assert_eq!(receipt["summary"]["dispatch_count"].as_u64(), Some(20));
    assert_eq!(receipt["summary"]["latency_p95_ms"].as_u64(), Some(5));
    assert_eq!(receipt["graph"]["max_edge_count"].as_u64(), Some(512));
}

#[test]
fn relationship_search_promotion_gate_accepts_candidate_receipt() {
    let summary = RelationshipSearchStabilitySummary {
        run_count: 2,
        algorithm_count: 10,
        evidence_row_count: 20,
        dispatch_count: 20,
        queue_count: 0,
        fallback_count: 0,
        reject_count: 0,
        latency_p50_ms: 4,
        latency_p95_ms: 7,
        warm_max_ms: 6.519,
        warm_spread_ratio: 2.015,
        max_selected_batch_size: 4,
        min_node_count: 128,
        max_node_count: 128,
        min_edge_count: 512,
        max_edge_count: 512,
    };
    let temp_dir =
        tempfile::tempdir().unwrap_or_else(|error| panic!("create temp receipt dir: {error}"));
    let receipt_path = temp_dir.path().join("candidate_receipt.json");
    write_relationship_search_stability_receipt(&summary, "synthetic-large", &receipt_path)
        .unwrap_or_else(|error| panic!("write candidate receipt: {error}"));

    let (_mode, summary) = read_relationship_search_stability_receipt(&receipt_path)
        .unwrap_or_else(|error| panic!("read candidate receipt: {error}"));
    let decision = relationship_search_promotion_gate_decision(&summary);

    assert_eq!(
        decision.status,
        RelationshipSearchPromotionGateStatus::Candidate
    );
    assert_eq!(decision.reason, "candidate");
    assert_eq!(decision.expected_evidence_rows, 20);
    assert_eq!(decision.actual_evidence_rows, 20);
}

#[test]
fn relationship_search_promotion_gate_rejects_unstable_receipts() {
    let mut summary = RelationshipSearchStabilitySummary {
        run_count: 2,
        algorithm_count: 10,
        evidence_row_count: 20,
        dispatch_count: 20,
        queue_count: 0,
        fallback_count: 0,
        reject_count: 0,
        latency_p50_ms: 4,
        latency_p95_ms: 30,
        warm_max_ms: 40.0,
        warm_spread_ratio: 8.0,
        max_selected_batch_size: 4,
        min_node_count: 128,
        max_node_count: 128,
        min_edge_count: 512,
        max_edge_count: 512,
    };

    let latency_decision = relationship_search_promotion_gate_decision(&summary);
    assert_eq!(
        latency_decision.status,
        RelationshipSearchPromotionGateStatus::Reject
    );
    assert_eq!(latency_decision.reason, "latency-p95-exceeds-threshold");

    summary.latency_p95_ms = 7;
    let spread_decision = relationship_search_promotion_gate_decision(&summary);
    assert_eq!(spread_decision.reason, "spread-exceeds-threshold");

    summary.warm_spread_ratio = 2.0;
    summary.dispatch_count = 19;
    summary.fallback_count = 1;
    let action_decision = relationship_search_promotion_gate_decision(&summary);
    assert_eq!(action_decision.reason, "non-dispatch-actions");
}

#[test]
fn wendaograph_relationship_search_synthetic_stability_runs_real_julia_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_RELATIONSHIP_SEARCH_SYNTHETIC_STABILITY_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph relationship-search synthetic stability; set {RUN_WENDAOGRAPH_RELATIONSHIP_SEARCH_SYNTHETIC_STABILITY_TEST_ENV}=1, {WENDAOGRAPH_PACKAGE_DIR_ENV}, and {WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE_ENV}=synthetic-large"
        );
        return;
    }

    let mode = env::var(WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE_ENV).unwrap_or_default();
    assert_eq!(
        mode, "synthetic-large",
        "synthetic stability proof requires {WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE_ENV}=synthetic-large"
    );
    let run_count = env::var(WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_RUNS_ENV)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let mut reports = Vec::with_capacity(run_count);
    let mut evidence_runs = Vec::with_capacity(run_count);

    for run_index in 1..=run_count {
        let report =
            probe_wendaograph_link_graph_full_structural_host_request(3).unwrap_or_else(|error| {
                panic!("run real WendaoGraph synthetic stability probe {run_index}: {error}")
            });
        let evidence = relationship_search_evidence_for_report(&report);
        assert_eq!(
            evidence.len(),
            wendaograph_relationship_search_algorithm_refs().len()
        );
        assert!(evidence.iter().all(|row| row.probe_table.is_some()));
        assert!(evidence.iter().all(|row| row.probe_rows.is_some()));
        eprintln!(
            "wendaograph_relationship_search_synthetic_stability_run index={} mode={} node_count={} edge_count={} semantic_neighbor_count={} warm_median_ms={:.3} warm_p95_ms={:.3} warm_max_ms={:.3} dispatch_count={}",
            run_index,
            report.base.mode,
            report.base.node_count,
            report.base.edge_count,
            report.base.semantic_neighbor_count,
            report.base.warm_median_ms,
            report.base.warm_p95_ms,
            report.base.warm_max_ms,
            evidence
                .iter()
                .filter(|row| row.schedule_plan.action == JuliaScheduleAction::Dispatch)
                .count()
        );
        reports.push(report);
        evidence_runs.push(evidence);
    }

    let summary = relationship_search_stability_summary(&reports, &evidence_runs)
        .unwrap_or_else(|error| panic!("summarize real synthetic stability proof: {error}"));
    assert_eq!(summary.run_count, run_count);
    assert_eq!(
        summary.algorithm_count,
        wendaograph_relationship_search_algorithm_refs().len()
    );
    assert_eq!(
        summary.dispatch_count,
        run_count * wendaograph_relationship_search_algorithm_refs().len()
    );
    assert_eq!(summary.fallback_count, 0);
    assert_eq!(summary.reject_count, 0);
    let receipt_path = relationship_search_stability_receipt_path();
    write_relationship_search_stability_receipt(&summary, &reports[0].base.mode, &receipt_path)
        .unwrap_or_else(|error| panic!("write real synthetic stability receipt: {error}"));
    let receipt = fs::read_to_string(&receipt_path)
        .unwrap_or_else(|error| panic!("read real synthetic stability receipt: {error}"));
    assert!(receipt.contains(RELATIONSHIP_SEARCH_STABILITY_RECEIPT_SCHEMA));
    let (_receipt_mode, receipt_summary) =
        read_relationship_search_stability_receipt(&receipt_path)
            .unwrap_or_else(|error| panic!("read real stability receipt for gate: {error}"));
    let gate_decision = relationship_search_promotion_gate_decision(&receipt_summary);
    if env::var_os(WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_REQUIRE_CANDIDATE_ENV).is_some() {
        assert_eq!(
            gate_decision.status,
            RelationshipSearchPromotionGateStatus::Candidate
        );
    }
    eprintln!(
        "wendaograph_relationship_search_synthetic_stability_summary run_count={} algorithm_count={} evidence_row_count={} dispatch_count={} queue_count={} fallback_count={} reject_count={} latency_p50_ms={} latency_p95_ms={} warm_max_ms={:.3} warm_spread_ratio={:.3} max_selected_batch_size={} min_node_count={} max_node_count={} min_edge_count={} max_edge_count={} gate_status={:?} gate_reason={} receipt_path={}",
        summary.run_count,
        summary.algorithm_count,
        summary.evidence_row_count,
        summary.dispatch_count,
        summary.queue_count,
        summary.fallback_count,
        summary.reject_count,
        summary.latency_p50_ms,
        summary.latency_p95_ms,
        summary.warm_max_ms,
        summary.warm_spread_ratio,
        summary.max_selected_batch_size,
        summary.min_node_count,
        summary.max_node_count,
        summary.min_edge_count,
        summary.max_edge_count,
        gate_decision.status,
        gate_decision.reason,
        receipt_path.display()
    );
}

#[test]
fn wendaograph_relationship_search_live_perf_runs_real_julia_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_RELATIONSHIP_SEARCH_LIVE_PERF_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph relationship-search live perf; set {RUN_WENDAOGRAPH_RELATIONSHIP_SEARCH_LIVE_PERF_TEST_ENV}=1 and {WENDAOGRAPH_PACKAGE_DIR_ENV}"
        );
        return;
    }

    let report =
        probe_wendaograph_link_graph_full_structural_host_request(3).unwrap_or_else(|error| {
            panic!("run real WendaoGraph relationship-search live perf probe: {error}")
        });
    let evidence = wendaograph_relationship_search_evidence_from_full_structural_host_probe(
        &report,
        relationship_search_workload_from_report(&report),
        JuliaProfileSchedulingFacts::new(
            JuliaRuntimeStats::new().with_benchmark(BenchmarkState::WithinThreshold),
        )
        .with_max_in_flight(Some(4))
        .with_target_latency_ms(Some(50)),
    );

    assert_eq!(
        evidence.len(),
        wendaograph_relationship_search_algorithm_refs().len()
    );
    assert!(evidence.iter().all(|row| row.probe_table.is_some()));
    assert!(evidence.iter().all(|row| row.probe_rows.is_some()));

    eprintln!(
        "wendaograph_relationship_search_live_perf_input mode={} node_count={} edge_count={} semantic_neighbor_count={} graph_metric_rows={} semantic_overlay_rows={} diffusion_rows={} frontier_rows={}",
        report.base.mode,
        report.base.node_count,
        report.base.edge_count,
        report.base.semantic_neighbor_count,
        report.base.graph_metric_rows,
        report.base.semantic_overlay_rows,
        report.base.diffusion_rows,
        report.base.frontier_rows
    );

    for row in &evidence {
        eprintln!(
            "wendaograph_relationship_search_live_perf algorithm_id={} probe_table={} probe_rows={} p50_ms={} p95_ms={} action={:?} confidence={} batch_size={}",
            row.algorithm.algorithm_id,
            row.probe_table.unwrap_or("none"),
            row.probe_rows.unwrap_or(0),
            row.runtime_stats.p50_latency_ms.unwrap_or(0),
            row.runtime_stats.p95_latency_ms.unwrap_or(0),
            row.schedule_plan.action,
            row.schedule_plan.confidence_score,
            row.schedule_plan.selected_batch_size
        );
    }
}

fn relationship_search_workload_from_report(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
) -> WendaoGraphAlgorithmWorkload {
    WendaoGraphAlgorithmWorkload::new()
        .with_rows(relationship_search_observed_rows(report))
        .with_graph_size(
            saturating_usize_to_u32(report.base.node_count),
            saturating_usize_to_u32(report.base.edge_count),
        )
        .with_feature_columns(6)
        .with_byte_size(relationship_search_estimated_bytes(report))
}

fn relationship_search_evidence_for_report(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
) -> Vec<WendaoGraphRelationshipSearchEvidence> {
    wendaograph_relationship_search_evidence_from_full_structural_host_probe(
        report,
        relationship_search_workload_from_report(report),
        JuliaProfileSchedulingFacts::new(
            JuliaRuntimeStats::new().with_benchmark(BenchmarkState::WithinThreshold),
        )
        .with_max_in_flight(Some(4))
        .with_target_latency_ms(Some(50)),
    )
}

fn relationship_search_stability_summary(
    reports: &[WendaoGraphLinkGraphFullStructuralHostProbeReport],
    evidence_runs: &[Vec<WendaoGraphRelationshipSearchEvidence>],
) -> Result<RelationshipSearchStabilitySummary, String> {
    if reports.is_empty() {
        return Err("missing stability reports".to_owned());
    }
    if reports.len() != evidence_runs.len() {
        return Err(format!(
            "stability report/run mismatch: reports={} evidence_runs={}",
            reports.len(),
            evidence_runs.len()
        ));
    }
    let algorithm_count = evidence_runs
        .first()
        .map(Vec::len)
        .ok_or_else(|| "missing stability evidence runs".to_owned())?;
    if algorithm_count == 0 {
        return Err("empty stability evidence run".to_owned());
    }

    let mut dispatch_count = 0;
    let mut queue_count = 0;
    let mut fallback_count = 0;
    let mut reject_count = 0;
    let mut p50_samples = Vec::with_capacity(reports.len() * algorithm_count);
    let mut p95_samples = Vec::with_capacity(reports.len() * algorithm_count);
    let mut max_selected_batch_size = 0;

    for evidence in evidence_runs {
        if evidence.len() != algorithm_count {
            return Err(format!(
                "stability algorithm-count mismatch: expected={} actual={}",
                algorithm_count,
                evidence.len()
            ));
        }
        for row in evidence {
            match row.schedule_plan.action {
                JuliaScheduleAction::Dispatch => dispatch_count += 1,
                JuliaScheduleAction::Queue => queue_count += 1,
                JuliaScheduleAction::Fallback => fallback_count += 1,
                JuliaScheduleAction::Reject => reject_count += 1,
            }
            p50_samples.push(row.runtime_stats.p50_latency_ms.unwrap_or(0));
            p95_samples.push(row.runtime_stats.p95_latency_ms.unwrap_or(0));
            max_selected_batch_size =
                max_selected_batch_size.max(row.schedule_plan.selected_batch_size);
        }
    }

    let warm_min_ms = reports
        .iter()
        .map(|report| report.base.warm_min_ms)
        .filter(|value| value.is_finite() && *value > 0.0)
        .fold(f64::INFINITY, f64::min);
    let warm_max_ms = reports
        .iter()
        .map(|report| report.base.warm_max_ms)
        .filter(|value| value.is_finite())
        .fold(0.0, f64::max);
    let warm_spread_ratio = if warm_min_ms.is_finite() && warm_min_ms > 0.0 {
        warm_max_ms / warm_min_ms
    } else {
        0.0
    };

    Ok(RelationshipSearchStabilitySummary {
        run_count: reports.len(),
        algorithm_count,
        evidence_row_count: evidence_runs.iter().map(Vec::len).sum(),
        dispatch_count,
        queue_count,
        fallback_count,
        reject_count,
        latency_p50_ms: percentile_u32(&mut p50_samples, 500),
        latency_p95_ms: percentile_u32(&mut p95_samples, 950),
        warm_max_ms,
        warm_spread_ratio,
        max_selected_batch_size,
        min_node_count: reports
            .iter()
            .map(|report| report.base.node_count)
            .min()
            .unwrap_or(0),
        max_node_count: reports
            .iter()
            .map(|report| report.base.node_count)
            .max()
            .unwrap_or(0),
        min_edge_count: reports
            .iter()
            .map(|report| report.base.edge_count)
            .min()
            .unwrap_or(0),
        max_edge_count: reports
            .iter()
            .map(|report| report.base.edge_count)
            .max()
            .unwrap_or(0),
    })
}

fn relationship_search_stability_receipt_path() -> PathBuf {
    if let Some(configured) = env::var_os(WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_STABILITY_RECEIPT_ENV) {
        return PathBuf::from(configured);
    }
    let cache_home =
        env::var_os("PRJ_CACHE_HOME").map_or_else(|| PathBuf::from(".cache"), PathBuf::from);
    cache_home
        .join("wendaograph")
        .join("relationship_search_synthetic_stability_receipt.json")
}

fn write_relationship_search_stability_receipt(
    summary: &RelationshipSearchStabilitySummary,
    mode: &str,
    receipt_path: &Path,
) -> Result<(), String> {
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create relationship-search stability receipt dir `{}`: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(
        receipt_path,
        relationship_search_stability_receipt_json(summary, mode),
    )
    .map_err(|error| {
        format!(
            "write relationship-search stability receipt `{}`: {error}",
            receipt_path.display()
        )
    })
}

fn relationship_search_stability_receipt_json(
    summary: &RelationshipSearchStabilitySummary,
    mode: &str,
) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": RELATIONSHIP_SEARCH_STABILITY_RECEIPT_SCHEMA,
        "workload": {
            "mode": mode,
            "min_node_count": summary.min_node_count,
            "max_node_count": summary.max_node_count,
            "min_edge_count": summary.min_edge_count,
            "max_edge_count": summary.max_edge_count,
        },
        "summary": {
            "run_count": summary.run_count,
            "algorithm_count": summary.algorithm_count,
            "evidence_row_count": summary.evidence_row_count,
            "dispatch_count": summary.dispatch_count,
            "queue_count": summary.queue_count,
            "fallback_count": summary.fallback_count,
            "reject_count": summary.reject_count,
            "latency_p50_ms": summary.latency_p50_ms,
            "latency_p95_ms": summary.latency_p95_ms,
            "warm_max_ms": summary.warm_max_ms,
            "warm_spread_ratio": summary.warm_spread_ratio,
            "max_selected_batch_size": summary.max_selected_batch_size,
        },
        "graph": {
            "min_node_count": summary.min_node_count,
            "max_node_count": summary.max_node_count,
            "min_edge_count": summary.min_edge_count,
            "max_edge_count": summary.max_edge_count,
        },
    }))
    .unwrap_or_else(|error| panic!("serialize relationship-search stability receipt: {error}"))
}

fn read_relationship_search_stability_receipt(
    receipt_path: &Path,
) -> Result<(String, RelationshipSearchStabilitySummary), String> {
    let receipt = fs::read_to_string(receipt_path).map_err(|error| {
        format!(
            "read relationship-search stability receipt `{}`: {error}",
            receipt_path.display()
        )
    })?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt).map_err(|error| {
        format!(
            "parse relationship-search stability receipt `{}`: {error}",
            receipt_path.display()
        )
    })?;
    let schema = json_string(&receipt, "schema")?;
    if schema != RELATIONSHIP_SEARCH_STABILITY_RECEIPT_SCHEMA {
        return Err(format!(
            "unsupported relationship-search stability receipt schema `{schema}`"
        ));
    }
    let workload = receipt
        .get("workload")
        .ok_or_else(|| "missing receipt `workload` object".to_owned())?;
    let summary = receipt
        .get("summary")
        .ok_or_else(|| "missing receipt `summary` object".to_owned())?;
    let graph = receipt
        .get("graph")
        .ok_or_else(|| "missing receipt `graph` object".to_owned())?;
    let mode = json_string(workload, "mode")?;
    Ok((
        mode,
        RelationshipSearchStabilitySummary {
            run_count: json_usize(summary, "run_count")?,
            algorithm_count: json_usize(summary, "algorithm_count")?,
            evidence_row_count: json_usize(summary, "evidence_row_count")?,
            dispatch_count: json_usize(summary, "dispatch_count")?,
            queue_count: json_usize(summary, "queue_count")?,
            fallback_count: json_usize(summary, "fallback_count")?,
            reject_count: json_usize(summary, "reject_count")?,
            latency_p50_ms: json_u32(summary, "latency_p50_ms")?,
            latency_p95_ms: json_u32(summary, "latency_p95_ms")?,
            warm_max_ms: json_f64(summary, "warm_max_ms")?,
            warm_spread_ratio: json_f64(summary, "warm_spread_ratio")?,
            max_selected_batch_size: json_u32(summary, "max_selected_batch_size")?,
            min_node_count: json_usize(graph, "min_node_count")?,
            max_node_count: json_usize(graph, "max_node_count")?,
            min_edge_count: json_usize(graph, "min_edge_count")?,
            max_edge_count: json_usize(graph, "max_edge_count")?,
        },
    ))
}

fn relationship_search_promotion_gate_decision(
    summary: &RelationshipSearchStabilitySummary,
) -> RelationshipSearchPromotionGateDecision {
    let expected_evidence_rows = summary.run_count.saturating_mul(summary.algorithm_count);
    let reject = |reason: &str| RelationshipSearchPromotionGateDecision {
        status: RelationshipSearchPromotionGateStatus::Reject,
        reason: reason.to_owned(),
        latency_p95_ms: summary.latency_p95_ms,
        warm_spread_ratio: summary.warm_spread_ratio,
        expected_evidence_rows,
        actual_evidence_rows: summary.evidence_row_count,
    };
    if summary.run_count == 0 {
        return reject("missing-runs");
    }
    if summary.algorithm_count != wendaograph_relationship_search_algorithm_refs().len() {
        return reject("algorithm-count-mismatch");
    }
    if summary.evidence_row_count != expected_evidence_rows {
        return reject("row-count-mismatch");
    }
    if summary.dispatch_count != expected_evidence_rows
        || summary.queue_count > 0
        || summary.fallback_count > 0
        || summary.reject_count > 0
    {
        return reject("non-dispatch-actions");
    }
    if summary.latency_p95_ms > RELATIONSHIP_SEARCH_PROMOTION_P95_THRESHOLD_MS {
        return reject("latency-p95-exceeds-threshold");
    }
    if summary.warm_spread_ratio > RELATIONSHIP_SEARCH_PROMOTION_SPREAD_THRESHOLD {
        return reject("spread-exceeds-threshold");
    }
    RelationshipSearchPromotionGateDecision {
        status: RelationshipSearchPromotionGateStatus::Candidate,
        reason: "candidate".to_owned(),
        latency_p95_ms: summary.latency_p95_ms,
        warm_spread_ratio: summary.warm_spread_ratio,
        expected_evidence_rows,
        actual_evidence_rows: summary.evidence_row_count,
    }
}

fn json_string(value: &serde_json::Value, key: &str) -> Result<String, String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing or invalid receipt string `{key}`"))
}

fn json_usize(value: &serde_json::Value, key: &str) -> Result<usize, String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| format!("missing or invalid receipt usize `{key}`"))
}

fn json_u32(value: &serde_json::Value, key: &str) -> Result<u32, String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| format!("missing or invalid receipt u32 `{key}`"))
}

fn json_f64(value: &serde_json::Value, key: &str) -> Result<f64, String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_f64)
        .filter(|value| value.is_finite())
        .ok_or_else(|| format!("missing or invalid receipt f64 `{key}`"))
}

fn percentile_u32(values: &mut [u32], percentile_per_mille: usize) -> u32 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let index = values
        .len()
        .saturating_mul(percentile_per_mille)
        .div_ceil(1000)
        .saturating_sub(1)
        .min(values.len() - 1);
    values[index]
}

fn synthetic_full_structural_report(
    node_count: usize,
    edge_count: usize,
    semantic_neighbor_count: usize,
    warm_median_ms: f64,
    warm_p95_ms: f64,
    warm_max_ms: f64,
) -> WendaoGraphLinkGraphFullStructuralHostProbeReport {
    parse_link_graph_full_structural_probe_report_line(
        format!(
            "wendaograph_link_graph_host_probe mode=synthetic-large node_count={node_count} edge_count={edge_count} semantic_neighbor_count={semantic_neighbor_count} sample_count=3 first_ms=12.5 warm_min_ms=2.0 warm_median_ms={warm_median_ms} warm_p95_ms={warm_p95_ms} warm_max_ms={warm_max_ms} graph_metric_rows={node_count} component_rows={node_count} topology_profile_rows={node_count} topology_candidate_rows=4 topology_bottleneck_rows={node_count} topology_community_rows={node_count} topology_cover_rows={node_count} topology_core_rows={node_count} topology_boundary_rows={node_count} topology_transition_rows=8 topology_gateway_rows={node_count} topology_community_summary_rows=8 topology_community_link_rows=7 topology_community_frontier_rows=2 semantic_overlay_rows={semantic_neighbor_count} diffusion_rows={node_count} frontier_rows=3"
        )
        .as_str(),
    )
    .unwrap_or_else(|error| panic!("parse synthetic full structural report: {error}"))
}

fn relationship_search_observed_rows(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
) -> u32 {
    saturating_usize_to_u32(
        report.base.semantic_overlay_rows
            + report.topology_community_rows
            + report.topology_community_link_rows
            + report.topology_community_frontier_rows
            + report.base.diffusion_rows
            + report.base.frontier_rows
            + report.base.topology_candidate_rows
            + report.component_rows
            + report.base.graph_metric_rows,
    )
    .max(1)
}

fn relationship_search_estimated_bytes(
    report: &WendaoGraphLinkGraphFullStructuralHostProbeReport,
) -> u64 {
    let node_bytes = saturating_usize_to_u64(report.base.node_count).saturating_mul(128);
    let edge_bytes = saturating_usize_to_u64(report.base.edge_count).saturating_mul(64);
    let row_bytes = u64::from(relationship_search_observed_rows(report)).saturating_mul(48);
    node_bytes
        .saturating_add(edge_bytes)
        .saturating_add(row_bytes)
}

fn saturating_usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn saturating_usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
