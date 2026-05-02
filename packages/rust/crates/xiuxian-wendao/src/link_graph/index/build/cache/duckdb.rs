use super::CacheLookupOutcome;
use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::index::build::fingerprint::LinkGraphFingerprint;
use std::path::{Path, PathBuf};

#[cfg(feature = "duckdb")]
use super::arrow_snapshot::{
    LINK_GRAPH_DUCKDB_ARROW_CACHE_SCHEMA_VERSION, LinkGraphArrowSnapshotPayload,
    decode_arrow_cached_index_payload, duckdb_arrow_cache_schema_fingerprint,
    encode_arrow_cached_index_payload,
};
#[cfg(feature = "duckdb")]
use duckdb::{AccessMode, Config, Connection, params};
#[cfg(feature = "duckdb")]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(feature = "duckdb")]
use xiuxian_config_core::resolve_cache_home;
#[cfg(feature = "duckdb")]
use xiuxian_db_store::duckdb::{
    DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig, open_duckdb_connection,
};

#[cfg(feature = "duckdb")]
const LINK_GRAPH_LOCAL_CACHE_TABLE: &str = "link_graph_index_cache";
#[cfg(feature = "duckdb")]
const LOCAL_DUCKDB_THREADS: u64 = 2;
#[cfg(feature = "duckdb")]
const LINK_GRAPH_LOCAL_CACHE_COLUMNS: &[&str] = &[
    "slot_key",
    "schema_version",
    "schema_fingerprint",
    "root",
    "include_dirs_json",
    "excluded_dirs_json",
    "fingerprint_json",
    "docs_ipc",
    "sections_ipc",
    "edges_ipc",
    "aliases_ipc",
    "passages_json",
    "attachments_json",
    "page_index_json",
    "updated_at_unix_ms",
];

#[cfg(feature = "duckdb")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CacheTableShape {
    Missing,
    Compatible,
    Incompatible,
}

#[cfg(feature = "duckdb")]
fn unix_epoch_millis() -> i64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    i64::try_from(millis).unwrap_or(i64::MAX)
}

#[cfg(feature = "duckdb")]
fn local_duckdb_runtime(cache_path: &Path) -> DuckDbRuntimeConfig {
    let cache_dir = cache_path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    DuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::File(cache_path.to_path_buf()),
        temp_directory: cache_dir.join("tmp"),
        threads: LOCAL_DUCKDB_THREADS,
        execution: DuckDbExecutionConfig {
            preserve_insertion_order: true,
            parquet_metadata_cache: false,
            prefer_virtual_arrow: true,
        },
        memory_limit: None,
        max_temp_directory_size: None,
        materialize_threshold_rows: 200_000,
    }
}

#[cfg(feature = "duckdb")]
fn open_local_cache_connection(cache_path: &Path) -> Result<Connection, String> {
    let runtime = local_duckdb_runtime(cache_path);
    open_duckdb_connection(&runtime).map_err(|error| format!("link-graph local cache {error}"))
}

#[cfg(feature = "duckdb")]
fn open_local_cache_read_connection(cache_path: &Path) -> Result<Option<Connection>, String> {
    if !cache_path.exists() {
        return Ok(None);
    }
    let config = Config::default()
        .access_mode(AccessMode::ReadOnly)
        .map_err(|error| format!("failed to configure read-only DuckDB cache: {error}"))?;
    Connection::open_with_flags(cache_path, config)
        .map(Some)
        .map_err(|error| {
            format!(
                "failed to open read-only link-graph DuckDB cache `{}`: {error}",
                cache_path.display()
            )
        })
}

#[cfg(feature = "duckdb")]
fn ensure_cache_table(connection: &Connection) -> Result<(), String> {
    drop_incompatible_cache_table(connection)?;
    connection
        .execute_batch(
            format!(
                "CREATE TABLE IF NOT EXISTS {LINK_GRAPH_LOCAL_CACHE_TABLE} (
                    slot_key TEXT PRIMARY KEY,
                    schema_version TEXT NOT NULL,
                    schema_fingerprint TEXT NOT NULL,
                    root TEXT NOT NULL,
                    include_dirs_json TEXT NOT NULL,
                    excluded_dirs_json TEXT NOT NULL,
                    fingerprint_json TEXT NOT NULL,
                    docs_ipc BLOB NOT NULL,
                    sections_ipc BLOB NOT NULL,
                    edges_ipc BLOB NOT NULL,
                    aliases_ipc BLOB NOT NULL,
                    passages_json TEXT NOT NULL,
                    attachments_json TEXT NOT NULL,
                    page_index_json TEXT NOT NULL,
                    updated_at_unix_ms BIGINT NOT NULL
                );"
            )
            .as_str(),
        )
        .map_err(|error| format!("failed to initialize link-graph DuckDB cache table: {error}"))
}

