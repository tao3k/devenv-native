//! Fetch helpers for Modelica AST query analysis routes.

use std::{
    future::Future,
    sync::{OnceLock, mpsc},
    time::Duration,
};

use xiuxian_wendao_core::repo_intelligence::RepositoryAnalysisOutput;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepoIntelligenceError};

use super::contract::{
    ModelicaAstQueryRequest, build_modelica_ast_query_request_batch,
    decode_modelica_ast_query_analysis, decode_modelica_ast_query_response_rows,
};
use crate::modelica_plugin::parser_summary::{
    ParserSummaryRouteKind, modelica_parser_summary_timeout_secs_for_repository,
    process_modelica_parser_summary_flight_batches_for_repository,
};
use crate::modelica_plugin::types::ModelicaSourceId;

const DEFAULT_MODELICA_PACKAGE_AST_QUERY_LIMIT: i64 = 128;
static MODELICA_AST_QUERY_RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> =
    OnceLock::new();

async fn fetch_modelica_ast_query_analysis_for_repository(
    repository: &RegisteredRepository,
    source_id: &str,
    source_text: &str,
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let request = ModelicaAstQueryRequest {
        request_id: format!("modelica-ast-query:{source_id}"),
        source_id: source_id.to_string(),
        source_text: source_text.to_string(),
        limit: Some(DEFAULT_MODELICA_PACKAGE_AST_QUERY_LIMIT),
    };
    let batch = build_modelica_ast_query_request_batch(&[request])?;
    let response_batches: Vec<arrow::record_batch::RecordBatch> =
        process_modelica_parser_summary_flight_batches_for_repository(
            repository,
            ParserSummaryRouteKind::AstQuery,
            &[batch],
        )
        .await?;
    let rows = decode_modelica_ast_query_response_rows(response_batches.as_slice())?;
    decode_modelica_ast_query_analysis(repository.id.as_str(), source_id, &rows)
}

/// Build one bounded Modelica AST-query request, execute the configured Julia
/// Flight roundtrip, and materialize a lightweight repository analysis payload.
///
/// This owner exists for package-file code-AST previews where the full
/// Modelica file-summary row surface is too expensive to ship through the
/// parser-summary transport.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request cannot be materialized,
/// the repository does not declare a usable Modelica Flight transport, the
/// roundtrip fails, or the response violates the staged AST-query contract.
pub fn fetch_modelica_ast_query_analysis_blocking_for_repository(
    repository: &RegisteredRepository,
    source_id: ModelicaSourceId<'_>,
    source_text: &str,
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let source_id = source_id.as_str().to_string();
    let repository = repository.clone();
    let source_text = source_text.to_string();
    let source_id_for_error = source_id.clone();
    let timeout_secs = modelica_ast_query_blocking_timeout_secs_for_repository(&repository)?;
    let runtime = modelica_ast_query_runtime()?;
    let result = run_modelica_ast_query_blocking(runtime, async move {
        tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            fetch_modelica_ast_query_analysis_for_repository(
                &repository,
                source_id.as_str(),
                &source_text,
            ),
        )
        .await
    })
    .map_err(|_| RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Modelica ast-query task exceeded {timeout_secs}s for `{source_id_for_error}`"
        ),
    })?;
    let result = result?;
    Ok(result)
}

fn run_modelica_ast_query_blocking<T, TFuture>(
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
            receiver
                .recv()
                .expect("failed to execute modelica ast-query task in blocking helper")
        }
    } else {
        runtime.block_on(future)
    }
}

fn modelica_ast_query_runtime() -> Result<&'static tokio::runtime::Runtime, RepoIntelligenceError> {
    MODELICA_AST_QUERY_RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .thread_name("wendao-modelica-ast-query")
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|message| RepoIntelligenceError::AnalysisFailed {
            message: format!("failed to build shared Modelica ast-query runtime: {message}"),
        })
}

fn modelica_ast_query_blocking_timeout_secs_for_repository(
    repository: &RegisteredRepository,
) -> Result<u64, RepoIntelligenceError> {
    modelica_parser_summary_timeout_secs_for_repository(
        repository,
        ParserSummaryRouteKind::AstQuery,
    )
}

#[cfg(test)]
#[path = "../../../tests/unit/modelica_plugin/ast_query.rs"]
mod tests;
