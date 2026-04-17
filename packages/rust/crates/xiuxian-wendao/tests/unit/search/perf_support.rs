use crate::search::perf_support::{
    RepoContentParquetMutationBenchmarkFixture, RepoContentQueryBenchmarkFixture,
};
use crate::search::repo_content_chunk::repo_content_chunk_partition_count_for_document_count;
#[cfg(all(feature = "duckdb", feature = "performance"))]
use crate::{clear_link_graph_wendao_config_override, set_link_graph_wendao_config_override};
#[cfg(all(feature = "duckdb", feature = "performance"))]
use serial_test::serial;
#[cfg(all(feature = "duckdb", feature = "performance"))]
use tempfile::TempDir;

#[cfg(all(feature = "duckdb", feature = "performance"))]
struct SearchDuckDbConfigOverride {
    _temp: TempDir,
}

#[cfg(all(feature = "duckdb", feature = "performance"))]
impl SearchDuckDbConfigOverride {
    fn install(
        slug: &str,
        preserve_insertion_order: bool,
        parquet_metadata_cache: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let config_path = temp.path().join("wendao.toml");
        let temp_directory = temp.path().join(format!("duckdb-{slug}-tmp"));
        std::fs::write(
            &config_path,
            format!(
                "[search.duckdb]\n\
                 enabled = true\n\
                 database_path = \":memory:\"\n\
                 temp_directory = \"{}\"\n\
                 threads = 4\n\
                 preserve_insertion_order = {}\n\
                 parquet_metadata_cache = {}\n\
                 prefer_virtual_arrow = true\n",
                temp_directory.display(),
                preserve_insertion_order,
                parquet_metadata_cache
            ),
        )?;
        set_link_graph_wendao_config_override(&config_path.to_string_lossy());
        Ok(Self { _temp: temp })
    }
}

#[cfg(all(feature = "duckdb", feature = "performance"))]
impl Drop for SearchDuckDbConfigOverride {
    fn drop(&mut self) {
        clear_link_graph_wendao_config_override();
    }
}

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

#[test]
fn repo_content_query_benchmark_fixture_reports_broad_query_path_dedup() {
    let fixture = RepoContentQueryBenchmarkFixture::synthetic(256);
    let hot = fixture
        .prepare_iteration()
        .measure_hot_query_for_token_after_cold_warmup("value");
    let flight = fixture
        .prepare_iteration()
        .measure_flight_batch_for_token_after_cold_warmup("value");

    assert_eq!(hot.hit_count, 5);
    assert_eq!(hot.rows_scanned, 256);
    assert_eq!(hot.matched_rows, 256);
    assert_eq!(flight.row_count, 5);
    assert_eq!(flight.rows_scanned, 256);
    assert_eq!(flight.matched_rows, 256);
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
            .prewarm
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .finalize
            .record_publication
            .as_secs_f64()
            * 1_000.0,
        small_snapshot
            .publish_profile
            .finalize
            .set_fingerprints
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
            .prewarm
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .finalize
            .record_publication
            .as_secs_f64()
            * 1_000.0,
        large_snapshot
            .publish_profile
            .finalize
            .set_fingerprints
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

#[cfg(feature = "performance")]
#[test]
fn repo_content_query_benchmark_reports_100k_broad_query_sample() {
    let fixture = RepoContentQueryBenchmarkFixture::synthetic(100_000);
    let hot = fixture
        .prepare_iteration()
        .measure_hot_query_for_token_after_cold_warmup("value");
    let flight = fixture
        .prepare_iteration()
        .measure_flight_batch_for_token_after_cold_warmup("value");

    println!(
        "repo content broad-query benchmark: docs={} hot_ms={:.3} hot_hits={} hot_rows_scanned={} hot_matched_rows={} flight_batch_ms={:.3} flight_rows={} flight_rows_scanned={} flight_matched_rows={}",
        100_000,
        hot.elapsed.as_secs_f64() * 1_000.0,
        hot.hit_count,
        hot.rows_scanned,
        hot.matched_rows,
        flight.elapsed.as_secs_f64() * 1_000.0,
        flight.row_count,
        flight.rows_scanned,
        flight.matched_rows
    );

    assert_eq!(hot.hit_count, 5);
    assert_eq!(hot.rows_scanned, 256);
    assert_eq!(hot.matched_rows, 256);
    assert_eq!(flight.row_count, 5);
    assert_eq!(flight.rows_scanned, 256);
    assert_eq!(flight.matched_rows, 256);
}

#[cfg(all(feature = "duckdb", feature = "performance"))]
#[test]
#[serial]
fn repo_content_query_benchmark_reports_duckdb_official_setting_profiles() {
    let fixture = RepoContentQueryBenchmarkFixture::synthetic(100_000);
    let profiles = [
        ("official_defaults", true, false),
        ("no_insertion_order", false, false),
        ("metadata_cache_only", true, true),
        ("combined_tuning", false, true),
    ];

    for (slug, preserve_insertion_order, parquet_metadata_cache) in profiles {
        let _override = SearchDuckDbConfigOverride::install(
            slug,
            preserve_insertion_order,
            parquet_metadata_cache,
        )
        .unwrap_or_else(|error| panic!("install DuckDB benchmark override `{slug}`: {error}"));
        let point = fixture
            .prepare_iteration()
            .measure_hot_query_after_cold_warmup();
        let broad = fixture
            .prepare_iteration()
            .measure_hot_query_for_token_after_cold_warmup("value");

        println!(
            "repo content query duckdb profile benchmark: profile={} preserve_insertion_order={} parquet_metadata_cache={} point_hot_ms={:.3} point_hits={} point_rows_scanned={} broad_hot_ms={:.3} broad_hits={} broad_rows_scanned={} broad_matched_rows={}",
            slug,
            preserve_insertion_order,
            parquet_metadata_cache,
            point.elapsed.as_secs_f64() * 1_000.0,
            point.hit_count,
            point.rows_scanned,
            broad.elapsed.as_secs_f64() * 1_000.0,
            broad.hit_count,
            broad.rows_scanned,
            broad.matched_rows
        );

        assert_eq!(point.hit_count, 1);
        assert_eq!(point.rows_scanned, 1);
        assert_eq!(broad.hit_count, 5);
        assert_eq!(broad.rows_scanned, 256);
        assert_eq!(broad.matched_rows, 256);
    }
}