#[cfg(feature = "duckdb")]
fn cache_table_shape(connection: &Connection) -> Result<CacheTableShape, String> {
    let table_count = connection
        .query_row(
            "SELECT COUNT(*)
             FROM information_schema.tables
             WHERE table_name = ?",
            params![LINK_GRAPH_LOCAL_CACHE_TABLE],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("failed to inspect DuckDB cache table existence: {error}"))?;
    if table_count == 0 {
        return Ok(CacheTableShape::Missing);
    }

    let total_columns = connection
        .query_row(
            format!(
                "SELECT COUNT(*)
                 FROM pragma_table_info('{LINK_GRAPH_LOCAL_CACHE_TABLE}')"
            )
            .as_str(),
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("failed to inspect DuckDB cache table shape: {error}"))?;
    if total_columns == 0 {
        return Ok(CacheTableShape::Missing);
    }

    let expected_columns_sql = LINK_GRAPH_LOCAL_CACHE_COLUMNS
        .iter()
        .map(|column| format!("'{column}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let expected_columns = connection
        .query_row(
            format!(
                "SELECT COUNT(*)
             FROM pragma_table_info('{LINK_GRAPH_LOCAL_CACHE_TABLE}')
             WHERE name IN ({expected_columns_sql})"
            )
            .as_str(),
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("failed to inspect DuckDB cache table columns: {error}"))?;
    let expected_column_count = i64::try_from(LINK_GRAPH_LOCAL_CACHE_COLUMNS.len())
        .map_err(|error| format!("failed to convert DuckDB cache table shape length: {error}"))?;
    if total_columns == expected_column_count && expected_columns == expected_column_count {
        Ok(CacheTableShape::Compatible)
    } else {
        Ok(CacheTableShape::Incompatible)
    }
}

#[cfg(feature = "duckdb")]
fn drop_incompatible_cache_table(connection: &Connection) -> Result<(), String> {
    match cache_table_shape(connection)? {
        CacheTableShape::Missing | CacheTableShape::Compatible => Ok(()),
        CacheTableShape::Incompatible => connection
            .execute_batch(format!("DROP TABLE IF EXISTS {LINK_GRAPH_LOCAL_CACHE_TABLE};").as_str())
            .map_err(|error| format!("failed to drop incompatible DuckDB cache table: {error}")),
    }
}

#[cfg(feature = "duckdb")]
fn cache_lookup_prepare_miss_reason(error: &duckdb::Error) -> &'static str {
    let message = error.to_string();
    if message.contains(LINK_GRAPH_LOCAL_CACHE_TABLE) && message.contains("does not exist") {
        "key_not_found"
    } else {
        "cache_table_shape_mismatch"
    }
}

#[cfg(feature = "duckdb")]
fn validate_cache_row_metadata(
    row: &duckdb::Row<'_>,
    root: &Path,
    include_dirs: &[String],
    excluded_dirs: &[String],
    fingerprint: &LinkGraphFingerprint,
) -> Result<Option<&'static str>, String> {
    let row_schema_version = row.get::<_, String>(0).map_err(|error| {
        format!("failed to decode link-graph DuckDB cache schema version: {error}")
    })?;
    if row_schema_version != LINK_GRAPH_DUCKDB_ARROW_CACHE_SCHEMA_VERSION {
        return Ok(Some("schema_version_mismatch"));
    }
    let row_schema_fingerprint = row.get::<_, String>(1).map_err(|error| {
        format!("failed to decode link-graph DuckDB cache schema fingerprint: {error}")
    })?;
    if row_schema_fingerprint != duckdb_arrow_cache_schema_fingerprint() {
        return Ok(Some("schema_fingerprint_mismatch"));
    }
    let row_root = row
        .get::<_, String>(2)
        .map_err(|error| format!("failed to decode link-graph DuckDB cache root: {error}"))?;
    if Path::new(&row_root) != root {
        return Ok(Some("root_mismatch"));
    }
    let include_dirs_json = row.get::<_, String>(3).map_err(|error| {
        format!("failed to decode link-graph DuckDB cache include dirs: {error}")
    })?;
    let Ok(row_include_dirs) = serde_json::from_str::<Vec<String>>(&include_dirs_json) else {
        return Ok(Some("include_dirs_parse_error"));
    };
    if row_include_dirs != include_dirs {
        return Ok(Some("include_dirs_mismatch"));
    }
    let excluded_dirs_json = row.get::<_, String>(4).map_err(|error| {
        format!("failed to decode link-graph DuckDB cache excluded dirs: {error}")
    })?;
    let Ok(row_excluded_dirs) = serde_json::from_str::<Vec<String>>(&excluded_dirs_json) else {
        return Ok(Some("excluded_dirs_parse_error"));
    };
    if row_excluded_dirs != excluded_dirs {
        return Ok(Some("excluded_dirs_mismatch"));
    }
    let fingerprint_json = row.get::<_, String>(5).map_err(|error| {
        format!("failed to decode link-graph DuckDB cache fingerprint: {error}")
    })?;
    let Ok(row_fingerprint) = serde_json::from_str::<LinkGraphFingerprint>(&fingerprint_json)
    else {
        return Ok(Some("fingerprint_parse_error"));
    };
    if &row_fingerprint != fingerprint {
        return Ok(Some("content_fingerprint_mismatch"));
    }
    Ok(None)
}

