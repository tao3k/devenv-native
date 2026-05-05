use std::path::Path;

use super::support::{PerfBudget, PerfReport, PerfRunConfig, assert_perf_budget, run_sync_budget};
use serial_test::file_serial;
use tempfile::tempdir;
use xiuxian_wendao::LinkGraphIndex;
use xiuxian_wendao::link_graph::perf_support::{
    decode_link_graph_arrow_core_stream_stats, encode_link_graph_arrow_core_streams,
};

use super::support::{build_link_graph_fixture, env_f64, env_u64, env_usize};

const SUITE: &str = "xiuxian-wendao/perf";
const COLD_CASE: &str = "local_duckdb_cache_cold_build_p95";
const MISS_CASE: &str = "local_duckdb_cache_miss_write_p95";
const HIT_CASE: &str = "local_duckdb_cache_hit_read_p95";
const ARROW_CORE_STREAM_CASE: &str = "local_duckdb_cache_arrow_core_stream_p95";
const DEFAULT_NODE_COUNT: usize = 2_048;
const DEFAULT_HUB_COUNT: usize = 32;

#[test]
#[file_serial(wendao_perf_gate)]
fn local_duckdb_cache_cold_miss_hit_profile_gate() -> Result<(), String> {
    let root = tempdir().map_err(|error| format!("create local DuckDB cache fixture: {error}"))?;
    let node_count = env_usize("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_NODES", DEFAULT_NODE_COUNT);
    let hub_count = env_usize("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_HUBS", DEFAULT_HUB_COUNT);
    let expected_note_count = node_count + hub_count;
    build_link_graph_fixture(root.path(), node_count, hub_count)?;
    let arrow_index = LinkGraphIndex::build_with_filters(root.path(), &[], &[])?;
    let arrow_expected_stats = arrow_index.stats();
    let arrow_sample_streams = encode_link_graph_arrow_core_streams(&arrow_index)?;
    let arrow_sample_stats = decode_link_graph_arrow_core_stream_stats(&arrow_sample_streams)?;

    let cache_dir = root.path().join(".cache/wendao/link_graph/perf");
    std::fs::create_dir_all(&cache_dir).map_err(|error| {
        format!(
            "create local DuckDB cache perf directory {}: {error}",
            cache_dir.display()
        )
    })?;
    let include_dirs = Vec::<String>::new();
    let excluded_dirs = Vec::<String>::new();
    let build_config = build_perf_config();
    let hit_config = hit_perf_config();

    let mut cold_report =
        run_sync_budget(SUITE, COLD_CASE, &build_config, || -> Result<(), String> {
            let index =
                LinkGraphIndex::build_with_filters(root.path(), &include_dirs, &excluded_dirs)?;
            assert_note_count(&index, expected_note_count)
        });
    annotate_report(&mut cold_report, node_count, hub_count, "cold_build");

    let mut miss_counter = 0_usize;
    let mut miss_report =
        run_sync_budget(SUITE, MISS_CASE, &build_config, || -> Result<(), String> {
            miss_counter = miss_counter.saturating_add(1);
            let cache_path = cache_dir.join(format!("miss-{miss_counter:04}.duckdb"));
            let (index, meta) = LinkGraphIndex::build_with_local_cache_path_with_meta(
                root.path(),
                &include_dirs,
                &excluded_dirs,
                cache_path.as_path(),
            )?;
            if meta.status != "miss" {
                return Err(format!(
                    "expected local DuckDB cache miss during miss/write profile, got {}",
                    meta.status
                ));
            }
            assert_note_count(&index, expected_note_count)
        });
    annotate_report(&mut miss_report, node_count, hub_count, "miss_write");

    let hit_cache_path = cache_dir.join("hit-profile.duckdb");
    prime_hit_cache(
        root.path(),
        &include_dirs,
        &excluded_dirs,
        hit_cache_path.as_path(),
        expected_note_count,
    )?;
    let mut hit_report = run_sync_budget(SUITE, HIT_CASE, &hit_config, || -> Result<(), String> {
        let (index, meta) = LinkGraphIndex::build_with_local_cache_path_with_meta(
            root.path(),
            &include_dirs,
            &excluded_dirs,
            hit_cache_path.as_path(),
        )?;
        if meta.status != "hit" {
            return Err(format!(
                "expected local DuckDB cache hit during hit/read profile, got {}",
                meta.status
            ));
        }
        assert_note_count(&index, expected_note_count)
    });
    annotate_report(&mut hit_report, node_count, hub_count, "hit_read");
    let hit_cold_ratio = p95_ratio(&hit_report, &cold_report);
    hit_report.add_metadata("hit_vs_cold_ratio", format!("{hit_cold_ratio:.3}"));

    let mut arrow_report = run_sync_budget(
        SUITE,
        ARROW_CORE_STREAM_CASE,
        &arrow_perf_config(),
        || -> Result<(), String> {
            let streams = encode_link_graph_arrow_core_streams(&arrow_index)?;
            let stats = decode_link_graph_arrow_core_stream_stats(&streams)?;
            if stats.doc_count != arrow_expected_stats.total_notes {
                return Err(format!(
                    "expected {} Arrow document rows, got {}",
                    arrow_expected_stats.total_notes, stats.doc_count
                ));
            }
            if stats.edge_count != arrow_expected_stats.links_in_graph {
                return Err(format!(
                    "expected {} Arrow edge rows, got {}",
                    arrow_expected_stats.links_in_graph, stats.edge_count
                ));
            }
            if stats.alias_count == 0 {
                return Err("expected non-empty Arrow alias stream".to_string());
            }
            Ok(())
        },
    );
    annotate_report(
        &mut arrow_report,
        node_count,
        hub_count,
        "arrow_core_stream",
    );
    arrow_report.add_metadata("arrow_doc_rows", arrow_sample_stats.doc_count.to_string());
    arrow_report.add_metadata("arrow_edge_rows", arrow_sample_stats.edge_count.to_string());
    arrow_report.add_metadata(
        "arrow_alias_rows",
        arrow_sample_stats.alias_count.to_string(),
    );
    arrow_report.add_metadata(
        "arrow_total_bytes",
        arrow_sample_stats.total_bytes.to_string(),
    );

    assert_perf_budget(
        &cold_report,
        &PerfBudget {
            max_error_rate: Some(0.0),
            ..PerfBudget::new()
        },
    );
    assert_perf_budget(
        &miss_report,
        &PerfBudget {
            max_error_rate: Some(0.0),
            ..PerfBudget::new()
        },
    );
    assert_perf_budget(&hit_report, &hit_budget());
    assert_hit_cold_ratio(hit_cold_ratio)?;
    assert_perf_budget(
        &arrow_report,
        &PerfBudget {
            max_error_rate: Some(0.0),
            ..PerfBudget::new()
        },
    );
    println!(
        "local_duckdb_cache_perf_gate: notes={}, cold_p95_ms={:.3}, miss_write_p95_ms={:.3}, hit_read_p95_ms={:.3}, arrow_core_stream_p95_ms={:.3}, arrow_total_bytes={}, hit_vs_cold_ratio={:.3}, cold_report={:?}, miss_report={:?}, hit_report={:?}, arrow_report={:?}",
        expected_note_count,
        cold_report.quantiles.p95_ms,
        miss_report.quantiles.p95_ms,
        hit_report.quantiles.p95_ms,
        arrow_report.quantiles.p95_ms,
        arrow_sample_stats.total_bytes,
        hit_cold_ratio,
        cold_report.report_path,
        miss_report.report_path,
        hit_report.report_path,
        arrow_report.report_path
    );
    Ok(())
}

