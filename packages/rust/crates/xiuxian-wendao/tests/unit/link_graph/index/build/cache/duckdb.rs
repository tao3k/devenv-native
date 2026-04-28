use crate::link_graph::index::LinkGraphIndex;
use std::fs;
use std::path::Path;

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
    let payload_sizes = connection
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
                Ok((
                    docs.len(),
                    sections.len(),
                    edges.len(),
                    aliases.len(),
                    page_index.len(),
                ))
            },
        )
        .unwrap_or_else(|error| panic!("read DuckDB cache Arrow payloads: {error}"));
    assert!(payload_sizes.0 > 0, "docs Arrow IPC payload is empty");
    assert!(payload_sizes.1 > 0, "sections Arrow IPC payload is empty");
    assert!(payload_sizes.2 > 0, "edges Arrow IPC payload is empty");
    assert!(payload_sizes.3 > 0, "aliases Arrow IPC payload is empty");
    assert!(payload_sizes.4 > 0, "page-index residual payload is empty");

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