#[cfg(feature = "duckdb")]
fn read_cache_row_payload(row: &duckdb::Row<'_>) -> Result<LinkGraphArrowSnapshotPayload, String> {
    Ok(LinkGraphArrowSnapshotPayload {
        docs_ipc: row
            .get::<_, Vec<u8>>(6)
            .map_err(|error| format!("failed to decode link-graph DuckDB docs stream: {error}"))?,
        sections_ipc: row.get::<_, Vec<u8>>(7).map_err(|error| {
            format!("failed to decode link-graph DuckDB sections stream: {error}")
        })?,
        edges_ipc: row
            .get::<_, Vec<u8>>(8)
            .map_err(|error| format!("failed to decode link-graph DuckDB edges stream: {error}"))?,
        aliases_ipc: row.get::<_, Vec<u8>>(9).map_err(|error| {
            format!("failed to decode link-graph DuckDB aliases stream: {error}")
        })?,
        passages_json: row.get::<_, String>(10).map_err(|error| {
            format!("failed to decode link-graph DuckDB passages residuals: {error}")
        })?,
        attachments_json: row.get::<_, String>(11).map_err(|error| {
            format!("failed to decode link-graph DuckDB attachments residuals: {error}")
        })?,
        page_index_json: row.get::<_, String>(12).map_err(|error| {
            format!("failed to decode link-graph DuckDB page-index residuals: {error}")
        })?,
    })
}

#[cfg(feature = "duckdb")]
pub(in crate::link_graph::index::build) fn default_local_duckdb_cache_path(root: &Path) -> PathBuf {
    resolve_cache_home(Some(root))
        .unwrap_or_else(|| root.join(".cache"))
        .join("wendao")
        .join("link_graph")
        .join("index_cache.duckdb")
}

#[cfg(not(feature = "duckdb"))]
pub(in crate::link_graph::index::build) fn default_local_duckdb_cache_path(root: &Path) -> PathBuf {
    root.join(".cache")
        .join("wendao")
        .join("link_graph")
        .join("index_cache.duckdb")
}

#[cfg(feature = "duckdb")]
pub(in crate::link_graph::index::build) fn load_cached_index_from_duckdb(
    cache_path: &Path,
    slot_key: &str,
    root: &Path,
    include_dirs: &[String],
    excluded_dirs: &[String],
    fingerprint: &LinkGraphFingerprint,
) -> Result<CacheLookupOutcome, String> {
    let Some(connection) = open_local_cache_read_connection(cache_path)? else {
        return Ok(CacheLookupOutcome::Miss("key_not_found"));
    };
    load_cached_index_from_duckdb_connection(
        &connection,
        slot_key,
        root,
        include_dirs,
        excluded_dirs,
        fingerprint,
    )
}

#[cfg(feature = "duckdb")]
fn load_cached_index_from_duckdb_connection(
    connection: &Connection,
    slot_key: &str,
    root: &Path,
    include_dirs: &[String],
    excluded_dirs: &[String],
    fingerprint: &LinkGraphFingerprint,
) -> Result<CacheLookupOutcome, String> {
    let lookup_sql = format!(
        "SELECT
            schema_version,
            schema_fingerprint,
            root,
            include_dirs_json,
            excluded_dirs_json,
            fingerprint_json,
            docs_ipc,
            sections_ipc,
            edges_ipc,
            aliases_ipc,
            passages_json,
            attachments_json,
            page_index_json
        FROM {LINK_GRAPH_LOCAL_CACHE_TABLE}
        WHERE slot_key = ?"
    );
    let mut statement = match connection.prepare(lookup_sql.as_str()) {
        Ok(statement) => statement,
        Err(error) => {
            return Ok(CacheLookupOutcome::Miss(cache_lookup_prepare_miss_reason(
                &error,
            )));
        }
    };
    let mut rows = statement
        .query(params![slot_key])
        .map_err(|error| format!("failed to query link-graph DuckDB cache: {error}"))?;
    let Some(row) = rows
        .next()
        .map_err(|error| format!("failed to read link-graph DuckDB cache row: {error}"))?
    else {
        return Ok(CacheLookupOutcome::Miss("key_not_found"));
    };

    if let Some(reason) =
        validate_cache_row_metadata(row, root, include_dirs, excluded_dirs, fingerprint)?
    {
        return Ok(CacheLookupOutcome::Miss(reason));
    }

    let payload = read_cache_row_payload(row)?;
    match decode_arrow_cached_index_payload(
        &payload,
        root.to_path_buf(),
        include_dirs.to_vec(),
        excluded_dirs.to_vec(),
    ) {
        Ok(index) => Ok(CacheLookupOutcome::Hit(Box::new(index))),
        Err(_error) => Ok(CacheLookupOutcome::Miss("payload_parse_error")),
    }
}

