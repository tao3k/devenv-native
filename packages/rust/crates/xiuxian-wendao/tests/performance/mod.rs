pub(crate) mod support;

#[cfg(all(feature = "duckdb", feature = "zhenfa-router"))]
mod flightsql_statement;
#[cfg(feature = "zhenfa-router")]
mod gateway_search;
mod latency_related_search;
#[cfg(feature = "duckdb")]
mod local_duckdb_cache;
#[cfg(feature = "duckdb")]
mod parquet_query_engine;
mod throughput_related_search;
