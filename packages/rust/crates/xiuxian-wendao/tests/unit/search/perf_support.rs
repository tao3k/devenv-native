use crate::search::perf_support::{
    RepoContentParquetMutationBenchmarkFixture, RepoContentQueryBenchmarkFixture,
};
use crate::search::repo_content_chunk::repo_content_chunk_partition_count_for_document_count;

#[test]
fn repo_content_parquet_mutation_fixture_preserves_row_count_and_query_readability() {
    let fixture = RepoContentParquetMutationBenchmarkFixture::synthetic(256);
    let snapshot = fixture.prepare_iteration().run();

    assert_eq!(snapshot.base_document_count, 256);
    assert_eq!(snapshot.changed_document_count, 3);
    assert_eq!(snapshot.deleted_path_count, 1);
    assert_eq!(
        snapshot.partition_bucket_count,
        repo_content_chunk_partition_count_for_document_count(snapshot.base_document_count)
    );
    assert_eq!(
        snapshot.touched_partition_count,
        snapshot.touched_base_documents_by_partition.len()
    );
    assert_eq!(
        snapshot.touched_base_document_count,
        snapshot
            .touched_base_documents_by_partition
            .values()
            .copied()
            .sum::<usize>()
    );
    assert!(snapshot.touched_partition_count > 0);
    assert_eq!(
        snapshot
            .publish_profile
            .mutation_write
            .touched_partition_count,
        snapshot.touched_partition_count
    );
    assert_eq!(snapshot.row_count, fixture.expected_row_count());
    assert_eq!(
        snapshot.added_query_paths,
        vec![fixture.added_path().to_string()]
    );
    assert!(snapshot.deleted_query_paths.is_empty());
}

#[test]
fn repo_content_query_benchmark_fixture_reports_cold_hot_and_flight_samples() {
    let fixture = RepoContentQueryBenchmarkFixture::synthetic(256);
    let snapshot = fixture.prepare_iteration().run();

    assert_eq!(snapshot.base_document_count, 256);
    assert_eq!(snapshot.publication_row_count, 3_072);
    assert_eq!(snapshot.cold_query_hit_count, 1);
    assert_eq!(snapshot.cold_query_rows_scanned, 1);
    assert_eq!(snapshot.hot_query_hit_count, 1);
    assert_eq!(snapshot.hot_query_rows_scanned, 1);
    assert_eq!(snapshot.flight_batch_row_count, 1);
    assert_eq!(snapshot.flight_batch_rows_scanned, 1);
    assert_eq!(
        snapshot.cold_first_path.as_deref(),
        Some(snapshot.expected_path.as_str())
    );
    assert_eq!(
        snapshot.hot_first_path.as_deref(),
        Some(snapshot.expected_path.as_str())
    );
    assert!(!snapshot.query_token.is_empty());
    assert!(!snapshot.query_engine_kind.is_empty());
    assert!(!snapshot.persisted_metadata_backend.is_empty());
}

