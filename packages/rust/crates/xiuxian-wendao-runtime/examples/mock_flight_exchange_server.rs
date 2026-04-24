//! Minimal Arrow Flight `do_exchange` mock server for Python transport smoke tests.

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
use xiuxian_wendao_runtime::transport::{
    REPO_SEARCH_BEST_SECTION_COLUMN, REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_HIERARCHY_COLUMN,
    REPO_SEARCH_LANGUAGE_COLUMN, REPO_SEARCH_MATCH_REASON_COLUMN,
    REPO_SEARCH_NAVIGATION_CATEGORY_COLUMN, REPO_SEARCH_NAVIGATION_LINE_COLUMN,
    REPO_SEARCH_NAVIGATION_LINE_END_COLUMN, REPO_SEARCH_NAVIGATION_PATH_COLUMN,
    REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TAGS_COLUMN,
    REPO_SEARCH_TITLE_COLUMN, WendaoFlightService,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind_addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:0".to_string());
    let expected_schema_version = std::env::args().nth(2).unwrap_or_else(|| "v2".to_string());

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
            Arc::new(LanceStringArray::from(vec![
                "7: Repo Search Result section",
            ])),
            Arc::new(LanceStringArray::from(vec!["repo_content_search"])),
            Arc::new(LanceStringArray::from(vec!["alpha/repo/src/lib.rs"])),
            Arc::new(LanceStringArray::from(vec!["repo_code"])),
            Arc::new(LanceInt32Array::from(vec![7_i32])),
            Arc::new(LanceInt32Array::from(vec![7_i32])),
            Arc::new(build_utf8_list_array(&[&[
                "src".to_string(),
                "lib.rs".to_string(),
            ]])),
            Arc::new(build_utf8_list_array(&[&[
                "code".to_string(),
                "file".to_string(),
                "lang:rust".to_string(),
            ]])),
            Arc::new(LanceFloat64Array::from(vec![0.91_f64])),
            Arc::new(LanceStringArray::from(vec!["rust"])),
        ],
    )?;
    writeln!(io::stdout(), "READY http://{address}")?;
    io::stdout().flush()?;

    let service = WendaoFlightService::new(expected_schema_version, query_response_batch, 3)?;

    Server::builder()
        .add_service(FlightServiceServer::new(service))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await?;

    Ok(())
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