fn build_perf_config() -> PerfRunConfig {
    PerfRunConfig {
        warmup_samples: env_usize("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_BUILD_WARMUP", 1),
        samples: env_usize("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_BUILD_SAMPLES", 3),
        timeout_ms: env_u64("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_BUILD_TIMEOUT_MS", 3_000),
        concurrency: 1,
    }
}

fn hit_perf_config() -> PerfRunConfig {
    PerfRunConfig {
        warmup_samples: env_usize("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_HIT_WARMUP", 2),
        samples: env_usize("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_HIT_SAMPLES", 12),
        timeout_ms: env_u64("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_HIT_TIMEOUT_MS", 1_500),
        concurrency: 1,
    }
}

fn arrow_perf_config() -> PerfRunConfig {
    PerfRunConfig {
        warmup_samples: env_usize("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_ARROW_WARMUP", 2),
        samples: env_usize("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_ARROW_SAMPLES", 12),
        timeout_ms: env_u64("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_ARROW_TIMEOUT_MS", 1_500),
        concurrency: 1,
    }
}

fn hit_budget() -> PerfBudget {
    PerfBudget {
        max_p95_latency_ms: Some(env_f64(
            "XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_HIT_P95_MS",
            if std::env::var_os("CI").is_some() {
                500.0
            } else {
                300.0
            },
        )),
        max_error_rate: Some(0.0),
        ..PerfBudget::new()
    }
}