#[cfg(feature = "performance")]
#[test]
fn repo_content_parquet_mutation_benchmark_reports_1k_and_10k_samples() {
    let small_fixture = RepoContentParquetMutationBenchmarkFixture::synthetic(1_000);
    let small_snapshot = small_fixture.prepare_iteration().run();
    let large_fixture = RepoContentParquetMutationBenchmarkFixture::synthetic(10_000);
    let large_snapshot = large_fixture.prepare_iteration().run();
    let ratio = large_snapshot.elapsed.as_secs_f64() / small_snapshot.elapsed.as_secs_f64();

    println!(
        "repo publication parquet benchmark: docs={} changed={} deleted={} touched_partitions={} untouched_partitions={} touched_base_docs={} touched_distribution={:?} row_count={} elapsed_ms={:.3} profile_ms={{fingerprints:{:.3}, record:{:.3}, merge:{:.3}, plan:{:.3}, copy_untouched:{:.3}, load_touched:{:.3}, filter:{:.3}, changed_payload:{:.3}, write_touched:{:.3}, snapshot:{:.3}, prewarm:{:.3}, record_publication:{:.3}, set_fingerprints:{:.3}}}",
        small_snapshot.base_document_count,
        small_snapshot.changed_document_count,
        small_snapshot.deleted_path_count,
        small_snapshot.touched_partition_count,
        small_snapshot
            .publish_profile
            .mutation_write
            .untouched_partition_count,
        small_snapshot.touched_base_document_count,
        small_snapshot.touched_base_documents_by_partition,
        small_snapshot.row_count,
        small_snapshot.elapsed.as_secs_f64() * 1_000.0,
        small_snapshot
            .publish_profile
            .previous_fingerprint_read_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .current_record_read_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .fingerprint_merge_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot.publish_profile.plan_elapsed.as_secs_f64() * 1_000.0,
        small_snapshot
            .publish_profile
            .mutation_write
            .copy_untouched_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .mutation_write
            .load_touched_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .mutation_write
            .filter_replaced_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .mutation_write
            .changed_payload_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .mutation_write
            .write_touched_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .mutation_write
            .write_snapshot_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .finalize
            .prewarm_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .finalize
            .record_publication_elapsed
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .finalize
            .set_fingerprints_elapsed
            .as_secs_f64()
            * 1_000.0
    );
    println!(
        "repo publication parquet benchmark: docs={} changed={} deleted={} touched_partitions={} untouched_partitions={} touched_base_docs={} touched_distribution={:?} row_count={} elapsed_ms={:.3} profile_ms={{fingerprints:{:.3}, record:{:.3}, merge:{:.3}, plan:{:.3}, copy_untouched:{:.3}, load_touched:{:.3}, filter:{:.3}, changed_payload:{:.3}, write_touched:{:.3}, snapshot:{:.3}, prewarm:{:.3}, record_publication:{:.3}, set_fingerprints:{:.3}}}",
        large_snapshot.base_document_count,
        large_snapshot.changed_document_count,
        large_snapshot.deleted_path_count,
        large_snapshot.touched_partition_count,
        large_snapshot
            .publish_profile
            .mutation_write
            .untouched_partition_count,
        large_snapshot.touched_base_document_count,
        large_snapshot.touched_base_documents_by_partition,
        large_snapshot.row_count,
        large_snapshot.elapsed.as_secs_f64() * 1_000.0,
        large_snapshot
            .publish_profile
            .previous_fingerprint_read_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .current_record_read_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .fingerprint_merge_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot.publish_profile.plan_elapsed.as_secs_f64() * 1_000.0,
        large_snapshot
            .publish_profile
            .mutation_write
            .copy_untouched_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .mutation_write
            .load_touched_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .mutation_write
            .filter_replaced_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .mutation_write
            .changed_payload_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .mutation_write
            .write_touched_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .mutation_write
            .write_snapshot_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .finalize
            .prewarm_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .finalize
            .record_publication_elapsed
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .finalize
            .set_fingerprints_elapsed
            .as_secs_f64()
            * 1_000.0
    );
    println!(
        "repo publication parquet benchmark ratio: docs={} over docs={} => {:.2}x",
        large_snapshot.base_document_count, small_snapshot.base_document_count, ratio
    );

    assert_eq!(small_snapshot.row_count, small_fixture.expected_row_count());
    assert_eq!(large_snapshot.row_count, large_fixture.expected_row_count());
    assert_eq!(
        small_snapshot.added_query_paths,
        vec![small_fixture.added_path().to_string()]
    );
    assert_eq!(
        large_snapshot.added_query_paths,
        vec![large_fixture.added_path().to_string()]
    );
    assert!(small_snapshot.touched_partition_count > 0);
    assert!(large_snapshot.touched_partition_count > 0);
    assert!(small_snapshot.deleted_query_paths.is_empty());
    assert!(large_snapshot.deleted_query_paths.is_empty());
}

#[cfg(feature = "performance")]
#[test]
fn repo_content_query_benchmark_reports_100k_sample() {
    let fixture = RepoContentQueryBenchmarkFixture::synthetic(100_000);
    let snapshot = fixture.prepare_iteration().run();

    println!(
        "repo content query benchmark: docs={} row_count={} engine={} metadata_backend={} valkey_target_configured={} cold_ms={:.3} hot_ms={:.3} flight_batch_ms={:.3} cold_hits={} cold_rows_scanned={} hot_hits={} hot_rows_scanned={} flight_rows={} flight_rows_scanned={} expected_path={}",
        snapshot.base_document_count,
        snapshot.publication_row_count,
        snapshot.query_engine_kind,
        snapshot.persisted_metadata_backend,
        snapshot.valkey_target_configured,
        snapshot.cold_query_elapsed.as_secs_f64() * 1_000.0,
        snapshot.hot_query_elapsed.as_secs_f64() * 1_000.0,
        snapshot.flight_batch_elapsed.as_secs_f64() * 1_000.0,
        snapshot.cold_query_hit_count,
        snapshot.cold_query_rows_scanned,
        snapshot.hot_query_hit_count,
        snapshot.hot_query_rows_scanned,
        snapshot.flight_batch_row_count,
        snapshot.flight_batch_rows_scanned,
        snapshot.expected_path
    );

    assert_eq!(snapshot.base_document_count, 100_000);
    assert_eq!(snapshot.publication_row_count, 1_200_000);
    assert_eq!(snapshot.cold_query_hit_count, 1);
    assert_eq!(snapshot.cold_query_rows_scanned, 1);
    assert_eq!(snapshot.hot_query_hit_count, 1);
    assert_eq!(snapshot.hot_query_rows_scanned, 1);
    assert_eq!(snapshot.flight_batch_row_count, 1);
    assert_eq!(snapshot.flight_batch_rows_scanned, 1);
    assert_eq!(
        snapshot.cold_first_path.as_deref(),
        Some(snapshot.expected_path.as_str())
    );
    assert_eq!(
        snapshot.hot_first_path.as_deref(),
        Some(snapshot.expected_path.as_str())
    );
}
