use crate::link_graph::index::LinkGraphIndex;
use arrow::ipc::reader::StreamReader;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;

fn write_note(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|error| panic!("create note parent: {error}"));
    }
    fs::write(path, content).unwrap_or_else(|error| panic!("write note: {error}"));
}

fn assert_arrow_cache_payloads_exist(cache_path: &Path) {
    let connection = duckdb::Connection::open(cache_path)
        .unwrap_or_else(|error| panic!("open DuckDB cache for shape assertion: {error}"));
    let payloads = connection
        .query_row(
            "SELECT docs_ipc, sections_ipc, edges_ipc, aliases_ipc, page_index_json
             FROM link_graph_index_cache",
            [],
            |row| {
                let docs = row.get::<_, Vec<u8>>(0)?;
                let sections = row.get::<_, Vec<u8>>(1)?;
                let edges = row.get::<_, Vec<u8>>(2)?;
                let aliases = row.get::<_, Vec<u8>>(3)?;
                let page_index = row.get::<_, String>(4)?;
                Ok((docs, sections, edges, aliases, page_index))
            },
        )
        .unwrap_or_else(|error| panic!("read DuckDB cache Arrow payloads: {error}"));
    assert!(!payloads.0.is_empty(), "docs Arrow IPC payload is empty");
    assert!(
        !payloads.1.is_empty(),
        "sections Arrow IPC payload is empty"
    );
    assert!(!payloads.2.is_empty(), "edges Arrow IPC payload is empty");
    assert!(!payloads.3.is_empty(), "aliases Arrow IPC payload is empty");
    assert!(
        !payloads.4.is_empty(),
        "page-index residual payload is empty"
    );
    assert_ipc_table_metadata(&payloads.0, "link_graph_snapshot_docs");
    assert_ipc_table_metadata(&payloads.1, "link_graph_snapshot_sections");
    assert_ipc_table_metadata(&payloads.2, "link_graph_snapshot_edges");
    assert_ipc_table_metadata(&payloads.3, "link_graph_snapshot_aliases");

    let legacy_payload_columns = connection
        .query_row(
            "SELECT COUNT(*)
             FROM pragma_table_info('link_graph_index_cache')
             WHERE name = 'payload_json'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or_else(|error| panic!("inspect DuckDB cache table columns: {error}"));
    assert_eq!(legacy_payload_columns, 0);
}

fn assert_ipc_table_metadata(payload: &[u8], expected_table: &str) {
    let mut reader = StreamReader::try_new(Cursor::new(payload), None)
        .unwrap_or_else(|error| panic!("decode Arrow IPC stream: {error}"));
    let Some(batch_result) = reader.next() else {
        panic!("Arrow IPC stream must contain one batch");
    };
    let batch = batch_result.unwrap_or_else(|error| panic!("decode Arrow IPC batch: {error}"));
    assert_eq!(
        batch
            .schema()
            .metadata()
            .get(WENDAO_TABLE_METADATA_KEY)
            .map(String::as_str),
        Some(expected_table)
    );
}

fn write_incompatible_cache_table(cache_path: &Path) {
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("create incompatible cache parent: {error}"));
    }
    let connection = duckdb::Connection::open(cache_path)
        .unwrap_or_else(|error| panic!("open incompatible DuckDB cache: {error}"));
    connection
        .execute_batch(
            "CREATE TABLE link_graph_index_cache (
                slot_key TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL
            );",
        )
        .unwrap_or_else(|error| panic!("create incompatible DuckDB cache table: {error}"));
}

