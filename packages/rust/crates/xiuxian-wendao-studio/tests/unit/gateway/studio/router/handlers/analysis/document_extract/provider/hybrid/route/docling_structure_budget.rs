#[test]
fn docling_structure_recovery_can_split_overbudget_structure_cost_ranges() {
    let ranges = vec![(0, 2), (3, 3), (4, 5), (6, 8)];
    let profiles = vec![
        sample_source_page_profile(0, 30),
        sample_source_page_profile(1, 30),
        sample_source_page_profile(2, 30),
        sample_source_page_profile_with_path_ops(3, 120, 120),
        sample_source_page_profile(4, 30),
        sample_source_page_profile(5, 30),
        sample_source_page_profile_with_path_ops(6, 120, 220),
        sample_source_page_profile(7, 60),
        sample_source_page_profile(8, 60),
    ];

    assert_eq!(
        structure_cost_budgeted_docling_page_range_fallback_ranges(
            ranges.as_slice(),
            profiles.as_slice(),
            1_000,
        ),
        vec![(0, 2), (3, 3), (4, 5), (6, 6), (7, 8)]
    );
}

#[test]
fn docling_structure_budget_does_not_exceed_target_chunk_count() {
    let ranges = vec![(0, 2), (3, 3), (4, 5), (6, 8)];
    let profiles = vec![
        sample_source_page_profile(0, 30),
        sample_source_page_profile(1, 30),
        sample_source_page_profile(2, 30),
        sample_source_page_profile_with_path_ops(3, 120, 120),
        sample_source_page_profile(4, 30),
        sample_source_page_profile(5, 30),
        sample_source_page_profile_with_path_ops(6, 120, 220),
        sample_source_page_profile(7, 60),
        sample_source_page_profile(8, 60),
    ];

    assert_eq!(
        structure_cost_budgeted_docling_page_range_fallback_ranges_with_limit(
            ranges.as_slice(),
            profiles.as_slice(),
            1_000,
            4,
        ),
        ranges
    );
}

#[test]
fn docling_structure_budget_can_spend_spare_endpoint_capacity() {
    let ranges = vec![(0, 2), (3, 3), (4, 5), (6, 8)];
    let profiles = vec![
        sample_source_page_profile(0, 30),
        sample_source_page_profile(1, 30),
        sample_source_page_profile(2, 30),
        sample_source_page_profile_with_path_ops(3, 120, 120),
        sample_source_page_profile(4, 30),
        sample_source_page_profile(5, 30),
        sample_source_page_profile_with_path_ops(6, 120, 220),
        sample_source_page_profile(7, 60),
        sample_source_page_profile(8, 60),
    ];

    assert_eq!(
        structure_cost_budgeted_docling_page_range_fallback_ranges_with_limit(
            ranges.as_slice(),
            profiles.as_slice(),
            1_000,
            5,
        ),
        vec![(0, 2), (3, 3), (4, 5), (6, 6), (7, 8)]
    );
}
