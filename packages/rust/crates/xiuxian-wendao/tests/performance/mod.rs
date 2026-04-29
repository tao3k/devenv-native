pub(crate) mod support;

#[cfg(feature = "zhenfa-router")]
mod document_extract_flight;
#[cfg(all(feature = "duckdb", feature = "julia", feature = "zhenfa-router"))]
mod flightsql_statement;
#[cfg(feature = "zhenfa-router")]
mod gateway_search;
mod latency_related_search;
#[cfg(feature = "duckdb")]
mod local_duckdb_cache;
#[cfg(feature = "duckdb")]
mod parquet_query_engine;
#[cfg(feature = "document-extract-pdf-inspector")]
mod pdf_inspector_detect_audit;
mod throughput_related_search;
