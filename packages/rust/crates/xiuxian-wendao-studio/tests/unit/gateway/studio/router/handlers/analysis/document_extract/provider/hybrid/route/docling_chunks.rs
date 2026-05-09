#[test]
fn docling_page_range_fallback_batches_contiguous_pages() {
    let ranges = contiguous_page_ranges(&std::collections::BTreeSet::from([0, 1, 2, 5, 7, 8]));

    assert_eq!(ranges, vec![(0, 2), (5, 5), (7, 8)]);
}

#[test]
fn docling_page_range_fallback_can_split_contiguous_ranges() {
    let pages = std::collections::BTreeSet::from([0, 1, 2, 3, 4, 5, 8]);

    let ranges = docling_page_range_fallback_ranges(&pages, Some(2));

    assert_eq!(ranges, vec![(0, 1), (2, 3), (4, 5), (8, 8)]);
    assert_eq!(
        docling_page_range_fallback_ranges(&pages, None),
        vec![(0, 5), (8, 8)]
    );
}

#[test]
fn docling_page_range_chunk_plan_requires_exact_fallback_coverage() -> Result<(), String> {
    let pages = std::collections::BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8]);

    assert_eq!(
        docling_page_range_chunk_plan_with_lookup(&pages, &|_key| None)?,
        None
    );
    assert_eq!(
        docling_page_range_chunk_plan_with_lookup(&pages, &|_key| Some(
            "1:3,4:4,5:6,7:9".to_string()
        ))?,
        Some(vec![(0, 2), (3, 3), (4, 5), (6, 8)])
    );
    assert!(
        docling_page_range_chunk_plan_with_lookup(&pages, &|_key| Some("1:3,5:6,7:9".to_string()))
            .is_err()
    );
    assert!(
        docling_page_range_chunk_plan_with_lookup(&pages, &|_key| Some("1:4,4:6,7:9".to_string()))
            .is_err()
    );
    assert!(
        docling_page_range_chunk_plan_with_lookup(&pages, &|_key| Some("1:3,4:6,7:10".to_string()))
            .is_err()
    );

    Ok(())
}

#[test]
fn docling_structure_recovery_defaults_to_evidence_chunk_size() -> Result<(), String> {
    let small_pages = std::collections::BTreeSet::from([0, 1, 2, 3]);
    let large_pages = std::collections::BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8]);

    assert_eq!(
        docling_page_range_chunk_size_for_planner_with_lookup(
            HybridPdfOcrProfilePlanner::Disabled,
            &|_key| None,
        ),
        None
    );
    assert_eq!(
        docling_page_range_chunk_size_for_planner_with_lookup(
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            &|_key| None,
        ),
        Some(3)
    );
    assert_eq!(
        docling_page_range_chunk_size_for_planner_with_lookup(
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            &|_key| Some("5".to_string()),
        ),
        Some(5)
    );
    assert_eq!(
        docling_page_range_chunk_size_for_pages_with_lookup(
            &small_pages,
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            &|_key| None,
        ),
        Some(1)
    );
    assert_eq!(
        docling_page_range_chunk_size_for_pages_with_lookup(
            &large_pages,
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            &|_key| None,
        ),
        Some(3)
    );
    assert_eq!(
        docling_page_range_fallback_ranges_with_lookup(
            &small_pages,
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            &|_key| None,
        )?,
        vec![(0, 0), (1, 1), (2, 2), (3, 3)]
    );
    assert_eq!(
        docling_page_range_fallback_ranges_with_lookup(
            &large_pages,
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            &|_key| None,
        )?,
        vec![(0, 2), (3, 5), (6, 8)]
    );
    assert_eq!(
        docling_page_range_chunk_size_for_pages_with_lookup(
            &large_pages,
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            &|_key| Some("5".to_string()),
        ),
        Some(5)
    );

    Ok(())
}

