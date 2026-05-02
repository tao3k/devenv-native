//! Fetch helpers for Modelica AST query analysis routes.

use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

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
    let source_id = source_id.as_str();
    let request = ModelicaAstQueryRequest {
        request_id: format!("modelica-ast-query:{source_id}"),
        source_id: source_id.to_string(),
        source_text: source_text.to_string(),
        limit: Some(DEFAULT_MODELICA_PACKAGE_AST_QUERY_LIMIT),
    };
    let batch = build_modelica_ast_query_request_batch(&[request])?;
    let repository = repository.clone();
    let source_id = source_id.to_string();
    let source_id_for_task = source_id.clone();
    let timeout_secs = modelica_ast_query_blocking_timeout_secs_for_repository(&repository)?;
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!("failed to build Modelica ast-query runtime: {error}"),
        })?;
    std::thread::spawn(move || {
        let result = runtime.block_on(async move {
            let response_batches: Vec<arrow::record_batch::RecordBatch> =
                process_modelica_parser_summary_flight_batches_for_repository(
                    &repository,
                    ParserSummaryRouteKind::AstQuery,
                    &[batch],
                )
                .await?;
            let rows = decode_modelica_ast_query_response_rows(response_batches.as_slice())?;
            decode_modelica_ast_query_analysis(
                repository.id.as_str(),
                source_id_for_task.as_str(),
                &rows,
            )
        });
        let _ = sender.send(result);
    });
    receiver
        .recv_timeout(Duration::from_secs(timeout_secs))
        .map_err(|error| match error {
            RecvTimeoutError::Timeout => RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "Modelica ast-query task exceeded {timeout_secs}s for `{source_id}`"
                ),
            },
            RecvTimeoutError::Disconnected => RepoIntelligenceError::AnalysisFailed {
                message: "Modelica ast-query task stopped before returning".to_string(),
            },
        })?
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
