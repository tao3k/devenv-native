mod config;
mod materialize;
mod task_list;

fn assert_agent_task_row_count(database_path: &std::path::Path, expected: i64) {
    let connection = xiuxian_db_store::duckdb_crate::Connection::open(database_path)
        .unwrap_or_else(|error| panic!("open read-model duckdb: {error}"));
    let count = connection
        .query_row("SELECT COUNT(*) FROM agent_org_tasks", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or_else(|error| panic!("query read-model row count: {error}"));
    assert_eq!(count, expected);
}

fn assert_agent_memory_object_row_count(database_path: &std::path::Path, expected: i64) {
    let connection = xiuxian_db_store::duckdb_crate::Connection::open(database_path)
        .unwrap_or_else(|error| panic!("open read-model duckdb: {error}"));
    let count = connection
        .query_row("SELECT COUNT(*) FROM agent_org_memory_objects", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or_else(|error| panic!("query memory object row count: {error}"));
    assert_eq!(count, expected);
}

fn assert_agent_org_element_row_count_at_least(database_path: &std::path::Path, minimum: i64) {
    let connection = xiuxian_db_store::duckdb_crate::Connection::open(database_path)
        .unwrap_or_else(|error| panic!("open read-model duckdb: {error}"));
    let count = connection
        .query_row("SELECT COUNT(*) FROM agent_org_elements", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or_else(|error| panic!("query org element row count: {error}"));
    assert!(
        count >= minimum,
        "expected at least {minimum} org element row(s), got {count}"
    );
}

fn assert_agent_org_element_projection_exists(
    database_path: &std::path::Path,
    expected: (&str, &str, &str),
) {
    let connection = xiuxian_db_store::duckdb_crate::Connection::open(database_path)
        .unwrap_or_else(|error| panic!("open read-model duckdb: {error}"));
    let count = connection
        .query_row(
            "SELECT COUNT(*) FROM agent_org_elements \
             WHERE category = ?1 AND kind = ?2 AND contains(source_raw, ?3)",
            [expected.0, expected.1, expected.2],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or_else(|error| panic!("query org element projection: {error}"));
    assert!(count > 0, "missing org element projection: {expected:?}");
}

fn assert_agent_org_element_join_matches_text(
    database_path: &std::path::Path,
    needle: &str,
    expected_orgid: &str,
) {
    let connection = xiuxian_db_store::duckdb_crate::Connection::open(database_path)
        .unwrap_or_else(|error| panic!("open read-model duckdb: {error}"));
    let orgid = connection
        .query_row(
            "SELECT task.orgid \
             FROM agent_org_tasks AS task \
             JOIN agent_org_elements AS element \
               ON element.source_path = task.source_path \
              AND element.source_range_start >= task.source_range_start \
              AND element.source_range_start < task.source_range_end \
             WHERE contains(lower(element.source_raw), lower(?1)) \
             ORDER BY task.source_line ASC \
             LIMIT 1",
            [needle],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|error| panic!("query org element task join: {error}"));
    assert_eq!(orgid, expected_orgid);
}

fn assert_agent_memory_object_row_count_for_orgid(
    database_path: &std::path::Path,
    orgid: &str,
    expected: i64,
) {
    let connection = xiuxian_db_store::duckdb_crate::Connection::open(database_path)
        .unwrap_or_else(|error| panic!("open read-model duckdb: {error}"));
    let count = connection
        .query_row(
            "SELECT COUNT(*) FROM agent_org_memory_objects WHERE orgid = ?1",
            [orgid],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or_else(|error| panic!("query memory object row count for orgid: {error}"));
    assert_eq!(count, expected);
}

fn assert_agent_memory_object_projection(
    database_path: &std::path::Path,
    object_index: i64,
    expected: (&str, &str, &str, &str, &str, &str),
) {
    let connection = xiuxian_db_store::duckdb_crate::Connection::open(database_path)
        .unwrap_or_else(|error| panic!("open read-model duckdb: {error}"));
    let row = connection
        .query_row(
            "SELECT orgid, kind, facet, source_kind, source_key, value \
             FROM agent_org_memory_objects \
             WHERE object_index = ?1",
            [object_index],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .unwrap_or_else(|error| panic!("query memory object projection: {error}"));
    assert_eq!(
        row,
        (
            expected.0.to_string(),
            expected.1.to_string(),
            expected.2.to_string(),
            expected.3.to_string(),
            expected.4.to_string(),
            expected.5.to_string(),
        )
    );
}
