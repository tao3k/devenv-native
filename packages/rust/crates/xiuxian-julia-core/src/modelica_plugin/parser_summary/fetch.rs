use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use std::sync::OnceLock;

use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepoIntelligenceError};

use super::contract::{
    ModelicaParserSummaryRequestRow, build_modelica_parser_summary_request_batch,
    decode_modelica_parser_file_summary, decode_modelica_parser_summary_response_rows,
};
use super::route::ParserSummaryRouteKind;
use super::transport::{
    build_modelica_parser_summary_flight_transport_client,
    modelica_parser_summary_timeout_secs_for_repository,
    process_modelica_parser_summary_flight_batches_for_repository,
};
use super::types::ModelicaParserFileSummary;

static MODELICA_PARSER_SUMMARY_RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> =
    OnceLock::new();

pub(crate) async fn fetch_modelica_parser_file_summary_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    source_text: &str,
) -> Result<ModelicaParserFileSummary, RepoIntelligenceError> {
    let batch = build_modelica_parser_summary_request_batch(&[ModelicaParserSummaryRequestRow {
        request_id: format!("modelica-file-summary:{source_id}"),
        source_id: source_id.to_string(),
        source_text: source_text.to_string(),
    }])?;
    let response_batches = process_modelica_parser_summary_flight_batches_for_repository(
        repository,
        ParserSummaryRouteKind::FileSummary,
        &[batch],
    )
    .await?;
    let rows = decode_modelica_parser_summary_response_rows(response_batches.as_slice())?;
    decode_modelica_parser_file_summary(ParserSummaryRouteKind::FileSummary, rows.as_slice())
}

pub(crate) fn fetch_modelica_parser_file_summary_blocking_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    source_text: &str,
) -> Result<ModelicaParserFileSummary, RepoIntelligenceError> {
    let runtime = modelica_parser_summary_runtime()?;
    let repository = repository.clone();
    let source_id = source_id.to_string();
    let source_id_for_task = source_id.clone();
    let source_text = source_text.to_string();
    let timeout_secs = modelica_file_summary_blocking_timeout_secs_for_repository(&repository)?;
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    runtime.spawn(async move {
        let result = fetch_modelica_parser_file_summary_for_repository(
            &repository,
            &source_id_for_task,
            &source_text,
        )
        .await;
        let _ = sender.send(result);
    });
    receiver
        .recv_timeout(Duration::from_secs(timeout_secs))
        .map_err(|error| match error {
            RecvTimeoutError::Timeout => RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "Modelica parser-summary file-summary task exceeded {timeout_secs}s for `{source_id}`"
                ),
            },
            RecvTimeoutError::Disconnected => RepoIntelligenceError::AnalysisFailed {
                message: "Modelica parser-summary file-summary task stopped before returning"
                    .to_string(),
            },
        })?
}

fn modelica_file_summary_blocking_timeout_secs_for_repository(
    repository: &RegisteredRepository,
) -> Result<u64, RepoIntelligenceError> {
    modelica_parser_summary_timeout_secs_for_repository(
        repository,
        ParserSummaryRouteKind::FileSummary,
    )
}

pub(crate) fn validate_modelica_parser_summary_preflight_for_repository(
    repository: &RegisteredRepository,
) -> Result<(), RepoIntelligenceError> {
    let _client = build_modelica_parser_summary_flight_transport_client(
        repository,
        ParserSummaryRouteKind::FileSummary,
    )?;
    Ok(())
}

fn modelica_parser_summary_runtime()
-> Result<&'static tokio::runtime::Runtime, RepoIntelligenceError> {
    MODELICA_PARSER_SUMMARY_RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .thread_name("wendao-modelica-parser-summary")
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|message| RepoIntelligenceError::AnalysisFailed {
            message: format!("failed to build shared Modelica parser-summary runtime: {message}"),
        })
}

#[cfg(test)]
fn shared_modelica_parser_summary_runtime_identity_for_tests()
-> Result<usize, RepoIntelligenceError> {
    let runtime = modelica_parser_summary_runtime()?;
    Ok(std::ptr::from_ref(runtime) as usize)
}

#[cfg(test)]
#[path = "../../../tests/unit/modelica_plugin/parser_summary_fetch.rs"]
mod tests;
