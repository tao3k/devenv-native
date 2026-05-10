use super::{
    WendaoSearchGraphStructuralStabilizationLimits, WendaoSearchGraphStructuralWarmPathStats,
    warm_path_passes_limits,
};

fn stats(p95_ms: f64, max_ms: f64, spread_ratio: f64) -> WendaoSearchGraphStructuralWarmPathStats {
    WendaoSearchGraphStructuralWarmPathStats {
        sample_count: 3,
        min_ms: 1.0,
        median_ms: p95_ms,
        p95_ms,
        max_ms,
        spread_ratio,
    }
}

#[test]
fn low_millisecond_spread_is_observed_without_degrading_admission() {
    let limits = WendaoSearchGraphStructuralStabilizationLimits {
        max_spread_ratio: 2.0,
        ..WendaoSearchGraphStructuralStabilizationLimits::default()
    };

    assert!(warm_path_passes_limits(&stats(44.0, 44.0, 44.0), &limits));
}

#[test]
fn p95_or_max_budget_overflow_degrades_admission() {
    let limits = WendaoSearchGraphStructuralStabilizationLimits::default();

    assert!(!warm_path_passes_limits(&stats(151.0, 151.0, 1.0), &limits));
    assert!(!warm_path_passes_limits(&stats(149.0, 251.0, 1.0), &limits));
}

#[test]
fn spread_ratio_is_secondary_only_inside_tail_budget_region() {
    let limits = WendaoSearchGraphStructuralStabilizationLimits {
        max_spread_ratio: 2.0,
        ..WendaoSearchGraphStructuralStabilizationLimits::default()
    };

    assert!(warm_path_passes_limits(&stats(12.0, 12.0, 12.0), &limits));
    assert!(!warm_path_passes_limits(&stats(149.0, 160.0, 3.0), &limits));
}
