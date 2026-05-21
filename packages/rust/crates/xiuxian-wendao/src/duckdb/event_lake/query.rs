//! Filtered row queries for Wendao event-lake records.

use chrono::{TimeZone, Utc};
use duckdb::types::Type;
use duckdb::{Row, params_from_iter};
use xiuxian_db_store::duckdb::{ensure_duckdb_identifier, quoted_duckdb_identifier};

use super::record::WendaoEventRecord;
use super::schema::{
    CASE_ID_COLUMN, CREATED_AT_COLUMN, EVENT_TYPE_COLUMN, PAYLOAD_COLUMN, TENANT_ID_COLUMN,
    WENDAO_EVENT_LAKE_EVENTS_TABLE,
};

/// Default maximum number of event rows returned by a query.
pub const WENDAO_EVENT_QUERY_DEFAULT_LIMIT: u32 = 100;

/// Hard cap for one event-lake row query.
pub const WENDAO_EVENT_QUERY_MAX_LIMIT: u32 = 10_000;

/// Wendao event-row filters for bounded event-lake reads.
#[derive(Debug, Clone, PartialEq, Eq)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct WendaoEventQuery {
    /// Optional tenant or workspace boundary.
    pub tenant_id: Option<String>,
    /// Optional case, process, or workflow identifier.
    pub case_id: Option<String>,
    /// Optional event kind such as `bpmn.step`, `llm.call`, or `tool.call`.
    pub event_type: Option<String>,
    /// Maximum rows returned by the query.
    pub limit: u32,
}

impl Default for WendaoEventQuery {
    fn default() -> Self {
        Self {
            tenant_id: None,
            case_id: None,
            event_type: None,
            limit: WENDAO_EVENT_QUERY_DEFAULT_LIMIT,
        }
    }
}

impl WendaoEventQuery {
    /// Build an unfiltered bounded event query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a query scoped to one tenant and case.
    #[must_use]
    pub fn for_case(tenant_id: impl Into<String>, case_id: impl Into<String>) -> Self {
        Self::new().with_tenant_id(tenant_id).with_case_id(case_id)
    }

    /// Add or replace the tenant filter.
    #[must_use]
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Add or replace the case filter.
    #[must_use]
    pub fn with_case_id(mut self, case_id: impl Into<String>) -> Self {
        self.case_id = Some(case_id.into());
        self
    }

    /// Add or replace the event-type filter.
    #[must_use]
    pub fn with_event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = Some(event_type.into());
        self
    }

    /// Add or replace the row limit.
    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Validate this query before SQL rendering.
    ///
    /// # Errors
    ///
    /// Returns an error when the row limit is outside the bounded contract or
    /// when a present string filter is blank.
    pub fn validate(&self) -> Result<(), String> {
        if self.limit == 0 {
            return Err("Wendao event query limit must be greater than zero".to_string());
        }
        if self.limit > WENDAO_EVENT_QUERY_MAX_LIMIT {
            return Err(format!(
                "Wendao event query limit {} exceeds max {}",
                self.limit, WENDAO_EVENT_QUERY_MAX_LIMIT
            ));
        }
        validate_optional_filter(self.tenant_id.as_deref(), "tenant_id")?;
        validate_optional_filter(self.case_id.as_deref(), "case_id")?;
        validate_optional_filter(self.event_type.as_deref(), "event_type")?;
        Ok(())
    }
}

