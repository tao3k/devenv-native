use std::collections::BTreeMap;

#[cfg(not(feature = "duckdb"))]
use xiuxian_db_store::SearchEngineContext;

use crate::search::SearchCorpusKind;

use crate::search::queries::sql::registration::{
    RegisteredSqlTable, RegisteredSqlViewSource, naming,
};

pub(crate) fn collect_repo_logical_views(
    tables: &mut BTreeMap<String, RegisteredSqlTable>,
) -> Vec<RegisteredSqlViewSource> {
    let mut view_sources = Vec::new();
    for corpus in [
        SearchCorpusKind::RepoContentChunk,
        SearchCorpusKind::RepoEntity,
    ] {
        let mut repo_tables = tables
            .values()
            .filter(|table| table.scope == "repo" && table.corpus == corpus.to_string())
            .cloned()
            .collect::<Vec<_>>();
        repo_tables.sort_by(|left, right| {
            left.repo_id
                .cmp(&right.repo_id)
                .then(left.sql_table_name.cmp(&right.sql_table_name))
        });
        if repo_tables.is_empty() {
            continue;
        }

        let logical_view_name = naming::repo_logical_view_name(corpus);
        view_sources.extend(repo_tables.iter().enumerate().map(|(index, table)| {
            RegisteredSqlViewSource::logical(logical_view_name.as_str(), table, index + 1)
        }));
        tables.insert(
            logical_view_name.clone(),
            RegisteredSqlTable::repo_logical(corpus, logical_view_name, repo_tables.len()),
        );
    }

    view_sources
}

#[cfg(not(feature = "duckdb"))]
pub(crate) async fn register_repo_logical_views(
    query_engine: &SearchEngineContext,
    tables: &BTreeMap<String, RegisteredSqlTable>,
) -> Result<(), String> {
    for (logical_view_name, view_sql) in collect_repo_logical_view_sqls(tables) {
        query_engine
            .session()
            .sql(view_sql.as_str())
            .await
            .map_err(|error| {
                format!(
                    "studio SQL Flight provider failed to register repo logical view `{logical_view_name}`: {error}"
                )
            })?;
    }
    Ok(())
}

pub(crate) fn collect_repo_logical_view_sqls(
    tables: &BTreeMap<String, RegisteredSqlTable>,
) -> Vec<(String, String)> {
    let mut statements = Vec::new();
    for corpus in [
        SearchCorpusKind::RepoContentChunk,
        SearchCorpusKind::RepoEntity,
    ] {
        let mut repo_tables = tables
            .values()
            .filter(|table| table.scope == "repo" && table.corpus == corpus.to_string())
            .cloned()
            .collect::<Vec<_>>();
        repo_tables.sort_by(|left, right| {
            left.repo_id
                .cmp(&right.repo_id)
                .then(left.sql_table_name.cmp(&right.sql_table_name))
        });
        if repo_tables.is_empty() {
            continue;
        }

        let logical_view_name = naming::repo_logical_view_name(corpus);
        statements.push((
            logical_view_name.clone(),
            build_repo_logical_view_sql(corpus, logical_view_name.as_str(), &repo_tables),
        ));
    }
    statements
}

pub(crate) fn build_repo_logical_view_sql(
    corpus: SearchCorpusKind,
    view_name: &str,
    repo_tables: &[RegisteredSqlTable],
) -> String {
    let union_query = match corpus {
        SearchCorpusKind::RepoContentChunk => build_repo_content_chunk_union_query(repo_tables),
        SearchCorpusKind::RepoEntity => build_repo_entity_union_query(repo_tables),
        other => panic!("unsupported repo logical view corpus `{other}`"),
    };
    format!("CREATE VIEW {view_name} AS {union_query}")
}

fn build_repo_content_chunk_union_query(repo_tables: &[RegisteredSqlTable]) -> String {
    repo_tables
        .iter()
        .enumerate()
        .map(|(index, table)| {
            let repo_id = escaped_repo_id(table);
            let source_alias = format!("source_{index}");
            format!(
                "SELECT '{repo_id}' AS repo_id, \
                 {source_alias}.path AS title, \
                 'file' AS doc_type, \
                 'code' AS code_tag, \
                 'file' AS file_tag, \
                 'kind:file' AS kind_tag, \
                 CASE WHEN TRIM({source_alias}.language) = '' THEN NULL ELSE CONCAT('lang:', {source_alias}.language) END AS language_tag, \
                 {source_alias}.* \
                 FROM {} AS {source_alias}",
                table.sql_table_name
            )
        })
        .collect::<Vec<_>>()
        .join(" UNION ALL ")
}

fn build_repo_entity_union_query(repo_tables: &[RegisteredSqlTable]) -> String {
    repo_tables
        .iter()
        .enumerate()
        .map(|(index, table)| {
            let repo_id = escaped_repo_id(table);
            let source_alias = format!("source_{index}");
            format!(
                "SELECT '{repo_id}' AS repo_id, {source_alias}.* FROM {} AS {source_alias}",
                table.sql_table_name
            )
        })
        .collect::<Vec<_>>()
        .join(" UNION ALL ")
}

fn escaped_repo_id(table: &RegisteredSqlTable) -> String {
    escape_sql_string_literal(
        table
            .repo_id
            .as_deref()
            .unwrap_or_else(|| panic!("repo table should carry repo_id")),
    )
}

fn escape_sql_string_literal(value: &str) -> String {
    value.replace('\'', "''")
}