#[test]
fn docling_structure_recovery_target_chunk_count_limits_docling_contention() {
    assert_eq!(
        docling_page_range_target_chunk_count(
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            4,
            9,
        ),
        4
    );
    assert_eq!(
        docling_page_range_target_chunk_count(
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            9,
            9,
        ),
        4
    );
    assert_eq!(
        docling_page_range_target_chunk_count(
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            3,
            9,
        ),
        3
    );
    assert_eq!(
        docling_page_range_target_chunk_count(
            HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
            4,
            4,
        ),
        4
    );
    assert_eq!(
        docling_page_range_target_chunk_count(HybridPdfOcrProfilePlanner::Disabled, 4, 9),
        4
    );
    assert_eq!(
        docling_page_range_target_chunk_count(HybridPdfOcrProfilePlanner::Disabled, 1, 9),
        3
    );
}

#[test]
fn docling_structure_recovery_balances_page_ranges_by_source_profile_weight() {
    let pages = std::collections::BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8]);
    let profiles = vec![
        sample_source_page_profile(0, 8),
        sample_source_page_profile(1, 7),
        sample_source_page_profile(2, 6),
        sample_source_page_profile(3, 24),
        sample_source_page_profile(4, 11),
        sample_source_page_profile(5, 12),
        sample_source_page_profile(6, 8),
        sample_source_page_profile(7, 8),
        sample_source_page_profile(8, 8),
    ];

    assert_eq!(
        weighted_docling_page_range_fallback_ranges(&pages, profiles.as_slice(), 4),
        Some(vec![(0, 2), (3, 3), (4, 5), (6, 8)])
    );
    assert_eq!(
        weighted_docling_page_range_fallback_ranges(&pages, profiles.as_slice(), 3),
        Some(vec![(0, 2), (3, 4), (5, 8)])
    );
}

#[test]
fn docling_structure_recovery_preserves_tail_when_spending_extra_chunk() {
    let pages = std::collections::BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8]);
    let profiles = vec![
        sample_source_page_profile_with_path_ops(0, 202, 4),
        sample_source_page_profile_with_path_ops(1, 155, 3),
        sample_source_page_profile_with_path_ops(2, 118, 2),
        sample_source_page_profile_with_path_ops(3, 268, 240),
        sample_source_page_profile_with_path_ops(4, 128, 20),
        sample_source_page_profile_with_path_ops(5, 248, 160),
        sample_source_page_profile_with_path_ops(6, 305, 309),
        sample_source_page_profile_with_path_ops(7, 200, 100),
        sample_source_page_profile_with_path_ops(8, 171, 104),
    ];

    assert_eq!(
        weighted_docling_page_range_fallback_ranges(&pages, profiles.as_slice(), 4),
        Some(vec![(0, 2), (3, 3), (4, 5), (6, 8)])
    );
}

#[test]
fn docling_page_range_plan_records_decision_metadata() -> Result<(), String> {
    let pages = std::collections::BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8]);

    let (ranges, plan) = docling_page_range_fallback_plan_for_source_with_lookup(
        &pages,
        HybridPdfOcrProfilePlanner::DoclingStructureRecovery,
        Path::new("missing.pdf"),
        4,
        &|_key| Some("1:3,4:4,5:6,7:9".to_string()),
    )?;

    assert_eq!(ranges, vec![(0, 2), (3, 3), (4, 5), (6, 8)]);
    assert_eq!(plan.strategy, "explicit-plan");
    assert_eq!(plan.target_chunk_count, 4);
    assert_eq!(plan.fallback_page_count, 9);
    assert_eq!(plan.range_count, 4);
    assert_eq!(plan.chunk_size, None);
    assert!(!plan.source_profile_used);
    assert_eq!(plan.ranges[0].one_based_start, 1);
    assert_eq!(plan.ranges[3].one_based_end, 9);

    Ok(())
}

fn sample_source_page_profile(page_index: u32, estimated_weight: u32) -> PdfSourcePageProfile {
    PdfSourcePageProfile {
        page_index,
        content_bytes: estimated_weight,
        operation_count: estimated_weight,
        text_show_ops: 1,
        path_ops: 0,
        rectangle_ops: 0,
        draw_object_ops: 0,
        estimated_weight,
    }
}

fn sample_source_page_profile_with_path_ops(
    page_index: u32,
    estimated_weight: u32,
    path_ops: u32,
) -> PdfSourcePageProfile {
    PdfSourcePageProfile {
        path_ops,
        ..sample_source_page_profile(page_index, estimated_weight)
    }
}