fn assert_hit_cold_ratio(hit_cold_ratio: f64) -> Result<(), String> {
    let max_ratio = env_f64("XIUXIAN_WENDAO_PERF_LOCAL_DUCKDB_HIT_COLD_RATIO", 0.90);
    if hit_cold_ratio > max_ratio {
        return Err(format!(
            "expected local DuckDB cache hit p95 ratio <= {max_ratio:.3}, got {hit_cold_ratio:.3}"
        ));
    }
    Ok(())
}

fn p95_ratio(numerator: &PerfReport, denominator: &PerfReport) -> f64 {
    numerator.quantiles.p95_ms / denominator.quantiles.p95_ms.max(0.001)
}

fn prime_hit_cache(
    root: &Path,
    include_dirs: &[String],
    excluded_dirs: &[String],
    cache_path: &Path,
    expected_note_count: usize,
) -> Result<(), String> {
    let (index, meta) = LinkGraphIndex::build_with_local_cache_path_with_meta(
        root,
        include_dirs,
        excluded_dirs,
        cache_path,
    )?;
    if meta.status != "miss" {
        return Err(format!(
            "expected local DuckDB cache miss during hit-cache priming, got {}",
            meta.status
        ));
    }
    assert_note_count(&index, expected_note_count)?;

    let (index, meta) = LinkGraphIndex::build_with_local_cache_path_with_meta(
        root,
        include_dirs,
        excluded_dirs,
        cache_path,
    )?;
    if meta.status != "hit" {
        return Err(format!(
            "expected local DuckDB cache hit after priming, got {}",
            meta.status
        ));
    }
    assert_note_count(&index, expected_note_count)
}

fn assert_note_count(index: &LinkGraphIndex, expected_note_count: usize) -> Result<(), String> {
    let stats = index.stats();
    if stats.total_notes != expected_note_count {
        return Err(format!(
            "expected {expected_note_count} indexed notes, got {}",
            stats.total_notes
        ));
    }
    Ok(())
}

fn annotate_report(
    report: &mut PerfReport,
    node_count: usize,
    hub_count: usize,
    cache_phase: &str,
) {
    report.add_metadata("cache_backend", "duckdb");
    report.add_metadata("cache_scope", "link_graph_local_cache");
    report.add_metadata("cache_phase", cache_phase);
    report.add_metadata("node_count", node_count.to_string());
    report.add_metadata("hub_count", hub_count.to_string());
    report.add_metadata("fixture_note_count", (node_count + hub_count).to_string());
}
