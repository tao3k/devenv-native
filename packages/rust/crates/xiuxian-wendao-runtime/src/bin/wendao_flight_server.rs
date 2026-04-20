//! Runtime-owned Arrow Flight server binary for the stable Wendao query and rerank routes.

#[cfg(not(feature = "transport"))]
use std::io;
#[cfg(feature = "transport")]
use std::io::{self, Write};
#[cfg(feature = "transport")]
use std::sync::Arc;

#[cfg(feature = "transport")]
use arrow_flight::flight_service_server::FlightServiceServer;
#[cfg(feature = "transport")]
use tokio::net::TcpListener;
#[cfg(feature = "transport")]
use tokio_stream::wrappers::TcpListenerStream;
#[cfg(feature = "transport")]
use tonic::transport::Server;
#[cfg(feature = "transport")]
use xiuxian_db_store::{
    LanceDataType, LanceField, LanceFloat64Array, LanceInt32Array, LanceListArray,
    LanceListBuilder, LanceRecordBatch, LanceSchema, LanceStringArray, LanceStringBuilder,
};
#[cfg(feature = "transport")]
use xiuxian_wendao_runtime::transport::{
    EffectiveRerankFlightHostSettings, REPO_SEARCH_BEST_SECTION_COLUMN, REPO_SEARCH_DOC_ID_COLUMN,
    REPO_SEARCH_HIERARCHY_COLUMN, REPO_SEARCH_LANGUAGE_COLUMN, REPO_SEARCH_MATCH_REASON_COLUMN,
    REPO_SEARCH_NAVIGATION_CATEGORY_COLUMN, REPO_SEARCH_NAVIGATION_LINE_COLUMN,
    REPO_SEARCH_NAVIGATION_LINE_END_COLUMN, REPO_SEARCH_NAVIGATION_PATH_COLUMN,
    REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TAGS_COLUMN,
    REPO_SEARCH_TITLE_COLUMN, WendaoFlightService, rerank_score_weights_from_env,
    resolve_effective_rerank_flight_host_settings, split_rerank_flight_host_overrides,
};

#[cfg(not(feature = "transport"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err(io::Error::other("`wendao_flight_server` requires the `transport` feature").into())
}

#[cfg(feature = "transport")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let bind_addr = args.next().unwrap_or_else(|| "127.0.0.1:0".to_string());
    let parsed_overrides = split_rerank_flight_host_overrides(args)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let mut positional_args = parsed_overrides.positional_args.into_iter();
    let positional_rerank_dimension = positional_args
        .next()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .unwrap_or(3);
    let effective_settings: EffectiveRerankFlightHostSettings =
        resolve_effective_rerank_flight_host_settings(
            parsed_overrides.schema_version_override,
            parsed_overrides.rerank_dimension_override,
            None,
            None,
            positional_rerank_dimension,
            rerank_score_weights_from_env().map_err(io::Error::other)?,
        );

    let listener = TcpListener::bind(bind_addr).await?;
    let address = listener.local_addr()?;
    let query_response_batch = LanceRecordBatch::try_new(
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
    )?;
    writeln!(io::stdout(), "READY http://{address}")?;
    io::stdout().flush()?;

    let service = WendaoFlightService::new_with_weights(
        effective_settings.expected_schema_version,
        query_response_batch,
        effective_settings.rerank_dimension,
        effective_settings.rerank_weights,
    )?;

    Server::builder()
        .add_service(FlightServiceServer::new(service))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await?;

    Ok(())
}

#[cfg(feature = "transport")]
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
