use std::time::Instant;

use super::{
    DEFAULT_PPR_ALPHA, DEFAULT_PPR_MAX_ITER, DEFAULT_PPR_TOL, DEFAULT_TARGET_P95_MS,
    HARD_SANITY_P95_MS, HUB_COUNT, NODE_COUNT, QUERY_COUNT, RELATED_LIMIT, RELATED_MAX_DISTANCE,
    build_large_fixture, env_f64, env_flag, env_subgraph_mode, env_usize, note_id, percentile,
};
use tempfile::tempdir;
use xiuxian_wendao::{LinkGraphIndex, LinkGraphPprSubgraphMode, LinkGraphRelatedPprOptions};

#[test]
#[ignore = "heavy benchmark; run with --ignored to validate LinkGraph PPR latency"]
fn test_link_graph_related_ppr_latency_on_10k_fixture() -> Result<(), Box<dyn std::error::Error>> {
    const WARMUP_QUERY_COUNT: usize = 8;

    let tmp = tempdir()?;
    build_large_fixture(tmp.path())?;

    let build_started = Instant::now();
    let index = LinkGraphIndex::build(tmp.path())
        .map_err(|err| format!("failed to build link graph benchmark fixture: {err}"))?;
    let build_elapsed_ms = build_started.elapsed().as_secs_f64() * 1000.0;

    let ppr_alpha = env_f64("XIUXIAN_WENDAO_PPR_ALPHA", DEFAULT_PPR_ALPHA);
    let ppr_max_iter = env_usize("XIUXIAN_WENDAO_PPR_MAX_ITER", DEFAULT_PPR_MAX_ITER);
    let ppr_tol = env_f64("XIUXIAN_WENDAO_PPR_TOL", DEFAULT_PPR_TOL);
    let ppr_subgraph_mode = env_subgraph_mode(
        "XIUXIAN_WENDAO_PPR_SUBGRAPH_MODE",
        LinkGraphPprSubgraphMode::Auto,
    );
    let ppr = LinkGraphRelatedPprOptions {
        alpha: Some(ppr_alpha),
        max_iter: Some(ppr_max_iter),
        tol: Some(ppr_tol),
        subgraph_mode: Some(ppr_subgraph_mode),
    };

    let mut ppr_core_ms: Vec<f64> = Vec::with_capacity(QUERY_COUNT);
    let mut ppr_total_ms: Vec<f64> = Vec::with_capacity(QUERY_COUNT);
    let mut e2e_ms: Vec<f64> = Vec::with_capacity(QUERY_COUNT);
    let mut timeout_count = 0_usize;

    // Warm up fixture/cache path so p95 reflects steady-state kernel behavior.
    for turn in 0..WARMUP_QUERY_COUNT {
        let seed = note_id((turn * 211) % NODE_COUNT);
        let _ =
            index.related_with_diagnostics(&seed, RELATED_MAX_DISTANCE, RELATED_LIMIT, Some(&ppr));
    }

    for turn in 0..QUERY_COUNT {
        let seed = note_id(((turn + WARMUP_QUERY_COUNT) * 211) % NODE_COUNT);
        let started = Instant::now();
        let (rows, diagnostics) =
            index.related_with_diagnostics(&seed, RELATED_MAX_DISTANCE, RELATED_LIMIT, Some(&ppr));
        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        e2e_ms.push(elapsed_ms);

        let diag = diagnostics.ok_or("missing related PPR diagnostics in benchmark run")?;
        if diag.timed_out {
            timeout_count += 1;
        }
        ppr_core_ms
            .push(diag.partition_duration_ms + diag.kernel_duration_ms + diag.fusion_duration_ms);
        ppr_total_ms.push(diag.total_duration_ms);

        assert!(
            !rows.is_empty(),
            "expected related neighbors for seed={seed}, but got none"
        );
    }

    let p50_ppr_ms = percentile(&ppr_core_ms, 50);
    let p95_ppr_ms = percentile(&ppr_core_ms, 95);
    let p95_total_ms = percentile(&ppr_total_ms, 95);
    let p95_e2e_ms = percentile(&e2e_ms, 95);
    let target_p95_ms = env_f64("XIUXIAN_WENDAO_PPR_P95_MS_BUDGET", DEFAULT_TARGET_P95_MS);
    let enforce_target = env_flag("XIUXIAN_WENDAO_ENFORCE_PPR_P95");
    let enforce_total_sanity = env_flag("XIUXIAN_WENDAO_ENFORCE_PPR_TOTAL_SANITY");

    assert_eq!(
        timeout_count, 0,
        "related PPR benchmark should not hit runtime timeout in fixture run"
    );
    if enforce_total_sanity {
        assert!(
            p95_total_ms <= HARD_SANITY_P95_MS,
            "related PPR total p95 too high for sanity bound: p95_total={p95_total_ms:.2}ms (sanity < {HARD_SANITY_P95_MS:.2}ms)"
        );
    }

    if enforce_target {
        assert!(
            p95_ppr_ms <= target_p95_ms,
            "related PPR p95 exceeded target budget: p95={p95_ppr_ms:.2}ms, budget={target_p95_ms:.2}ms"
        );
    }

    println!(
        "link_graph_related_ppr_benchmark: nodes={}, hubs={}, queries={}, build_ms={:.2}, p50_ppr_ms={:.2}, p95_ppr_ms={:.2}, p95_total_ms={:.2}, p95_e2e_ms={:.2}, alpha={:.4}, max_iter={}, tol={:.2e}, subgraph_mode={:?}, enforce_target={}, enforce_total_sanity={}, target_p95_ms={:.2}",
        NODE_COUNT + HUB_COUNT,
        HUB_COUNT,
        QUERY_COUNT,
        build_elapsed_ms,
        p50_ppr_ms,
        p95_ppr_ms,
        p95_total_ms,
        p95_e2e_ms,
        ppr_alpha,
        ppr_max_iter,
        ppr_tol,
        ppr_subgraph_mode,
        enforce_target,
        enforce_total_sanity,
        target_p95_ms
    );

    Ok(())
}
