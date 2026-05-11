//! Sample Arrow Flight server host used by the runtime binary entrypoint.

use std::io::{self, Write};
use std::sync::Arc;

use arrow_flight::flight_service_server::FlightServiceServer;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use xiuxian_db_store::{
    LanceDataType, LanceField, LanceFloat64Array, LanceInt32Array, LanceListArray,
    LanceListBuilder, LanceRecordBatch, LanceSchema, LanceStringArray, LanceStringBuilder,
};

use crate::transport::{
    EffectiveRerankFlightHostSettings, EffectiveRerankFlightHostSettingsInput,
    REPO_SEARCH_BEST_SECTION_COLUMN, REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_HIERARCHY_COLUMN,
    REPO_SEARCH_LANGUAGE_COLUMN, REPO_SEARCH_MATCH_REASON_COLUMN,
    REPO_SEARCH_NAVIGATION_CATEGORY_COLUMN, REPO_SEARCH_NAVIGATION_LINE_COLUMN,
    REPO_SEARCH_NAVIGATION_LINE_END_COLUMN, REPO_SEARCH_NAVIGATION_PATH_COLUMN,
    REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TAGS_COLUMN,
    REPO_SEARCH_TITLE_COLUMN, rerank_score_weights_from_env,
    resolve_effective_rerank_flight_host_settings, split_rerank_flight_host_overrides,
};

use super::WendaoFlightService;

/// Result surface for the sample Flight host application boundary.
pub type SampleFlightHostResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Run the sample Wendao Flight server from binary argument values.
///
/// # Errors
///
/// Returns an error when argument parsing, sample batch construction, socket
/// binding, or server execution fails.
pub async fn run_wendao_flight_server_from_args(
    args: impl IntoIterator<Item = String>,
) -> SampleFlightHostResult<()> {
    let host_args = parse_sample_host_args(args)?;
    let listener = TcpListener::bind(host_args.bind_addr).await?;
    let address = listener.local_addr()?;
    let query_response_batch = sample_repo_search_batch()?;
    writeln!(io::stdout(), "READY http://{address}")?;
    io::stdout().flush()?;

    let service = WendaoFlightService::new_with_weights(
        host_args.effective_settings.expected_schema_version,
        query_response_batch,
        host_args.effective_settings.rerank_dimension,
        host_args.effective_settings.rerank_weights,
    )?;

    Server::builder()
        .add_service(FlightServiceServer::new(service))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await?;

    Ok(())
}

struct SampleHostArgs {
    bind_addr: String,
    effective_settings: EffectiveRerankFlightHostSettings,
}

fn parse_sample_host_args(
    args: impl IntoIterator<Item = String>,
) -> SampleFlightHostResult<SampleHostArgs> {
    let mut args = args.into_iter();
    let bind_addr = args.next().unwrap_or_else(|| "127.0.0.1:0".to_string());
    let parsed_overrides = split_rerank_flight_host_overrides(args)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let positional_rerank_dimension =
        parse_positional_rerank_dimension(parsed_overrides.positional_args)?;
    let effective_settings =
        resolve_effective_rerank_flight_host_settings(EffectiveRerankFlightHostSettingsInput {
            schema_version_override: parsed_overrides.schema_version_override,
            rerank_dimension_override: parsed_overrides.rerank_dimension_override,
            file_backed_schema_version: None,
            file_backed_weights: None,
            fallback_dimension: positional_rerank_dimension,
            fallback_weights: rerank_score_weights_from_env().map_err(io::Error::other)?,
        });
    Ok(SampleHostArgs {
        bind_addr,
        effective_settings,
    })
}

fn parse_positional_rerank_dimension(positional_args: Vec<String>) -> io::Result<usize> {
    positional_args
        .into_iter()
        .next()
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
        .map(|dimension| dimension.unwrap_or(3))
}

fn sample_repo_search_batch() -> Result<LanceRecordBatch, Box<dyn std::error::Error>> {
    Ok(LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new(REPO_SEARCH_DOC_ID_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(REPO_SEARCH_PATH_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(REPO_SEARCH_TITLE_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(REPO_SEARCH_BEST_SECTION_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(REPO_SEARCH_MATCH_REASON_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(
                REPO_SEARCH_NAVIGATION_PATH_COLUMN,
                LanceDataType::Utf8,
                false,
            ),
            LanceField::new(
                REPO_SEARCH_NAVIGATION_CATEGORY_COLUMN,
                LanceDataType::Utf8,
                false,
            ),
            LanceField::new(
                REPO_SEARCH_NAVIGATION_LINE_COLUMN,
                LanceDataType::Int32,
                false,
            ),
            LanceField::new(
                REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
                LanceDataType::Int32,
                false,
            ),
            LanceField::new(
                REPO_SEARCH_HIERARCHY_COLUMN,
                LanceDataType::List(Arc::new(LanceField::new("item", LanceDataType::Utf8, true))),
                false,
            ),
            LanceField::new(
                REPO_SEARCH_TAGS_COLUMN,
                LanceDataType::List(Arc::new(LanceField::new("item", LanceDataType::Utf8, true))),
                false,
            ),
            LanceField::new(REPO_SEARCH_SCORE_COLUMN, LanceDataType::Float64, false),
            LanceField::new(REPO_SEARCH_LANGUAGE_COLUMN, LanceDataType::Utf8, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(vec!["doc-1"])),
            Arc::new(LanceStringArray::from(vec!["src/lib.rs"])),
            Arc::new(LanceStringArray::from(vec!["Repo Search Result"])),
            Arc::new(LanceStringArray::from(vec!["symbol"])),
            Arc::new(LanceStringArray::from(vec!["static_sample"])),
            Arc::new(LanceStringArray::from(vec!["src/lib.rs"])),
            Arc::new(LanceStringArray::from(vec!["file"])),
            Arc::new(LanceInt32Array::from(vec![1_i32])),
            Arc::new(LanceInt32Array::from(vec![1_i32])),
            Arc::new(build_utf8_list_array(&[&[
                "src".to_string(),
                "lib.rs".to_string(),
            ]])),
            Arc::new(build_utf8_list_array(&[&["lang:rust".to_string()]])),
            Arc::new(LanceFloat64Array::from(vec![0.91_f64])),
            Arc::new(LanceStringArray::from(vec!["rust"])),
        ],
    )?)
}

fn build_utf8_list_array(rows: &[&[String]]) -> LanceListArray {
    let mut builder = LanceListBuilder::new(LanceStringBuilder::new());
    for row in rows {
        for value in *row {
            builder.values().append_value(value);
        }
        builder.append(true);
    }
    builder.finish()
}