#[cfg(not(feature = "duckdb"))]
pub(in crate::link_graph::index::build) fn load_cached_index_from_duckdb(
    _cache_path: &Path,
    _slot_key: &str,
    _root: &Path,
    _include_dirs: &[String],
    _excluded_dirs: &[String],
    _fingerprint: &LinkGraphFingerprint,
) -> CacheLookupOutcome {
    CacheLookupOutcome::Miss("duckdb_feature_disabled")
}

#[cfg(feature = "duckdb")]
pub(in crate::link_graph::index::build) fn save_cached_index_to_duckdb(
    index: &LinkGraphIndex,
    cache_path: &Path,
    slot_key: &str,
    fingerprint: &LinkGraphFingerprint,
) -> Result<(), String> {
    let connection = open_local_cache_connection(cache_path)?;
    ensure_cache_table(&connection)?;
    let include_dirs_json = serde_json::to_string(&index.include_dirs)
        .map_err(|error| format!("failed to serialize link-graph include dirs: {error}"))?;
    let excluded_dirs_json = serde_json::to_string(&index.excluded_dirs)
        .map_err(|error| format!("failed to serialize link-graph excluded dirs: {error}"))?;
    let fingerprint_json = serde_json::to_string(&fingerprint)
        .map_err(|error| format!("failed to serialize link-graph fingerprint: {error}"))?;
    let encoded = encode_arrow_cached_index_payload(index)?;
    connection
        .execute(
            format!(
                "INSERT INTO {LINK_GRAPH_LOCAL_CACHE_TABLE} (
                    slot_key,
                    schema_version,
                    schema_fingerprint,
                    root,
                    include_dirs_json,
                    excluded_dirs_json,
                    fingerprint_json,
                    docs_ipc,
                    sections_ipc,
                    edges_ipc,
                    aliases_ipc,
                    passages_json,
                    attachments_json,
                    page_index_json,
                    updated_at_unix_ms
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(slot_key) DO UPDATE SET
                    schema_version = excluded.schema_version,
                    schema_fingerprint = excluded.schema_fingerprint,
                    root = excluded.root,
                    include_dirs_json = excluded.include_dirs_json,
                    excluded_dirs_json = excluded.excluded_dirs_json,
                    fingerprint_json = excluded.fingerprint_json,
                    docs_ipc = excluded.docs_ipc,
                    sections_ipc = excluded.sections_ipc,
                    edges_ipc = excluded.edges_ipc,
                    aliases_ipc = excluded.aliases_ipc,
                    passages_json = excluded.passages_json,
                    attachments_json = excluded.attachments_json,
                    page_index_json = excluded.page_index_json,
                    updated_at_unix_ms = excluded.updated_at_unix_ms"
            )
            .as_str(),
            params![
                slot_key,
                LINK_GRAPH_DUCKDB_ARROW_CACHE_SCHEMA_VERSION,
                duckdb_arrow_cache_schema_fingerprint(),
                index.root.to_string_lossy().as_ref(),
                include_dirs_json,
                excluded_dirs_json,
                fingerprint_json,
                encoded.docs_ipc,
                encoded.sections_ipc,
                encoded.edges_ipc,
                encoded.aliases_ipc,
                encoded.passages_json,
                encoded.attachments_json,
                encoded.page_index_json,
                unix_epoch_millis(),
            ],
        )
        .map_err(|error| format!("failed to write link-graph DuckDB cache row: {error}"))?;
    Ok(())
}

#[cfg(not(feature = "duckdb"))]
pub(in crate::link_graph::index::build) fn save_cached_index_to_duckdb(
    _index: &LinkGraphIndex,
    _cache_path: &Path,
    _slot_key: &str,
    _fingerprint: &LinkGraphFingerprint,
) {
}

#[cfg(all(test, feature = "duckdb"))]
#[path = "../../../../../tests/unit/link_graph/index/build/cache/duckdb.rs"]
mod tests;
