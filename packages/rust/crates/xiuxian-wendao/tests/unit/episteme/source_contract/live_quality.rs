#![cfg(feature = "julia")]

use std::env;

use super::support::{
    EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL_ENV, LiveQualityEvidenceInput,
    LiveQualityPhaseTimings, RUN_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_LIVE_ENV,
    live_quality_diagnostic_context, materialize_live_quality_read_model,
    normalize_live_quality_base_url, package_live_quality_batches, parse_live_quality_round_count,
    run_live_quality_prewarm_roundtrips, run_live_quality_roundtrips, start_live_quality_service,
    write_live_quality_evidence,
};

#[test]
fn episteme_source_contract_live_quality_base_url_normalization_trims_trailing_slash() {
    let Ok(base_url) = normalize_live_quality_base_url("  http://127.0.0.1:41082/  ") else {
        panic!("valid base URL should normalize");
    };

    assert_eq!(base_url, "http://127.0.0.1:41082");
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_base_url_normalization_rejects_blank() {
    let Err(error) = normalize_live_quality_base_url("   ") else {
        panic!("blank URL should fail");
    };

    assert!(error.contains(EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_BASE_URL_ENV));
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_base_url_normalization_rejects_unsupported_scheme() {
    let Err(error) = normalize_live_quality_base_url("grpc://127.0.0.1:41082") else {
        panic!("unsupported scheme should fail");
    };

    assert!(error.contains("http:// or https://"));
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_round_count_parser_uses_default() {
    let Ok(count) = parse_live_quality_round_count(None, "TEST_ROUNDS", 0, 0, 3) else {
        panic!("missing env should use default");
    };

    assert_eq!(count, 0);
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_round_count_parser_accepts_valid_count() {
    let Ok(count) = parse_live_quality_round_count(Some("2"), "TEST_ROUNDS", 0, 0, 3) else {
        panic!("valid round count should parse");
    };

    assert_eq!(count, 2);
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_round_count_parser_rejects_invalid_count() {
    let Err(error) = parse_live_quality_round_count(Some("abc"), "TEST_ROUNDS", 0, 0, 3) else {
        panic!("invalid round count should fail");
    };

    assert!(error.contains("TEST_ROUNDS"));
}

#[cfg(feature = "julia")]
#[test]
fn episteme_source_contract_live_quality_round_count_parser_rejects_out_of_range_count() {
    let Err(error) = parse_live_quality_round_count(Some("4"), "TEST_ROUNDS", 0, 0, 3) else {
        panic!("out-of-range round count should fail");
    };

    assert!(error.contains("between 0 and 3"));
}

#[cfg(feature = "julia")]
#[tokio::test]
async fn episteme_source_contract_live_wendaograph_quality_diagnostic_uses_compiled_seed()
-> Result<(), Box<dyn std::error::Error>> {
    if env::var_os(RUN_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_LIVE_ENV).is_none() {
        eprintln!(
            "skipping episteme source-contract WendaoGraph quality live diagnostic; set {RUN_EPISTEME_SOURCE_CONTRACT_WENDAOGRAPH_QUALITY_LIVE_ENV}=1"
        );
        return Ok(());
    }

    let context = live_quality_diagnostic_context()?;
    let materialized = materialize_live_quality_read_model(&context.repo_root)?;
    let (quality_batches, request_packaging_ms) =
        package_live_quality_batches(&materialized.materialization)?;
    let service = start_live_quality_service(&context).await?;
    let prewarm_summaries =
        run_live_quality_prewarm_roundtrips(&service.binding, &quality_batches).await?;
    let roundtrip_summaries =
        run_live_quality_roundtrips(&service.binding, &quality_batches).await?;

    write_live_quality_evidence(&LiveQualityEvidenceInput {
        repo_root: &context.repo_root,
        source_revision: &materialized.materialization.source_revision,
        request_row_counts: quality_batches.row_counts(),
        phase_timings: LiveQualityPhaseTimings {
            materialization: materialized.elapsed_ms,
            request_packaging: request_packaging_ms,
            service_ready: service.ready_ms,
        },
        service: &service,
        prewarm_summaries: &prewarm_summaries,
        roundtrip_summaries: &roundtrip_summaries,
        validation_hash_cache_report: materialized.validation_hash_cache_report.as_ref(),
    })?;

    Ok(())
}