#[test]
fn link_graph_local_duckdb_cache_reuses_snapshot() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let root = temp.path();
    write_note(root, "docs/a.md", "# Alpha\n\nLinks to [[Beta]].\n");
    write_note(root, "docs/b.md", "# Beta\n\nBack to [[Alpha]].\n");
    let cache_path = root.join(".cache-test").join("index.duckdb");
    let include_dirs = vec!["docs".to_string()];

    let (first, first_meta) = LinkGraphIndex::build_with_local_cache_path_with_meta(
        root,
        &include_dirs,
        &[],
        &cache_path,
    )
    .unwrap_or_else(|error| panic!("first local DuckDB cache build: {error}"));
    assert_eq!(first_meta.backend, "duckdb");
    assert_eq!(first_meta.status, "miss");
    assert_eq!(first_meta.miss_reason.as_deref(), Some("key_not_found"));
    assert_eq!(
        first_meta.schema_version,
        LinkGraphIndex::cache_schema_version()
    );
    assert!(!first_meta.schema_version.contains("valkey"));
    assert_eq!(first.docs_by_id.len(), 2);
    assert!(cache_path.exists());

    let (second, second_meta) = LinkGraphIndex::build_with_local_cache_path_with_meta(
        root,
        &include_dirs,
        &[],
        &cache_path,
    )
    .unwrap_or_else(|error| panic!("second local DuckDB cache build: {error}"));
    assert_eq!(second_meta.backend, "duckdb");
    assert_eq!(second_meta.status, "hit");
    assert_eq!(second_meta.miss_reason, None);
    assert_eq!(second.docs_by_id.len(), first.docs_by_id.len());
    assert!(
        second
            .page_index("docs/a")
            .is_some_and(|roots| !roots.is_empty())
    );
    assert_arrow_cache_payloads_exist(&cache_path);
}

#[test]
fn link_graph_local_duckdb_cache_rebuilds_incompatible_table_shape() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let root = temp.path();
    write_note(root, "docs/a.md", "# Alpha\n\nLinks to [[Beta]].\n");
    write_note(root, "docs/b.md", "# Beta\n\nBack to [[Alpha]].\n");
    let cache_path = root.join(".cache-test").join("index.duckdb");
    write_incompatible_cache_table(&cache_path);
    let include_dirs = vec!["docs".to_string()];

    let (rebuilt, meta) = LinkGraphIndex::build_with_local_cache_path_with_meta(
        root,
        &include_dirs,
        &[],
        &cache_path,
    )
    .unwrap_or_else(|error| panic!("local DuckDB cache rebuild from incompatible table: {error}"));
    assert_eq!(meta.backend, "duckdb");
    assert_eq!(meta.status, "miss");
    assert_eq!(
        meta.miss_reason.as_deref(),
        Some("cache_table_shape_mismatch")
    );
    assert_eq!(rebuilt.docs_by_id.len(), 2);
    assert_arrow_cache_payloads_exist(&cache_path);
}

#[test]
fn link_graph_local_duckdb_cache_rebuilds_when_fingerprint_changes() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let root = temp.path();
    write_note(root, "docs/a.md", "# Alpha\n\nOriginal body.\n");
    let cache_path = root.join(".cache-test").join("index.duckdb");
    let include_dirs = vec!["docs".to_string()];

    let (_, first_meta) = LinkGraphIndex::build_with_local_cache_path_with_meta(
        root,
        &include_dirs,
        &[],
        &cache_path,
    )
    .unwrap_or_else(|error| panic!("first local DuckDB cache build: {error}"));
    assert_eq!(first_meta.status, "miss");

    write_note(
        root,
        "docs/a.md",
        "# Alpha\n\nOriginal body with additional deterministic bytes.\n",
    );

    let (rebuilt, second_meta) = LinkGraphIndex::build_with_local_cache_path_with_meta(
        root,
        &include_dirs,
        &[],
        &cache_path,
    )
    .unwrap_or_else(|error| panic!("second local DuckDB cache build: {error}"));
    assert_eq!(second_meta.backend, "duckdb");
    assert_eq!(second_meta.status, "miss");
    assert_eq!(
        second_meta.miss_reason.as_deref(),
        Some("content_fingerprint_mismatch")
    );
    assert_eq!(rebuilt.docs_by_id.len(), 1);
    assert!(
        rebuilt
            .docs_by_id
            .values()
            .any(|doc| doc.search_text.contains("additional deterministic bytes"))
    );
}
