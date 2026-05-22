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
