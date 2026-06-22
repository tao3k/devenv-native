//! Minimal Rust Arrow Flight host for the stable attachment-search contract.

use std::collections::HashSet;
use std::io::{self, Write};
use std::sync::Arc;

use arrow_array::{
    ArrayRef, Float64Array, Int32Array, RecordBatch, StringArray,
    builder::{ListBuilder, StringBuilder},
};
use arrow_flight::flight_service_server::FlightServiceServer;
use arrow_schema::{DataType, Field, Schema};
use async_trait::async_trait;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use xiuxian_wendao_server::transport::{
    AttachmentSearchFlightRouteProvider, REPO_SEARCH_BEST_SECTION_COLUMN,
    REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_HIERARCHY_COLUMN, REPO_SEARCH_LANGUAGE_COLUMN,
    REPO_SEARCH_MATCH_REASON_COLUMN, REPO_SEARCH_NAVIGATION_CATEGORY_COLUMN,
    REPO_SEARCH_NAVIGATION_LINE_COLUMN, REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
    REPO_SEARCH_NAVIGATION_PATH_COLUMN, REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_SCORE_COLUMN,
    REPO_SEARCH_TAGS_COLUMN, REPO_SEARCH_TITLE_COLUMN, RepoSearchFlightRequest,
    RepoSearchFlightRouteProvider, RerankScoreWeights, SearchFlightRouteResponse,
    WendaoFlightRouteProviders, WendaoFlightService,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind_addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:0".to_string());
    let expected_schema_version = std::env::args().nth(2).unwrap_or_else(|| "v2".to_string());

    let listener = TcpListener::bind(bind_addr).await?;
    let address = listener.local_addr()?;
    let mut providers = WendaoFlightRouteProviders::new(Arc::new(ContractRepoSearchProvider));
    providers.attachment_search = Some(Arc::new(ContractAttachmentSearchProvider));
    let service = WendaoFlightService::new_with_route_providers(
        expected_schema_version,
        providers,
        3,
        RerankScoreWeights::default(),
    )?;

    writeln!(io::stdout(), "READY http://{address}")?;
    io::stdout().flush()?;

    Server::builder()
        .add_service(FlightServiceServer::new(service))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await?;

    Ok(())
}

#[derive(Debug)]
struct ContractRepoSearchProvider;

#[async_trait]
impl RepoSearchFlightRouteProvider for ContractRepoSearchProvider {
    async fn repo_search_batch(
        &self,
        _request: &RepoSearchFlightRequest,
    ) -> Result<RecordBatch, String> {
        empty_repo_search_batch()
    }
}

#[derive(Debug)]
struct ContractAttachmentSearchProvider;

#[async_trait]
impl AttachmentSearchFlightRouteProvider for ContractAttachmentSearchProvider {
    async fn attachment_search_batch_for_request(
        &self,
        request: xiuxian_wendao_server::transport::AttachmentSearchFlightRouteRequest<'_>,
    ) -> Result<SearchFlightRouteResponse, String> {
        self.attachment_search_batch(
            request.query_text,
            request.limit,
            request.ext_filters,
            request.kind_filters,
            request.case_sensitive,
        )
        .await
    }

