use std::{
    future::Future,
    sync::{OnceLock, mpsc},
    time::Duration,
};

use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepoIntelligenceError};

use super::contract::{
    JuliaParserSummaryRequestRow, build_julia_parser_summary_request_batch,
    decode_julia_parser_file_summary, decode_julia_parser_root_summary,
    decode_julia_parser_summary_response_rows,
};
use super::route::ParserSummaryRouteKind;
use super::transport::{
    build_julia_parser_summary_flight_transport_client,
    julia_parser_summary_timeout_secs_for_repository,
    process_julia_parser_summary_flight_batches_for_repository,
};
use super::types::{JuliaParserFileSummary, JuliaParserSourceSummary};

static JULIA_PARSER_SUMMARY_RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> =
    OnceLock::new();
const MAX_JULIA_PARSER_SUMMARY_BLOCKING_TIMEOUT_SECS: u64 = 30;

/// Build one parser-summary request, execute the configured Flight roundtrip,
/// and decode a Julia file summary.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request cannot be materialized,
/// the repository does not declare a usable parser-summary client, the roundtrip
/// fails, or the response violates the staged contract.
pub(crate) async fn fetch_julia_parser_file_summary_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    source_text: &str,
) -> Result<JuliaParserFileSummary, RepoIntelligenceError> {
    let batch = build_julia_parser_summary_request_batch(&[JuliaParserSummaryRequestRow {
        request_id: format!("julia-file-summary:{source_id}"),
        source_id: source_id.to_string(),
        source_text: source_text.to_string(),
    }])?;
    let response_batches = process_julia_parser_summary_flight_batches_for_repository(
        repository,
        ParserSummaryRouteKind::FileSummary,
        &[batch],
    )
    .await?;
    let rows = decode_julia_parser_summary_response_rows(response_batches.as_slice())?;
    decode_julia_parser_file_summary(ParserSummaryRouteKind::FileSummary, rows.as_slice())
}

/// Build one parser-summary request, execute the configured Flight roundtrip,
/// and decode a Julia root summary.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request cannot be materialized,
/// the repository does not declare a usable parser-summary client, the roundtrip
/// fails, or the response violates the staged contract.
pub(crate) async fn fetch_julia_parser_root_summary_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    source_text: &str,
) -> Result<JuliaParserSourceSummary, RepoIntelligenceError> {
    let batch = build_julia_parser_summary_request_batch(&[JuliaParserSummaryRequestRow {
        request_id: format!("julia-root-summary:{source_id}"),
        source_id: source_id.to_string(),
        source_text: source_text.to_string(),
    }])?;
    let response_batches = process_julia_parser_summary_flight_batches_for_repository(
        repository,
        ParserSummaryRouteKind::RootSummary,
        &[batch],
    )
    .await?;
    let rows = decode_julia_parser_summary_response_rows(response_batches.as_slice())?;
    decode_julia_parser_root_summary(ParserSummaryRouteKind::RootSummary, rows.as_slice())
}

pub(crate) fn fetch_julia_parser_file_summary_blocking_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    source_text: &str,
) -> Result<JuliaParserFileSummary, RepoIntelligenceError> {
    let runtime = julia_parser_summary_runtime()?;
    let repository = repository.clone();
    let source_id = source_id.to_string();
    let source_id_for_task = source_id.clone();
    let source_text = source_text.to_string();
    let timeout_secs = julia_parser_summary_timeout_secs_for_repository(
        &repository,
        ParserSummaryRouteKind::FileSummary,
    )?
    .min(MAX_JULIA_PARSER_SUMMARY_BLOCKING_TIMEOUT_SECS);
    let timeout = Duration::from_secs(timeout_secs);
    let result = run_julia_parser_summary_blocking(runtime, async move {
        tokio::time::timeout(
            timeout,
            fetch_julia_parser_file_summary_for_repository(
                &repository,
                &source_id_for_task,
                &source_text,
            ),
        )
        .await
    })
    .map_err(|_| RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Julia parser-summary file-summary task exceeded {timeout_secs}s for `{source_id}`"
        ),
    })?;
    let result = result?;
    Ok(result)
}

pub(crate) fn fetch_julia_parser_root_summary_blocking_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    source_text: &str,
) -> Result<JuliaParserSourceSummary, RepoIntelligenceError> {
    let runtime = julia_parser_summary_runtime()?;
    let repository = repository.clone();
    let source_id = source_id.to_string();
    let source_id_for_task = source_id.clone();
    let source_text = source_text.to_string();
    let timeout_secs = julia_parser_summary_timeout_secs_for_repository(
        &repository,
        ParserSummaryRouteKind::RootSummary,
    )?
    .min(MAX_JULIA_PARSER_SUMMARY_BLOCKING_TIMEOUT_SECS);
    let timeout = Duration::from_secs(timeout_secs);
    let result = run_julia_parser_summary_blocking(runtime, async move {
        tokio::time::timeout(
            timeout,
            fetch_julia_parser_root_summary_for_repository(
                &repository,
                &source_id_for_task,
                &source_text,
            ),
        )
        .await
    })
    .map_err(|_| RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Julia parser-summary root-summary task exceeded {timeout_secs}s for `{source_id}`"
        ),
    })?;
    let result = result?;
    Ok(result)
}

pub(crate) fn validate_julia_parser_summary_preflight_for_repository(
    repository: &RegisteredRepository,
) -> Result<(), RepoIntelligenceError> {
    for route_kind in [
        ParserSummaryRouteKind::FileSummary,
        ParserSummaryRouteKind::RootSummary,
    ] {
        let _client = build_julia_parser_summary_flight_transport_client(repository, route_kind)?;
    }
    Ok(())
}

fn julia_parser_summary_runtime() -> Result<&'static tokio::runtime::Runtime, RepoIntelligenceError>
{
    JULIA_PARSER_SUMMARY_RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .thread_name("wendao-julia-parser-summary")
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|message| RepoIntelligenceError::AnalysisFailed {
            message: format!("failed to build shared Julia parser-summary runtime: {message}"),
        })
}

#[cfg(test)]
fn shared_julia_parser_summary_runtime_identity_for_tests() -> Result<usize, RepoIntelligenceError>
{
    let runtime = julia_parser_summary_runtime()?;
    Ok(std::ptr::from_ref(runtime) as usize)
}

fn run_julia_parser_summary_blocking<T, TFuture>(
    runtime: &'static tokio::runtime::Runtime,
    future: TFuture,
) -> T
where
    T: Send + 'static,
    TFuture: Future<Output = T> + Send + 'static,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
            tokio::task::block_in_place(|| runtime.block_on(future))
        } else {
            let (sender, receiver) = mpsc::sync_channel(1);
            std::thread::spawn(move || {
                let _ = sender.send(runtime.block_on(future));
            });
            receiver.recv().unwrap_or_else(|error| {
                panic!("failed to run julia parser-summary blocking task: {error}")
            })
        }
    } else {
        runtime.block_on(future)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/plugin/parser_summary/fetch.rs"]
mod tests;