/// Query event rows from an attached Wendao event-lake catalog.
///
/// # Errors
///
/// Returns an error when the catalog alias or query is invalid, when `DuckDB`
/// rejects the SQL, or when persisted timestamp values cannot be converted
/// back into Wendao event records.
pub fn query_wendao_events(
    connection: &duckdb::Connection,
    catalog_alias: &str,
    query: &WendaoEventQuery,
) -> Result<Vec<WendaoEventRecord>, String> {
    let (sql, params) = build_wendao_event_query_sql(catalog_alias, query)?;
    let mut statement = connection
        .prepare(sql.as_str())
        .map_err(|error| format!("failed to prepare Wendao event-lake row query: {error}"))?;
    let mut rows = statement
        .query(params_from_iter(params.iter()))
        .map_err(|error| format!("failed to execute Wendao event-lake row query: {error}"))?;
    let mut events = Vec::with_capacity(query_result_capacity(query.limit));
    while let Some(row) = rows
        .next()
        .map_err(|error| format!("failed to read Wendao event-lake row: {error}"))?
    {
        events.push(
            read_wendao_event_row(row)
                .map_err(|error| format!("failed to read Wendao event-lake row: {error}"))?,
        );
    }
    Ok(events)
}

fn build_wendao_event_query_sql(
    catalog_alias: &str,
    query: &WendaoEventQuery,
) -> Result<(String, Vec<String>), String> {
    ensure_duckdb_identifier(catalog_alias, "DuckLake catalog")?;
    query.validate()?;

    let catalog = quoted_duckdb_identifier(catalog_alias);
    let table = quoted_duckdb_identifier(WENDAO_EVENT_LAKE_EVENTS_TABLE);
    let tenant_id = quoted_duckdb_identifier(TENANT_ID_COLUMN);
    let case_id = quoted_duckdb_identifier(CASE_ID_COLUMN);
    let event_type = quoted_duckdb_identifier(EVENT_TYPE_COLUMN);
    let payload = quoted_duckdb_identifier(PAYLOAD_COLUMN);
    let created_at = quoted_duckdb_identifier(CREATED_AT_COLUMN);

    let mut conditions = Vec::new();
    let mut params = Vec::new();
    push_optional_filter(
        &mut conditions,
        &mut params,
        TENANT_ID_COLUMN,
        query.tenant_id.as_deref(),
    );
    push_optional_filter(
        &mut conditions,
        &mut params,
        CASE_ID_COLUMN,
        query.case_id.as_deref(),
    );
    push_optional_filter(
        &mut conditions,
        &mut params,
        EVENT_TYPE_COLUMN,
        query.event_type.as_deref(),
    );

    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    let limit = query.limit;
    Ok((
        format!(
            "SELECT {tenant_id}, {case_id}, {event_type}, {payload}, epoch_ms({created_at})::BIGINT \
             FROM {catalog}.{table}{where_sql} \
             ORDER BY {created_at}, {tenant_id}, {case_id}, {event_type} \
             LIMIT {limit}"
        ),
        params,
    ))
}

fn query_result_capacity(limit: u32) -> usize {
    usize::try_from(limit).unwrap_or(WENDAO_EVENT_QUERY_MAX_LIMIT as usize)
}

fn read_wendao_event_row(row: &Row<'_>) -> duckdb::Result<WendaoEventRecord> {
    let payload_text: String = row.get(3)?;
    let created_at_ms: i64 = row.get(4)?;
    let created_at = Utc
        .timestamp_millis_opt(created_at_ms)
        .single()
        .ok_or_else(|| {
            duckdb::Error::FromSqlConversionFailure(
                4,
                Type::BigInt,
                Box::new(std::io::Error::other(format!(
                    "invalid UTC millisecond timestamp `{created_at_ms}`"
                ))),
            )
        })?;

    Ok(WendaoEventRecord::from_trusted_payload_json(
        row.get::<_, String>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        payload_text,
        created_at,
    ))
}

fn push_optional_filter(
    conditions: &mut Vec<String>,
    params: &mut Vec<String>,
    column_name: &str,
    value: Option<&str>,
) {
    if let Some(value) = value {
        conditions.push(format!("{} = ?", quoted_duckdb_identifier(column_name)));
        params.push(value.trim().to_string());
    }
}

fn validate_optional_filter(value: Option<&str>, label: &str) -> Result<(), String> {
    if matches!(value, Some(value) if value.trim().is_empty()) {
        return Err(format!("Wendao event query {label} filter cannot be blank"));
    }
    Ok(())
}