    async fn attachment_search_batch(
        &self,
        query_text: &str,
        limit: usize,
        ext_filters: &HashSet<String>,
        kind_filters: &HashSet<String>,
        case_sensitive: bool,
    ) -> Result<SearchFlightRouteResponse, String> {
        let rows = attachment_rows(query_text, ext_filters, kind_filters, case_sensitive, limit);
        Ok(
            SearchFlightRouteResponse::new(attachment_batch(&rows)?).with_app_metadata(
                r#"{"selectedScope":"attachments","partial":false,"indexingState":"active"}"#,
            ),
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct AttachmentRow {
    name: &'static str,
    path: &'static str,
    source_id: &'static str,
    source_stem: &'static str,
    source_title: &'static str,
    navigation_target_json: &'static str,
    source_path: &'static str,
    attachment_id: &'static str,
    attachment_path: &'static str,
    attachment_name: &'static str,
    attachment_ext: &'static str,
    kind: &'static str,
    score: f64,
    vision_snippet: Option<&'static str>,
}

fn attachment_rows(
    query_text: &str,
    ext_filters: &HashSet<String>,
    kind_filters: &HashSet<String>,
    case_sensitive: bool,
    limit: usize,
) -> Vec<AttachmentRow> {
    static ROWS: &[AttachmentRow] = &[AttachmentRow {
        name: "attachment-live-link-fixture.svg",
        path: "docs/testing/assets/attachment-live-link-fixture.svg",
        source_id: "doc-testing-attachment-live-link-fixture",
        source_stem: "attachment-live-link-fixture",
        source_title: "Wendao Attachment Live Link Fixture",
        navigation_target_json: r#"{"path":"docs/testing/attachment-live-link-fixture.md","category":"doc","line":1,"lineEnd":6}"#,
        source_path: "docs/testing/attachment-live-link-fixture.md",
        attachment_id: "attachment-live-link-fixture-svg",
        attachment_path: "docs/testing/assets/attachment-live-link-fixture.svg",
        attachment_name: "attachment-live-link-fixture.svg",
        attachment_ext: "svg",
        kind: "image",
        score: 0.97,
        vision_snippet: Some(
            "Stable SVG fixture referenced by the Wendao attachment live link test document.",
        ),
    }];

    ROWS.iter()
        .copied()
        .filter(|row| row_matches_query(row, query_text, case_sensitive))
        .filter(|row| filter_matches(ext_filters, row.attachment_ext))
        .filter(|row| filter_matches(kind_filters, row.kind))
        .take(limit)
        .collect()
}

fn row_matches_query(row: &AttachmentRow, query_text: &str, case_sensitive: bool) -> bool {
    let needle = comparable(query_text, case_sensitive);
    [
        row.name,
        row.path,
        row.source_id,
        row.source_stem,
        row.source_title,
        row.source_path,
        row.attachment_id,
        row.attachment_path,
        row.attachment_name,
        row.attachment_ext,
        row.kind,
        row.vision_snippet.unwrap_or_default(),
    ]
    .iter()
    .any(|value| comparable(value, case_sensitive).contains(&needle))
}

fn comparable(value: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        value.to_string()
    } else {
        value.to_lowercase()
    }
}

fn filter_matches(filters: &HashSet<String>, value: &str) -> bool {
    filters.is_empty()
        || filters
            .iter()
            .map(|filter| filter.trim().trim_start_matches('.').to_lowercase())
            .any(|filter| filter == value)
}

fn attachment_batch(rows: &[AttachmentRow]) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("name", DataType::Utf8, false),
            Field::new("path", DataType::Utf8, false),
            Field::new("sourceId", DataType::Utf8, false),
            Field::new("sourceStem", DataType::Utf8, false),
            Field::new("sourceTitle", DataType::Utf8, false),
            Field::new("navigationTargetJson", DataType::Utf8, true),
            Field::new("sourcePath", DataType::Utf8, false),
            Field::new("attachmentId", DataType::Utf8, false),
            Field::new("attachmentPath", DataType::Utf8, false),
            Field::new("attachmentName", DataType::Utf8, false),
            Field::new("attachmentExt", DataType::Utf8, false),
            Field::new("kind", DataType::Utf8, false),
            Field::new("score", DataType::Float64, false),
            Field::new("visionSnippet", DataType::Utf8, true),
        ])),
        vec![
            string_column(rows.iter().map(|row| row.name)),
            string_column(rows.iter().map(|row| row.path)),
            string_column(rows.iter().map(|row| row.source_id)),
            string_column(rows.iter().map(|row| row.source_stem)),
            string_column(rows.iter().map(|row| row.source_title)),
            nullable_string_column(rows.iter().map(|row| Some(row.navigation_target_json))),
            string_column(rows.iter().map(|row| row.source_path)),
            string_column(rows.iter().map(|row| row.attachment_id)),
            string_column(rows.iter().map(|row| row.attachment_path)),
            string_column(rows.iter().map(|row| row.attachment_name)),
            string_column(rows.iter().map(|row| row.attachment_ext)),
            string_column(rows.iter().map(|row| row.kind)),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.score).collect::<Vec<_>>(),
            )) as ArrayRef,
            nullable_string_column(rows.iter().map(|row| row.vision_snippet)),
        ],
    )
    .map_err(|error| error.to_string())
}

fn empty_repo_search_batch() -> Result<RecordBatch, String> {
    let item_field = Arc::new(Field::new("item", DataType::Utf8, true));
    let list_type = DataType::List(Arc::clone(&item_field));
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(REPO_SEARCH_DOC_ID_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_PATH_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_TITLE_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_BEST_SECTION_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_MATCH_REASON_COLUMN, DataType::Utf8, false),
            Field::new(REPO_SEARCH_NAVIGATION_PATH_COLUMN, DataType::Utf8, false),
            Field::new(
                REPO_SEARCH_NAVIGATION_CATEGORY_COLUMN,
                DataType::Utf8,
                false,
            ),
            Field::new(REPO_SEARCH_NAVIGATION_LINE_COLUMN, DataType::Int32, false),
            Field::new(
                REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
                DataType::Int32,
                false,
            ),
            Field::new(REPO_SEARCH_HIERARCHY_COLUMN, list_type.clone(), false),
            Field::new(REPO_SEARCH_TAGS_COLUMN, list_type.clone(), false),
            Field::new(REPO_SEARCH_SCORE_COLUMN, DataType::Float64, false),
            Field::new(REPO_SEARCH_LANGUAGE_COLUMN, DataType::Utf8, false),
        ])),
        vec![
            empty_string_column(),
            empty_string_column(),
            empty_string_column(),
            empty_string_column(),
            empty_string_column(),
            empty_string_column(),
            empty_string_column(),
            Arc::new(Int32Array::from(Vec::<i32>::new())) as ArrayRef,
            Arc::new(Int32Array::from(Vec::<i32>::new())) as ArrayRef,
            empty_string_list_column(),
            empty_string_list_column(),
            Arc::new(Float64Array::from(Vec::<f64>::new())) as ArrayRef,
            empty_string_column(),
        ],
    )
    .map_err(|error| error.to_string())
}

fn string_column<'a>(values: impl Iterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from(values.collect::<Vec<_>>())) as ArrayRef
}

fn nullable_string_column<'a>(values: impl Iterator<Item = Option<&'a str>>) -> ArrayRef {
    Arc::new(StringArray::from(values.collect::<Vec<_>>())) as ArrayRef
}

fn empty_string_column() -> ArrayRef {
    Arc::new(StringArray::from(Vec::<&str>::new())) as ArrayRef
}

fn empty_string_list_column() -> ArrayRef {
    let mut builder = ListBuilder::new(StringBuilder::new());
    Arc::new(builder.finish()) as ArrayRef
}
